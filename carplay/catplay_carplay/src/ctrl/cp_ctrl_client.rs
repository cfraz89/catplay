use std::{
    io,
    time::{Duration, Instant},
};

use catplay_bonjour::BonjourEntry;
use log::{debug, trace};
use macaddr::MacAddr6;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::timeout_at,
};

use crate::ctrl::{AirPlayMacId, CarPlayCtrlBonjourEntry};

pub struct CarPlayCtrlClient {
    entry: BonjourEntry<CarPlayCtrlBonjourEntry>,
}

impl CarPlayCtrlClient {
    pub fn new(entry: &BonjourEntry<CarPlayCtrlBonjourEntry>) -> Self {
        Self { entry: entry.clone() }
    }

    /// Invites iPhone to connect to this CarPlay/AirPlay server.
    ///
    /// All requests result in 200 OK, no matter if iPhone makes the decision to accept the invite
    /// or ignore it, and no matter if the provided MAC address is valid.
    ///
    /// Sometimes, the requests may get throttled, time out, or take a longer time to complete.
    pub async fn connect(&self, server_mac_addr: &MacAddr6, timeout: Duration) -> Result<(), io::Error> {
        let server_id = AirPlayMacId::from(*server_mac_addr);

        let addrs = &self.entry.meta.addrs;
        let port = self.entry.meta.port;
        debug!("Pinging carplay-ctrl at {addrs:?} to {server_mac_addr}");

        let deadline = Instant::now() + timeout;

        let mut stream = timeout_at(deadline.into(), TcpStream::connect(&addrs[..]))
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "connect timeout"))??;

        let request = format!(
            "GET /ctrl-int/1/connect HTTP/1.1\r\n\
            Host: carplay:{port}\r\n\
            AirPlay-Receiver-Device-ID: {server_id}\r\n\
            User-Agent: carplay-ctrl-client/0.1\r\n\
            Connection: close\r\n\
            \r\n"
        );

        trace!("carplay-ctrl > {}", request.replace("\r\n", "\n> "));

        timeout_at(deadline.into(), async move {
            stream.write_all(request.as_bytes()).await?;

            let mut buf = Vec::new();
            stream.read_to_end(&mut buf).await?;

            let formatted = match String::from_utf8(buf) {
                Ok(text) => text,
                Err(e) => format!("{:?}", e.into_bytes()),
            };

            trace!("carplay-ctrl < {formatted}");
            Ok(()) as Result<(), io::Error>
        })
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "response timeout"))??;

        Ok(())
    }
}
