use std::{
    io::{self},
    net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
    time::Duration,
};

use tokio::{
    net::{TcpListener as TokioTcpListener, TcpSocket as TokioTcpSocket, TcpStream as TokioTcpStream},
    time::timeout,
};

pub enum TcpBootstrap {
    Connect(TokioTcpSocket, SocketAddr, Duration),
    AcceptOnce(TokioTcpListener, SocketAddr, Duration),
    PreConnected(TokioTcpStream),
}

impl TcpBootstrap {
    pub async fn resolve(self) -> io::Result<TokioTcpStream> {
        let ret = match self {
            TcpBootstrap::Connect(sock, ip, t) => timeout(t, async move { sock.connect(ip).await })
                .await
                .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, format!("timed out connecting to {ip} in {t:?}")))??,
            TcpBootstrap::AcceptOnce(sock, ip, t) => {
                timeout(t, async move { sock.accept().await })
                    .await
                    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, format!("timed out connecting to {ip} in {t:?}")))??
                    .0
            }
            TcpBootstrap::PreConnected(sock) => sock,
        };
        Ok(ret)
    }

    pub fn connect(ip: impl ToSocketAddrs, timeout: Duration) -> io::Result<Self> {
        // TODO: try all IPs
        let ip = ip.to_socket_addrs()?.next().unwrap();
        let socket = if ip.is_ipv6() {
            TokioTcpSocket::new_v6()?
        } else {
            TokioTcpSocket::new_v4()?
        };

        let timeout = match timeout {
            Duration::ZERO => Duration::MAX,
            _ => timeout,
        };

        Ok(TcpBootstrap::Connect(socket, ip, timeout))
    }

    pub fn accept_once(bind_ip: impl ToSocketAddrs, timeout: Duration) -> io::Result<(Self, SocketAddr)> {
        let listener = TcpListener::bind(bind_ip)?;
        listener.set_nonblocking(true)?;

        let listener = TokioTcpListener::from_std(listener)?;
        let local_addr = listener.local_addr()?;

        let timeout = match timeout {
            Duration::ZERO => Duration::MAX,
            _ => timeout,
        };

        Ok((TcpBootstrap::AcceptOnce(listener, local_addr, timeout), local_addr))
    }

    pub fn external(stream: TcpStream) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        let stream = stream.try_into()?;

        Ok(TcpBootstrap::PreConnected(stream))
    }
}
