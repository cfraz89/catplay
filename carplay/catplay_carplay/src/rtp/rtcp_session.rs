use catplay_tokio::{UdpSession, UdpSocketPeer};
use catplay_util::EventSleeper;
use log::debug;

use crate::rtsp_frame::RtspError;

#[derive(Default)]
pub struct RtcpSession {}

impl UdpSession for RtcpSession {
    type Error = RtspError;

    fn on_datagram(&mut self, data: &mut [u8], _peer: std::net::SocketAddr, _sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
        debug!("Unsupported RTCP payload: {data:?}");

        Ok(())
    }
}

impl EventSleeper for RtcpSession {}
