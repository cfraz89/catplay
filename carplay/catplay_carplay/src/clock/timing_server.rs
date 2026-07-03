use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Instant,
};

use catplay_tokio::{UdpSession, UdpSocketPeer};
use catplay_util::EventSleeper;
use log::debug;

use crate::{
    clock::{ClockMonotonic, MediaClockProxy, MediaClockSession, RtcpTimeSyncPacket},
    rtsp_frame::{RtspError, RtspResult},
};

pub struct TimingServer {
    clock: Arc<Mutex<MediaClockSession<ClockMonotonic>>>,
    last_received: Option<Instant>,
    start: Option<Instant>,
}

impl TimingServer {
    pub fn new() -> (Self, MediaClockProxy) {
        let clock = Arc::new(Mutex::new(MediaClockSession::<ClockMonotonic>::new()));

        let proxy = MediaClockProxy::new(clock.clone());
        let me = Self {
            clock,
            last_received: None,
            start: None,
        };
        (me, proxy)
    }

    fn respond(&self, clock: &mut MediaClockSession<ClockMonotonic>, data: &mut [u8]) -> Option<[u8; 32]> {
        let p = RtcpTimeSyncPacket::parse(data);
        let Some(p) = p else {
            debug!("Unparsable timesync packet received!");
            return None;
        };

        let resp = clock.respond(p);
        debug!("Got timesync request {:?}, responding with {:?}", p, resp);
        Some(resp.serialize())
    }
}

impl UdpSession for TimingServer {
    type Error = RtspError;

    fn on_datagram(&mut self, data: &mut [u8], _peer: SocketAddr, sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
        let mut clock = self.clock.lock().unwrap();

        if let Some(resp) = self.respond(&mut clock, data) {
            let _ = sink.send(&resp);
            self.last_received.replace(Instant::now());
        }

        Ok(())
    }

    fn on_connect(&mut self, peer: SocketAddr, _sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
        self.start.replace(Instant::now());
        debug!("Connected to {peer}");
        Ok(())
    }

    fn reconcile(&mut self, _peer: Option<&dyn UdpSocketPeer<Self>>) -> RtspResult<()> {
        Ok(())
    }
}

impl EventSleeper for TimingServer {}
