use catplay_util::{ArcBox, EventReconciler, EventSleeper, EventToken};
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use log::{debug, error, info, warn};
use std::sync::Arc;
use std::{
    future::Future,
    io,
    marker::PhantomData,
    net::{IpAddr, SocketAddr, TcpListener},
    pin::Pin,
};
use tokio::{net::TcpListener as TokioTcpListener, select};
use tokio_util::sync::CancellationToken;

use crate::{TcpBootstrap, TcpHelperTask, bind_iface_ipv4, bind_iface_ipv6, tcp::TcpSession};

type ListenerFuture = Pin<Box<dyn Future<Output = io::Result<()>> + Send>>;

pub struct TcpServerTask<T>
where
    T: TcpSession,
{
    listeners: FuturesUnordered<ListenerFuture>,
    listener_status: Result<(), ArcBox<io::Error>>,
    _phantom: PhantomData<T>,
}

impl<T> TcpServerTask<T>
where
    T: TcpSession,
{
    pub fn new(
        addrs: &[IpAddr],
        port: u16,
        factory: Arc<dyn Fn(SocketAddr) -> T + Send + Sync + 'static>,
        token: CancellationToken,
    ) -> io::Result<Self> {
        let listeners = Self::create_listeners(addrs, port)?;
        Self::new_with_listeners(listeners, factory, token)
    }

    pub fn new_with_listeners(
        listeners: Vec<TcpListener>,
        factory: Arc<dyn Fn(SocketAddr) -> T + Send + Sync + 'static>,
        token: CancellationToken,
    ) -> io::Result<Self> {
        let outer = FuturesUnordered::new();
        for listener in listeners {
            let ip = listener.local_addr()?;
            let factory = factory.clone();
            let listener = TokioTcpListener::from_std(listener)?;
            let token = token.clone();
            outer.push(Box::pin(async move { Self::listener_loop(ip, listener, factory, token).await }) as ListenerFuture);
        }

        Ok(Self {
            listeners: outer,
            listener_status: Ok(()),
            _phantom: PhantomData,
        })
    }

    pub fn bind_iface(
        iface: &str,
        port: u16,
        factory: Arc<dyn Fn(SocketAddr) -> T + Send + Sync + 'static>,
        token: CancellationToken,
    ) -> io::Result<Self> {
        match bind_iface_ipv6(port, iface) {
            Err(err) => match bind_iface_ipv4(port, iface) {
                Ok(ipv4) => {
                    ipv4.set_nonblocking(true)?;
                    warn!(
                        "Auto-downgraded from IPv6 to IPv4 when binding {iface}:{port}, is kernel compiled without IPv6 support? Error: {err:?}"
                    );
                    Self::new_with_listeners(vec![ipv4], factory, token)
                }
                Err(err) => Err(err),
            },
            Ok(ipv6) => {
                ipv6.set_nonblocking(true)?;
                info!("TcpServer now listening on {iface}:{port} in IPv6+IPv4 mode");
                Self::new_with_listeners(vec![ipv6], factory, token)
            }
        }
    }

    fn create_listeners(ips: &[IpAddr], port: u16) -> io::Result<Vec<TcpListener>> {
        let mut listeners = Vec::new();
        for ip in ips {
            match TcpListener::bind((*ip, port)) {
                Ok(v) => {
                    v.set_nonblocking(true)?;
                    listeners.push(v)
                }
                Err(err) => {
                    error!("Failed to bind to {ip}:{port} {err:?}");
                    return Err(err);
                }
            };
        }
        Ok(listeners)
    }

    pub async fn run(mut self) -> Result<(), ArcBox<io::Error>> {
        loop {
            self.reconcile().await?;
            if self.sleep().await.is_none() {
                return Ok(());
            }
        }
    }

    async fn listener_loop(
        ip: SocketAddr,
        listener: TokioTcpListener,
        factory: Arc<dyn Fn(SocketAddr) -> T + Send + Sync + 'static>,
        token: CancellationToken,
    ) -> io::Result<()> {
        let mut inner = FuturesUnordered::new();
        let mut listener_err = None;

        loop {
            select! {
                _ = token.cancelled() => {
                    debug!("Listener on {ip} cancelled");
                    break;
                }
                res = listener.accept() => {
                    let (stream, peer) = match res {
                        Ok(v) => v,
                        Err(err) => {
                            match err.raw_os_error() {
                                Some(libc::ENODEV) | Some(libc::EBADF) | Some(libc::EINVAL) => {
                                    warn!("TcpServer listener for {ip}: device gone: {err:?}");
                                }
                                _ => {
                                    error!("TcpServer accept() failed on {ip}: {err:?}");
                                }
                            }

                            listener_err.replace(err);
                            break;
                        }
                    };

                    info!("Accepted connection from {peer} on {ip}");
                    let token = token.clone();
                    let factory = factory.clone();

                    inner.push(async move {
                        let session = factory(peer);
                        let stream_std = match stream.into_std() {
                            Ok(v) => v,
                            Err(err) => {
                                error!("Failed to initialize TCP conn: {err:?}");
                                return;
                            }
                        };

                        let bootstrap = match TcpBootstrap::external(stream_std) {
                            Ok(v) => v,
                            Err(err) => {
                                error!("Failed to initialize TCP conn: {err:?}");
                                return;
                            }
                        };

                        let mut task = TcpHelperTask::new(bootstrap, token, session);
                        task.run().await;
                    });
                }

                Some(()) = inner.next() => {}
            }
        }

        while let Some(()) = inner.next().await {}

        debug!("Listener {ip} finished");
        match listener_err {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}

impl<T> EventReconciler for TcpServerTask<T>
where
    T: TcpSession,
{
    type Error = ArcBox<io::Error>;

    async fn reconcile(&mut self) -> Result<(), Self::Error> {
        match self.listener_status.as_ref() {
            Ok(()) => Ok(()),
            Err(err) => Err(err.clone()),
        }
    }
}

impl<T> EventSleeper for TcpServerTask<T>
where
    T: TcpSession,
{
    async fn sleep(&mut self) -> Option<EventToken> {
        match self.listeners.next().await {
            Some(Ok(())) => Some(EventToken(1)),
            Some(Err(err)) => {
                self.listener_status = Err(err.into());
                Some(EventToken(1))
            }
            None => None,
        }
    }
}
