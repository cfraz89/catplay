use std::{
    marker::PhantomData,
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    sync::Arc,
    time::Duration,
};

use catplay_util::{AsyncShutdown, EventReconciler, EventSleeper, EventToken};
use log::debug;
use tokio::{
    spawn,
    sync::{Mutex as TokioMutex, watch},
};
use tokio_util::{sync::CancellationToken, task::AbortOnDropHandle};

use super::{TcpBootstrap, TcpHelperTask, TcpSession, TcpState};

pub struct TcpHelper<F: TcpSession> {
    handle: Option<AbortOnDropHandle<()>>,
    token: CancellationToken,
    status: watch::Receiver<TcpState<F::Error>>,
    _phantom: PhantomData<F>,
    _task: Arc<TokioMutex<TcpHelperTask<F>>>,
}

impl<F: TcpSession> TcpHelper<F> {
    /// Start connecting in background to remote.
    pub fn with_bootstrap(bootstrap: TcpBootstrap, session: F) -> Result<Self, F::Error> {
        let token = CancellationToken::new();
        let mut task = TcpHelperTask::new(bootstrap, token.clone(), session);
        let status = task.state();
        let m = Arc::new(TokioMutex::new(task));

        // SAFETY: TcpHelper recovers instance of TcpHelperTask and TcpSession to guarantee
        // that they outlive given instance of TcpHelper in case of panics
        let m0 = m.clone();
        let handle = AbortOnDropHandle::new(spawn(async move {
            m0.lock().await.run().await;
        }));

        let me = Self {
            handle: Some(handle),
            token,
            status,
            _phantom: PhantomData,
            _task: m,
        };
        Ok(me)
    }

    pub fn connect<A: ToSocketAddrs>(ip: A, session: F) -> Result<Self, F::Error> {
        Self::with_bootstrap(TcpBootstrap::connect(ip, Duration::ZERO)?, session)
    }

    pub fn connect_timeout<A: ToSocketAddrs>(ip: A, timeout: Duration, session: F) -> Result<Self, F::Error> {
        Self::with_bootstrap(TcpBootstrap::connect(ip, timeout)?, session)
    }

    pub fn accept_once<A: ToSocketAddrs>(bind_ip: A, session: F) -> Result<(Self, SocketAddr), F::Error> {
        let (bootstrap, local_addr) = TcpBootstrap::accept_once(bind_ip, Duration::ZERO)?;
        Ok((Self::with_bootstrap(bootstrap, session)?, local_addr))
    }

    pub fn accept_timeout<A: ToSocketAddrs>(bind_ip: A, timeout: Duration, session: F) -> Result<(Self, SocketAddr), F::Error> {
        let (bootstrap, local_addr) = TcpBootstrap::accept_once(bind_ip, timeout)?;
        Ok((Self::with_bootstrap(bootstrap, session)?, local_addr))
    }

    pub fn external(stream: TcpStream, session: F) -> Result<Self, F::Error> {
        Self::with_bootstrap(TcpBootstrap::external(stream)?, session)
    }
}
impl<F: TcpSession> AsyncShutdown for TcpHelper<F> {
    async fn shutdown(&mut self) {
        self.token.cancel();
        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }
    }
}

impl<F: TcpSession> EventSleeper for TcpHelper<F> {
    async fn sleep(&mut self) -> Option<EventToken> {
        match self.status.changed().await {
            Ok(()) => Some(EventToken(1)),
            Err(_) => None,
        }
    }
}

impl<F: TcpSession> EventReconciler for TcpHelper<F> {
    type Error = Option<F::Error>;

    async fn reconcile(&mut self) -> Result<(), Option<F::Error>> {
        match &*self.status.borrow() {
            TcpState::Connecting | TcpState::Connected | TcpState::Unwritable => Ok(()),
            TcpState::ConnectFailed(err) | TcpState::Eof(Some(err)) => Err(Some(err.clone())),
            TcpState::Eof(None) => Err(None),
        }
    }
}

impl<F: TcpSession> Drop for TcpHelper<F> {
    fn drop(&mut self) {
        debug!("Dropped TcpHelper");
        self.token.cancel();
    }
}
