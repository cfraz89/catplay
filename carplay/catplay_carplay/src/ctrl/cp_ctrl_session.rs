use async_trait::async_trait;
use catplay_tokio::{CItem, TcpSession, TcpSink};
use catplay_util::{AsyncShutdown, EventSleeper, mpsc};
use log::debug;

use crate::{
    ctrl::{AirPlayMacId, CarPlayCtrlInvite},
    rtsp_frame::{HttpHeader, HttpStatus, RtspError, RtspResponse, RtspResult},
    rtsp_transport::{RtspFrame, RtspFrameCodec},
};

pub struct CarPlayCtrlSession {
    invite_tx: mpsc::UnboundedSender<CarPlayCtrlInvite>,
}

impl CarPlayCtrlSession {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<CarPlayCtrlInvite>) {
        let (invite_tx, invite_rx) = mpsc::unbounded();
        (Self { invite_tx }, invite_rx)
    }

    pub fn with_invite_sender(invite_tx: mpsc::UnboundedSender<CarPlayCtrlInvite>) -> Self {
        Self { invite_tx }
    }
}

#[async_trait]
impl TcpSession for CarPlayCtrlSession {
    type Codec = RtspFrameCodec;
    type Error = RtspError;

    fn init_codec(&mut self) -> RtspResult<RtspFrameCodec> {
        Ok(RtspFrameCodec::default())
    }

    async fn on_msg(&mut self, _sink: &mut dyn TcpSink<Self>, msg: CItem<Self::Codec>) -> RtspResult<()> {
        const PATH: &str = "/ctrl-int/1/connect";
        const HTTP_1_1: &str = "HTTP/1.1";

        debug!("carplay-ctrl frame: {msg:?}");

        let RtspFrame::Request(msg) = msg else {
            return Err(RtspError::ProtocolViolationGeneric);
        };

        let mut resp = RtspResponse::new(None, HttpStatus::NotFound);
        resp.proto = HTTP_1_1.into();

        if msg.url == PATH
            && let Some(header) = msg.get_header(&HttpHeader::AirPlayReceiverDeviceID)
        {
            if let Ok(decoded) = AirPlayMacId::try_from(header) {
                debug!("Received invite to CarPlay server on mac {decoded}");
                let invite = CarPlayCtrlInvite::new(decoded.into());
                let _ = self.invite_tx.unbounded_send(invite);
            }

            resp.status = HttpStatus::Ok
        }

        if resp.status != HttpStatus::Ok {
            debug!("Rejected invalid CarPlay-Ctrl request!");
        }

        _sink.write(resp.into())?;
        Err(RtspError::DisconnectNow)
    }
}

impl EventSleeper for CarPlayCtrlSession {}
impl AsyncShutdown for CarPlayCtrlSession {}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use catplay_tokio::TcpHelper;
    use catplay_tracing::logger::setup_test_logger;
    use futures::StreamExt;
    use log::debug;
    use macaddr::MacAddr6;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
    };

    use crate::ctrl::{CarPlayCtrlInvite, CarPlayCtrlSession};

    #[tokio::test]
    async fn test_invite_receiver() {
        setup_test_logger(false);
        debug!("Starting test");

        let (responder, mut invite_rx) = CarPlayCtrlSession::new();

        let (_fut, local) = TcpHelper::accept_timeout(("127.0.0.1", 0), Duration::from_secs(1), responder).unwrap();
        let req = "GET /ctrl-int/1/connect HTTP/1.1\r
Host: 127.0.0.1:6000\r
User-Agent: curl/8.11.1\r
Accept: */*\r
AirPlay-Receiver-Device-ID: 963347909223\r
\r\n\r\n";

        let mut client = TcpStream::connect(local).await.unwrap();
        let _ = client.write(req.as_bytes()).await.unwrap();

        let mut resp = [0u8; 1024];
        let n = client.read(&mut resp).await.unwrap();
        let resp_str = String::from_utf8_lossy(&resp[..n]);

        let mac = invite_rx.next().await.unwrap();

        assert_eq!("HTTP/1.1 200 OK\r\n\r\n", resp_str);
        assert_eq!(mac, CarPlayCtrlInvite::new(MacAddr6::new(0x00, 0xE0, 0x4C, 0x02, 0x8A, 0x67)));
    }
}
