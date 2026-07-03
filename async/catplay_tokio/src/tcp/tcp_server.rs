use catplay_util::{ArcBox, AsyncShutdown, EventReconciler, EventSleeper, EventToken};
use log::{debug, error, info};
use std::sync::Arc;
use std::{
    io,
    marker::PhantomData,
    net::{IpAddr, SocketAddr, TcpListener},
};
use tokio::{spawn, sync::watch, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{TcpServerTask, tcp::TcpSession};

pub struct TcpServer<T>
where
    T: TcpSession,
{
    task: Option<JoinHandle<()>>,
    token: CancellationToken,
    listener_status: watch::Receiver<Result<(), ArcBox<io::Error>>>,
    _phantom: PhantomData<T>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TcpServerState {
    Listening,
    Eof(Option<ArcBox<io::Error>>),
}

impl<T> TcpServer<T>
where
    T: TcpSession,
{
    pub fn new<F: Fn(SocketAddr) -> T + Send + Sync + 'static>(addrs: &[IpAddr], port: u16, factory: F) -> io::Result<Self> {
        let token = CancellationToken::new();
        let task = TcpServerTask::new(addrs, port, Arc::new(factory), token.clone())?;
        info!("TcpServer now listening on port {port} and ips {addrs:?}");

        Ok(Self::start(task, token))
    }

    pub fn new_with_listeners<F: Fn(SocketAddr) -> T + Send + Sync + 'static>(listeners: Vec<TcpListener>, factory: F) -> io::Result<Self> {
        let token = CancellationToken::new();
        let task = TcpServerTask::new_with_listeners(listeners, Arc::new(factory), token.clone())?;

        Ok(Self::start(task, token))
    }

    pub fn bind_iface<F: Fn(SocketAddr) -> T + Send + Sync + 'static>(iface: &str, port: u16, factory: F) -> io::Result<Self> {
        let token = CancellationToken::new();
        let task = TcpServerTask::bind_iface(iface, port, Arc::new(factory), token.clone())?;

        Ok(Self::start(task, token))
    }

    fn start(task: TcpServerTask<T>, token: CancellationToken) -> Self {
        let (status_tx, status_rx) = watch::channel(Ok(()));

        let task = {
            spawn(async move {
                if let Err(e) = task.run().await {
                    error!("TcpServer listener failed: {e:?}");
                    let _ = status_tx.send_replace(Err(e));
                }
            })
        };

        Self {
            task: Some(task),
            token,
            listener_status: status_rx,
            _phantom: PhantomData,
        }
    }
}

impl<T> AsyncShutdown for TcpServer<T>
where
    T: TcpSession,
{
    async fn shutdown(&mut self) {
        debug!("TcpServer is shutting down!");

        self.token.cancel();
        if let Some(thread_handle) = self.task.take() {
            let _ = thread_handle.await;
        }
    }
}

impl<T> EventReconciler for TcpServer<T>
where
    T: TcpSession,
{
    type Error = ArcBox<io::Error>;

    async fn reconcile(&mut self) -> Result<(), Self::Error> {
        match self.listener_status.borrow().as_ref() {
            Ok(()) => Ok(()),
            Err(err) => Err(err.clone()),
        }
    }
}

impl<T> EventSleeper for TcpServer<T>
where
    T: TcpSession,
{
    async fn sleep(&mut self) -> Option<EventToken> {
        match self.listener_status.changed().await {
            Ok(()) => Some(EventToken(1)),
            Err(_) => None,
        }
    }
}

impl<T> Drop for TcpServer<T>
where
    T: TcpSession,
{
    fn drop(&mut self) {
        debug!("TcpServer was dropped!");
        self.token.cancel();
    }
}
