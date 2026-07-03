use std::{io, net::SocketAddr, sync::Arc};

use catplay_util::{EventSleeper, LazyAsync};
use futures::StreamExt;
use log::{debug, trace, warn};
use tokio::{
    io::AsyncWriteExt,
    net::{
        TcpStream as TokioTcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    select,
    sync::{Mutex as TokioMutex, watch},
};
use tokio_util::{codec::FramedRead, sync::CancellationToken};

use super::{BytesMutQueue, TcpBootstrap, TcpSession, TcpSinkBuffer};
use crate::CItem;

#[derive(Debug, PartialEq, Clone)]
pub enum TcpState<E> {
    Connecting,
    ConnectFailed(E),
    Connected,
    Unwritable,
    Eof(Option<E>),
}

pub struct TcpHelperTask<F: TcpSession> {
    session: Arc<TokioMutex<F>>,
    composites: BytesMutQueue,
    token: CancellationToken,
    state: TcpHelperTaskState<F>,
    sleep_outcome: Option<TcpSleepOutcome<CItem<F::Codec>, F::Error>>,
    ext_status: watch::Sender<TcpState<F::Error>>,
    had_shutdown: bool,
}

enum TcpHelperTaskState<F: TcpSession> {
    Bootstrapping { socket: LazyAsync<io::Result<TokioTcpStream>> },
    Initialized(TcpHelperInitialized<F>),
    Eof,
}

struct TcpHelperInitialized<F: TcpSession> {
    framed: FramedRead<OwnedReadHalf, F::Codec>,
    write: OwnedWriteHalf,
    local_addr: Option<SocketAddr>,
    peer_addr: Option<SocketAddr>,
}

enum TcpSleepOutcome<Item, Error> {
    Continue,
    Message(Item),
    Eof,
    Cancelled,
    Error(Error),
}

enum TcpReconcileOutcome {
    Continue,
    Eof,
    Cancelled,
}

impl<F: TcpSession> TcpHelperTask<F> {
    pub fn new(bootstrap: TcpBootstrap, token: CancellationToken, session: F) -> Self {
        let ch = watch::channel(TcpState::Connecting);
        Self {
            session: Arc::new(TokioMutex::new(session)),
            composites: BytesMutQueue::new(),
            token,
            state: TcpHelperTaskState::Bootstrapping {
                socket: LazyAsync::new(move || async move { bootstrap.resolve().await }),
            },
            sleep_outcome: None,
            had_shutdown: false,
            ext_status: ch.0,
        }
    }

    pub fn token(&mut self) -> CancellationToken {
        self.token.clone()
    }

    pub fn state(&mut self) -> watch::Receiver<TcpState<F::Error>> {
        self.ext_status.subscribe()
    }

    pub fn session(&mut self) -> Arc<TokioMutex<F>> {
        self.session.clone()
    }

    pub fn update_state(&mut self, state: TcpState<F::Error>) {
        self.ext_status.send_replace(state);
    }

    pub async fn run(&mut self) {
        let ret = self.work().await;
        let (local_addr, peer_addr) = self.socket_addrs();
        warn!("EOF on TCP socket {:?} -> {:?}! Error: {:?}", local_addr, peer_addr, ret,);
        let err = ret.err();
        let ext_err = err.clone();

        if let Some((stream, composites)) = self.write_and_composites_mut() {
            let _ = TcpSinkBuffer::<F>::flush_queue(stream, composites);
            let _ret = stream.shutdown().await;
            // warn!("shutdown() returned {ret:?}");
        }

        // if let Ok((stream, composites)) = self.initialized_and_composites_mut() {
        //     warn!("Starting stream.framed.next() POLL");
        //     let ret = stream.framed.next().await;
        //     warn!("Finished stream.framed.next() POLL {ret:?}");
        // }

        // fd is dropped here
        self.state = TcpHelperTaskState::Eof;

        {
            let mut session = self.session.lock().await;
            session.on_eof(err).await;
            session.shutdown().await;
            self.had_shutdown = true;
        }
        self.update_state(TcpState::Eof(ext_err));
    }

    async fn work(&mut self) -> Result<(), F::Error> {
        if self.token.is_cancelled() {
            debug!("Returning, because cancel token is active");
            return Ok(());
        }

        loop {
            match self.reconcile().await? {
                TcpReconcileOutcome::Continue => {}
                TcpReconcileOutcome::Eof => {
                    debug!("Observed EOF on TCP socket!");
                    break;
                }
                TcpReconcileOutcome::Cancelled => {
                    debug!("Returning, because cancel token is active");
                    return Ok(());
                }
            }

            self.sleep().await;
        }

        Ok(())
    }

    async fn init_connected(&mut self, socket: TokioTcpStream) -> Result<(), F::Error> {
        let session_arc = self.session.clone();
        let mut session = session_arc.lock().await;

        let local_addr = socket.local_addr().ok();
        let peer_addr = socket.peer_addr().ok();

        if let Some(local_addr) = local_addr {
            session.on_local_addr(local_addr).await?;
        }
        if let Some(peer_addr) = peer_addr {
            session.on_peer_addr(peer_addr).await?;
        }

        session.on_connected().await?;

        let mut stream_std = socket.into_std()?;
        session.init_stream(&mut stream_std)?;

        let stream = TokioTcpStream::from_std(stream_std)?;

        let codec = session.init_codec()?;
        let (read, write) = stream.into_split();
        let framed = FramedRead::new(read, codec);

        self.state = TcpHelperTaskState::Initialized(TcpHelperInitialized {
            framed,
            write,
            local_addr,
            peer_addr,
        });
        Ok(())
    }

    async fn reconcile(&mut self) -> Result<TcpReconcileOutcome, F::Error> {
        if self.token.is_cancelled() {
            return Ok(TcpReconcileOutcome::Cancelled);
        }

        if let Some(outcome) = self.sleep_outcome.take() {
            match outcome {
                TcpSleepOutcome::Continue => {}
                TcpSleepOutcome::Message(msg) => self.on_msg(msg).await?,
                TcpSleepOutcome::Eof => return Ok(TcpReconcileOutcome::Eof),
                TcpSleepOutcome::Cancelled => return Ok(TcpReconcileOutcome::Cancelled),
                TcpSleepOutcome::Error(e) => return Err(e),
            }
        }

        if let TcpHelperTaskState::Bootstrapping { socket } = &mut self.state
            && let Some(socket) = socket.take()
        {
            match socket {
                Ok(socket) => {
                    self.init_connected(socket).await?;
                    self.update_state(TcpState::Connected);
                }
                Err(err) => {
                    let err: F::Error = err.into();
                    self.update_state(TcpState::ConnectFailed(err.clone()));
                    return Err(err);
                }
            }
        }

        if let TcpHelperTaskState::Bootstrapping { .. } = self.state {
            return Ok(TcpReconcileOutcome::Continue);
        }

        let session_arc = self.session.clone();
        let mut session = session_arc.lock().await;
        let (init, composites) = self.initialized_and_composites_mut()?;
        let mut out = TcpSinkBuffer::new(composites, init.framed.decoder_mut());
        session.reconcile(&mut out).await?;
        Ok(TcpReconcileOutcome::Continue)
    }

    async fn sleep(&mut self) {
        let token = self.token.clone();
        if token.is_cancelled() {
            self.sleep_outcome = Some(TcpSleepOutcome::Cancelled);
            return;
        }

        match &mut self.state {
            TcpHelperTaskState::Bootstrapping { socket } => {
                select! {
                    _ = token.cancelled() => {
                        self.sleep_outcome = Some(TcpSleepOutcome::Cancelled);
                    }
                    _ = socket.sleep() => {
                        self.sleep_outcome = Some(TcpSleepOutcome::Continue);
                    }
                }
            }
            _ => {
                let session_arc = self.session.clone();
                let mut session = session_arc.lock().await;
                let (init, composites) = match self.initialized_and_composites_mut() {
                    Ok(v) => v,
                    Err(e) => {
                        self.sleep_outcome = Some(TcpSleepOutcome::Error(e));
                        return;
                    }
                };

                let write = &mut init.write;
                let framed = &mut init.framed;
                select! {
                    _ = token.cancelled() => {
                        self.sleep_outcome = Some(TcpSleepOutcome::Cancelled);
                    }
                    _ = write.writable(), if !composites.is_empty() => {
                        if let Err(e) = TcpSinkBuffer::<F>::flush_queue(write, composites) {
                            self.sleep_outcome = Some(TcpSleepOutcome::Error(e.into()));
                            return;
                        }
                        self.sleep_outcome = Some(TcpSleepOutcome::Continue);
                    }
                    result = framed.next() => {
                        match result {
                            Some(Ok(msg)) => self.sleep_outcome = Some(TcpSleepOutcome::Message(msg)),
                            Some(Err(e)) => self.sleep_outcome = Some(TcpSleepOutcome::Error(e.into())),
                            None => self.sleep_outcome = Some(TcpSleepOutcome::Eof),
                        }
                    }
                    Some(_event) = session.sleep() => {
                        self.sleep_outcome = Some(TcpSleepOutcome::Continue);
                    }
                }
            }
        }
    }

    async fn on_msg(&mut self, msg: CItem<F::Codec>) -> Result<(), F::Error> {
        #[cfg(debug_assertions)]
        trace!("Received frame: {msg:?}");

        let session_arc = self.session.clone();
        let mut session = session_arc.lock().await;
        let (init, composites) = self.initialized_and_composites_mut()?;
        let mut out = TcpSinkBuffer::new(composites, init.framed.decoder_mut());
        session.on_msg(&mut out, msg).await
    }

    fn write_and_composites_mut(&mut self) -> Option<(&mut OwnedWriteHalf, &mut BytesMutQueue)> {
        match (&mut self.state, &mut self.composites) {
            (TcpHelperTaskState::Initialized(init), composites) => Some((&mut init.write, composites)),
            (TcpHelperTaskState::Bootstrapping { .. } | TcpHelperTaskState::Eof, _) => None,
        }
    }

    fn initialized_and_composites_mut(&mut self) -> Result<(&mut TcpHelperInitialized<F>, &mut BytesMutQueue), F::Error> {
        match (&mut self.state, &mut self.composites) {
            (TcpHelperTaskState::Initialized(init), composites) => Ok((init, composites)),
            (TcpHelperTaskState::Bootstrapping { .. }, _) => {
                Err(io::Error::new(io::ErrorKind::WouldBlock, "TCP task still bootstrapping").into())
            }
            (TcpHelperTaskState::Eof, _) => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "TCP task reached EOF").into()),
        }
    }

    fn socket_addrs(&self) -> (Option<SocketAddr>, Option<SocketAddr>) {
        match &self.state {
            TcpHelperTaskState::Initialized(init) => (init.local_addr, init.peer_addr),
            TcpHelperTaskState::Bootstrapping { .. } | TcpHelperTaskState::Eof => (None, None),
        }
    }
}
