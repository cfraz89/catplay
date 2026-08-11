use std::time::{Duration, Instant};

use crate::clock::{Clock, ClockMonotonic, MediaClock, MediaPll, NtpU64, RtcpTimeSyncPacket};
use log::{debug, trace};

/// Every timestamp that leaves this session - time sync packets and the PTS of a frame or a HID
/// event alike - is NTP-based: session-elapsed nanoseconds plus this offset. The inner [`Clock`]
/// is monotonic and session-relative, so the [`MediaClock`] methods are the boundary that adds it
/// on the way out and removes it on the way in. Mixing the two domains puts a receiver that
/// honours timestamps 70 years out.
pub const NTP_UNIX_EPOCH_OFFSET_NS: i128 = 2_208_988_800i128 * 1_000_000_000;

/// The NTP epoch as [`NtpU64::as_nanos`] reports it, which is not `NTP_UNIX_EPOCH_OFFSET_NS`:
/// `as_nanos` reads the seconds field back through `i32`, so anything past 2038 wraps. Every
/// absolute timestamp wraps by the same amount, so subtracting this - a difference, which is how
/// `as_nanos` is used everywhere else - recovers the monotonic domain exactly.
fn ntp_epoch_nanos() -> i128 {
    NtpU64::from_monotonic_nanos(NTP_UNIX_EPOCH_OFFSET_NS).as_nanos()
}

pub struct MediaClockSession<C: Clock> {
    clock: C,
    pll: Option<MediaPll>,
    last_t3: Option<NtpU64>,

    // Burst/initialization phase
    initialized: bool,
    syncs: u64,
    best_rtt: i128,
    best_offset: i128,
}

impl<C: Clock> MediaClockSession<C> {
    pub fn new_with_clock(clock: C) -> Self {
        Self {
            clock,
            last_t3: None,
            initialized: false,
            syncs: 0,
            best_rtt: i128::MAX,
            best_offset: 0,
            pll: None,
        }
    }
}

impl Default for MediaClockSession<ClockMonotonic> {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaClockSession<ClockMonotonic> {
    pub fn new() -> Self {
        Self::new_with_clock(ClockMonotonic::new())
    }
}

impl<C: Clock> MediaClock for MediaClockSession<C> {
    fn decode_remote(&self, pts: NtpU64) -> Option<Instant> {
        let Some(ref pll) = self.pll else {
            // Either iPhone wants to perform a decode, and it's not possible - we don't know/track time domain of HU
            // or HU wants to perform a decode, but clock is not initialized at this stage (and caller should fallback to assuming `pts = Instant::now()`)
            return None;
        };

        if !self.initialized {
            return None;
        }

        let nanos = pll.remote_to_local(pts.as_nanos()) - ntp_epoch_nanos();
        Some(self.clock.decode_ns(nanos))
    }

    fn decode_local(&self, pts: NtpU64) -> Instant {
        // iPhone wants to decode "PTS" of a received HID event
        self.clock.decode_ns(pts.as_nanos() - ntp_epoch_nanos())
    }

    fn encode_local(&self, pts: Instant) -> NtpU64 {
        // iPhone wants to encode PTS of a video frame
        let nanos = self.clock.encode_ns(pts) + NTP_UNIX_EPOCH_OFFSET_NS;
        NtpU64::from_monotonic_nanos(nanos)
    }

    fn encode_remote(&self, pts: Instant) -> Option<NtpU64> {
        // HU wants to encode "PTS" of a HID event in iPhone-local time domain
        let local = self.clock.encode_ns(pts) + NTP_UNIX_EPOCH_OFFSET_NS;
        let nanos = self.pll.as_ref()?.local_to_remote(local);
        Some(NtpU64::from_monotonic_nanos(nanos))
    }

    fn is_synchronized(&self) -> bool {
        self.initialized
    }
}

impl<C: Clock> MediaClockSession<C> {
    /// As an iPhone, generate response to time sync request.
    pub fn respond(&self, ping: RtcpTimeSyncPacket) -> RtcpTimeSyncPacket {
        let now = NtpU64::from_monotonic_nanos(self.clock.elapsed_ns() + NTP_UNIX_EPOCH_OFFSET_NS);
        RtcpTimeSyncPacket::build_response(ping.t3, now, now)
    }

    /// As a HU, generate ping packet that will be send by the caller.
    pub fn create_ping(&mut self) -> RtcpTimeSyncPacket {
        let t1 = NtpU64::from_monotonic_nanos(self.clock.elapsed_ns() + NTP_UNIX_EPOCH_OFFSET_NS);
        self.last_t3.replace(t1);
        let req = RtcpTimeSyncPacket::build_request(t1);
        trace!("Creating RTCP ping from media clock: {:?}", req);
        req
    }

    /// As a HU, handle response that came after sending packet from `create_sync`.
    pub fn feed(&mut self, resp: RtcpTimeSyncPacket) {
        const RESET_THRESHOLD: Duration = Duration::from_millis(100);
        const SYNCS_TO_INITIALIZE: u64 = 3;

        let now_ns = self.clock.elapsed_ns() + NTP_UNIX_EPOCH_OFFSET_NS;
        let t4 = NtpU64::from_monotonic_nanos(now_ns);
        let offset = resp.offset_ns(t4);
        let rtt = resp.rtt_ns(t4);

        debug!("Feeding RTCP response to media clock: {:?} /t4 {t4:?}", resp);

        if !self.initialized {
            self.syncs += 1;

            if rtt < self.best_rtt {
                self.best_rtt = rtt;
                self.best_offset = offset;
                debug!("Collected offset probe {}/{} before clock init", self.syncs, SYNCS_TO_INITIALIZE);
            }

            if self.syncs >= SYNCS_TO_INITIALIZE {
                self.pll.replace(MediaPll::new(now_ns, self.best_offset));
                debug!(
                    "Initialized media clock: best_rtt={}, best_offset={}, now_ns={}",
                    self.best_rtt, self.best_offset, now_ns
                );

                self.initialized = true;
            }

            return;
        }

        if self.last_t3 != Some(resp.t1) {
            debug!("Ignored out-of-order NTP packet {:?} vs {:?}", resp.t1, self.last_t3);
            return;
        }

        if let Some(ref mut pll) = self.pll {
            // TODO improve tracking of delta_offset, track rolling average etc.
            let delta_offset = offset - pll.offset_ns;
            if delta_offset.abs() > RESET_THRESHOLD.as_nanos() as _ {
                self.pll.replace(MediaPll::new(now_ns, offset));
                debug!("Re-synchronized NTP time base with offset {}", offset);
                return;
            }

            pll.update(now_ns, delta_offset);
        }
    }

    /// Check whether clock was initialized by sufficient number of probe responses in burst mode.
    pub fn is_ready(&self) -> bool {
        self.initialized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::RtcpTimeSyncPacket;

    /// A frame PTS and the time sync a receiver synchronizes against have to be the same clock:
    /// the receiver schedules the first against the second. This was 70 years apart - `respond`
    /// carried the NTP epoch offset and `encode_local` did not - and a head unit that honours
    /// timestamps showed nothing at all while ACKing every frame.
    #[test]
    fn frame_pts_and_time_sync_share_a_domain() {
        let session = MediaClockSession::<ClockMonotonic>::new();

        let sync = session.respond(RtcpTimeSyncPacket::build_request(NtpU64::ZERO));
        let pts = session.encode_local(Instant::now() + Duration::from_millis(100));

        let skew = (pts.as_nanos() - sync.t3.as_nanos()).abs();
        assert!(
            skew < Duration::from_secs(1).as_nanos() as i128,
            "frame PTS {pts:?} is {skew} ns from the clock the receiver syncs to ({:?})",
            sync.t3
        );
    }

    /// Decoding is the inverse of encoding, so a timestamp we produced survives the round trip
    /// rather than landing an epoch away.
    #[test]
    fn local_timestamps_round_trip() {
        let session = MediaClockSession::<ClockMonotonic>::new();

        let now = Instant::now();
        let decoded = session.decode_local(session.encode_local(now));

        assert!(decoded.saturating_duration_since(now) < Duration::from_millis(1));
        assert!(now.saturating_duration_since(decoded) < Duration::from_millis(1));
    }
}
