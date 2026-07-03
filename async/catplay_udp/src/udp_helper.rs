use std::{net::SocketAddr, sync::Arc};

use crate::{UdpHelperSink, UdpHelperTask, UdpSession, udp_helper_task::Inner};
use catplay_util::{AsyncShutdown, EventReconciler, EventSleeper, EventToken};
use log::debug;
use tokio::spawn;

pub struct UdpHelper<const DATAGRAM_SIZE_MAX: usize, const PAD: usize, F: UdpSession> {
    inner: Arc<Inner<F>>,
    eof_consumed: bool,
}

impl<const DATAGRAM_SIZE: usize, const PAD: usize, F: UdpSession> EventSleeper for UdpHelper<DATAGRAM_SIZE, PAD, F> {
    async fn sleep(&mut self) -> Option<EventToken> {
        if self.eof_consumed {
            return None;
        }

        self.wait().await;
        self.eof_consumed = true;
        Some(EventToken(1))
    }
}

impl<const DATAGRAM_SIZE: usize, const PAD: usize, F: UdpSession> EventReconciler for UdpHelper<DATAGRAM_SIZE, PAD, F> {
    type Error = Option<F::Error>;

    async fn reconcile(&mut self) -> Result<(), Option<F::Error>> {
        self.status()
    }
}

#[allow(unused)]
impl<const DATAGRAM_SIZE: usize, const PAD: usize, F: UdpSession> UdpHelper<DATAGRAM_SIZE, PAD, F> {
    /// Waits for session to achieve `Eof` status or returns immediately if already `Eof`.
    ///
    /// **After Eof status, the session should be dropped to avoid busy-looping on [Self::wait].**
    async fn wait(&mut self) {
        let mut sub = self.inner.eof_reason.subscribe();
        let ret = sub.wait_for(|e| e.is_err()).await;
        debug_assert!(ret.is_ok());
    }

    /// Returns true if session has reached `Eof` status.
    fn eof(&mut self) -> bool {
        self.inner.eof_reason.borrow().is_err()
    }

    /// Returns Ok if session is still running, Err(None) if closed without error or Err(Error)
    /// if closed with error.
    fn status(&mut self) -> Result<(), Option<F::Error>> {
        self.inner.eof_reason.borrow().clone()
    }

    /// Connect to remote using provided bind address and return local bind address (corrected for port) and [UdpHelperSink].
    ///
    /// Returned [UdpHelperSink] is a utility that allows OOB datagram sends from any thread.
    pub fn connect(addr: SocketAddr, session: F) -> Result<(Self, SocketAddr, UdpHelperSink<F>), F::Error> {
        let (task, local_addr, sink) = UdpHelperTask::<DATAGRAM_SIZE, PAD, F>::connect(addr, session)?;
        Ok((Self::start(task), local_addr, sink))
    }

    /// Bind socket to provided address and return bind address corrected for port.
    ///
    /// Socket will receive datagrams from anonymous peers unless(until) call to connect() follows with [SocketAddr] of the remote end.
    pub fn bind(addr: SocketAddr, session: F) -> Result<(Self, SocketAddr), F::Error> {
        let (task, local) = UdpHelperTask::<DATAGRAM_SIZE, PAD, F>::bind(addr, session)?;
        Ok((Self::start(task), local))
    }

    /// Create a local pair of connected sockets.
    #[allow(clippy::type_complexity)]
    pub fn local_pair(session1: F, session2: F) -> Result<((Self, UdpHelperSink<F>), (Self, UdpHelperSink<F>)), F::Error> {
        let ((task1, task1_sink), (task2, task2_sink)) = UdpHelperTask::<DATAGRAM_SIZE, PAD, F>::local_pair(session1, session2)?;
        Ok(((Self::start(task1), task1_sink), (Self::start(task2), task2_sink)))
    }

    /// Complete the two-way connect process started by `bind()`, return [UdpHelperSink].
    ///
    /// Returned [UdpHelperSink] is a utility that allows OOB datagram sends from any thread.
    pub fn connect_finish(&self, remote: SocketAddr) -> Result<UdpHelperSink<F>, F::Error> {
        self.inner.socket.connect(remote)?;
        UdpHelperTask::<DATAGRAM_SIZE, PAD, F>::finish_peer_inner(&self.inner, remote)
    }

    /// Complete the two-way connect process started by `bind()`, by don't perform full `connect()` flow
    /// that would otherwise block the socket from receiving datagrams from anonymous source ports.
    ///
    /// Other than that, the session's behavior stays identical, including session termination on ICMP errors received when attempting to send
    /// datagrams to the peer sink.
    pub fn pseudo_connect(&self, remote: SocketAddr) -> Result<UdpHelperSink<F>, F::Error> {
        UdpHelperTask::<DATAGRAM_SIZE, PAD, F>::finish_peer_inner(&self.inner, remote)
    }

    fn start(mut task: UdpHelperTask<DATAGRAM_SIZE, PAD, F>) -> Self {
        let inner = task.inner.clone();
        let handle = spawn(async move {
            task.run().await;
        });
        inner.task.lock().unwrap().replace(handle);
        debug!("Started UDP worker");
        Self {
            inner,
            eof_consumed: false,
        }
    }

    /// Shutdown and return inner session object (in Eof state).
    pub async fn into_inner(mut self) -> F {
        self.shutdown().await;

        let mut session_holder = self.inner.session.lock().await;
        session_holder.take().unwrap().into_inner()
    }
}

impl<const DATAGRAM_SIZE: usize, const PAD: usize, F: UdpSession> AsyncShutdown for UdpHelper<DATAGRAM_SIZE, PAD, F> {
    async fn shutdown(&mut self) {
        debug!("UdpHelper shutdown was called!");
        self.inner.cancel_token.cancel();

        let mut task = self.inner.task.lock().unwrap().take();
        if let Some(task) = task.as_mut() {
            debug!("Waiting for task to terminate...");
            let _ = task.await;
            debug!("Task terminated");
        }
    }
}

impl<const DATAGRAM_SIZE: usize, const PAD: usize, F: UdpSession> Drop for UdpHelper<DATAGRAM_SIZE, PAD, F> {
    fn drop(&mut self) {
        debug!("UdpHelper was dropped!");
        self.inner.cancel_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use catplay_tracing::logger::setup_test_logger;
    use catplay_util::{EventSleeper, EventToken};
    use log::debug;
    use tokio::time::sleep;

    use crate::{UdpHelper, UdpSession, UdpSocketPeer};
    use std::time::Duration;

    use std::{io, sync::Arc};

    #[derive(Debug, Clone, thiserror::Error)]
    pub enum BasicIoError {
        #[error("I/O error: {0:?}")]
        Io(Arc<io::Error>),
    }

    impl From<io::Error> for BasicIoError {
        fn from(value: io::Error) -> Self {
            Self::Io(value.into())
        }
    }

    #[derive(Debug, Default, PartialEq)]
    struct SessionTest {
        had_eof: bool,
        had_connect: bool,
        had_reconcile: bool,
        had_reconcile_peer: bool,
        had_recv: bool,
        had_response: bool,
        had_sleep: bool,
        had_multimsg: usize,
    }

    #[derive(Debug, Default)]
    struct SessionNoOp {}
    impl UdpSession for SessionNoOp {
        type Error = BasicIoError;

        fn on_datagram(
            &mut self,
            _data: &mut [u8],
            _peer: std::net::SocketAddr,
            _sink: &dyn UdpSocketPeer<Self>,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    impl EventSleeper for SessionNoOp {}

    impl UdpSession for SessionTest {
        type Error = BasicIoError;

        fn on_datagram(&mut self, data: &mut [u8], _peer: std::net::SocketAddr, sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
            debug!("Received {data:?}");
            if data == [2, 2, 2, 2] {
                self.had_recv = true;
                let _ = sink.send(&[4, 3, 2, 1]);
            }

            if data == [4, 3, 2, 1] {
                self.had_response = true;
            }

            if data == [3, 3, 3, 3] || data == [4, 4, 4, 4] {
                self.had_multimsg += 1;
            }

            Ok(())
        }

        fn on_connect(&mut self, _peer: std::net::SocketAddr, sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
            let _ = sink.send(&[2, 2, 2, 2]);
            let _ = sink.send_multiple(&[&[3, 3, 3, 3], &[4, 4, 4, 4]]);
            self.had_connect = true;
            Ok(())
        }

        fn on_eof(&mut self, _error: Option<Self::Error>) {
            self.had_eof = true;
        }

        fn reconcile(&mut self, peer: Option<&dyn UdpSocketPeer<Self>>) -> Result<(), Self::Error> {
            self.had_reconcile = true;
            self.had_reconcile_peer |= peer.is_some();
            Ok(())
        }
    }

    impl EventSleeper for SessionTest {
        async fn sleep(&mut self) -> Option<EventToken> {
            self.had_sleep = true;
            None
        }
    }

    #[tokio::test]
    async fn test_basic_session() {
        setup_test_logger(true);
        let (socket1, socket2) = UdpHelper::<1500, 0, _>::local_pair(SessionTest::default(), SessionTest::default()).unwrap();

        sleep(Duration::from_secs(1)).await;

        let c = socket1.0.into_inner().await;
        let s = socket2.0.into_inner().await;

        let expected = SessionTest {
            had_eof: true,
            had_connect: true,
            had_reconcile: true,
            had_reconcile_peer: true,
            had_recv: true,
            had_response: true,
            had_sleep: true,
            had_multimsg: 2,
        };

        assert_eq!(c, expected);
        assert_eq!(s, expected);
    }

    #[tokio::test]
    async fn test_icmp_oob() {
        setup_test_logger(true);
        let (socket1, mut socket2) = UdpHelper::<1500, 0, _>::local_pair(SessionNoOp::default(), SessionNoOp::default()).unwrap();

        // sleep(Duration::from_secs(1)).await;
        drop(socket1);

        for _ in 0..10 {
            debug!("Next send...");
            if socket2.1.send(&[1, 2, 3, 4]).is_err() {
                break;
            }

            sleep(Duration::from_secs(1)).await;
        }

        socket2.0.wait().await;
        assert!(socket2.0.eof());

        let status = socket2.0.status().err().unwrap().map_or(Ok(()), Err);
        debug!("Status: {status:?}");
        status.expect_err("ICMP error during sendmsg()");
    }
}
