use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use catplay_tokio::{UdpSession, UdpSocketPeer};
use catplay_util::{EventSleeper, EventToken, deadline, event_select, notify::Notify};
use log::{debug, trace, warn};

use crate::{
    clock::{ClockMonotonic, MediaClockProxy, MediaClockSession, RtcpTimeSyncPacket},
    rtsp_frame::{RtspError, RtspResult},
};

pub struct TimingClient {
    clock: Arc<Mutex<MediaClockSession<ClockMonotonic>>>,

    last_sent: Option<Instant>,
    last_response: Option<Instant>,
    sent_pings: u64,
    pending_send: bool,

    notify: Notify,
    start: Option<Instant>,
}

impl TimingClient {
    const TIMEOUT: Duration = Duration::from_millis(10000);
    const SYNC_INTERVAL: Duration = Duration::from_millis(1000);
    const SYNC_JITTER_RANGE: Duration = Duration::from_millis(50);
    const BURST_BATCH_SIZE: u64 = 5;

    const BURST_INTERVAL: Duration = Duration::from_millis(50);

    pub fn new() -> (Self, MediaClockProxy) {
        let clock = Arc::new(Mutex::new(MediaClockSession::<ClockMonotonic>::new()));

        let proxy = MediaClockProxy::new(clock.clone());
        let me = Self {
            clock,
            last_sent: None,
            last_response: None,
            sent_pings: 0,
            pending_send: false,

            notify: Notify::new(),
            start: None,
        };
        (me, proxy)
    }

    fn feed(&self, clock: &mut MediaClockSession<ClockMonotonic>, data: &[u8]) {
        let resp = RtcpTimeSyncPacket::parse(data);
        let Some(resp) = resp else {
            return;
        };

        clock.feed(resp);
        if clock.is_ready() {
            self.notify.notify();
        }

        trace!("Received timing response: {resp:?}");
    }

    pub fn sleep_random(&mut self) -> Duration {
        let frac = rand::random::<f64>();
        let interval = Self::SYNC_INTERVAL.as_secs_f64();
        let jitter_range = Self::SYNC_JITTER_RANGE.as_secs_f64();
        let jitter = (frac * jitter_range * 2.0) - jitter_range;
        let time = Duration::from_secs_f64(interval + jitter);
        trace!("Sleeping for {time:?}");
        time
    }

    pub fn send_single(&self, clock: &mut MediaClockSession<ClockMonotonic>, peer: &dyn UdpSocketPeer<Self>) {
        let req = clock.create_ping();
        if let Err(err) = peer.send(&req.serialize()) {
            warn!("Failed to send time sync: {err:?}");
        }
    }

    pub fn proxy(&self) -> MediaClockProxy {
        MediaClockProxy::new(self.clock.clone())
    }
}

impl UdpSession for TimingClient {
    type Error = RtspError;

    fn on_datagram(&mut self, data: &mut [u8], _peer: SocketAddr, _sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
        let mut clock = self.clock.lock().unwrap();
        self.feed(&mut clock, data);
        self.last_response.replace(Instant::now());
        Ok(())
    }

    fn on_connect(&mut self, peer: SocketAddr, _sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
        self.start.replace(Instant::now());
        debug!("Connected to {peer}");
        Ok(())
    }

    fn reconcile(&mut self, peer: Option<&dyn UdpSocketPeer<Self>>) -> RtspResult<()> {
        let mut clock = self.clock.lock().unwrap();

        let now = Instant::now();
        if let Some(last_response) = self.last_response
            && now - last_response > Self::TIMEOUT
        {
            return Err(RtspError::KeepAliveTimeout);
        }

        if let Some(start) = self.start
            && self.last_response.is_none()
            && now - start > Self::TIMEOUT
        {
            return Err(RtspError::KeepAliveTimeout);
        }

        if self.pending_send {
            self.pending_send = false;
            self.last_sent.replace(Instant::now());
            self.sent_pings += 1;

            if let Some(peer) = peer {
                self.send_single(&mut clock, peer);
            }
        }

        Ok(())
    }
}

impl EventSleeper for TimingClient {
    async fn sleep(&mut self) -> Option<EventToken> {
        self.start?;

        // Burst mode at startup to select the best RTT
        let next_ping = if self.sent_pings < Self::BURST_BATCH_SIZE {
            Self::BURST_INTERVAL
        } else {
            self.sleep_random()
        };

        let sleep_until_time = match self.last_sent {
            Some(prev) => prev + next_ping,
            None => Instant::now(),
        };

        event_select!(deadline(sleep_until_time));
        self.pending_send = true;
        Some(EventToken(1))
    }
}
