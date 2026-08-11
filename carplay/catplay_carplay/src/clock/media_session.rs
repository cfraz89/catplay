use std::time::{Duration, Instant};

use crate::clock::{Clock, ClockMonotonic, MediaClock, MediaPll, NtpU64, RtcpTimeSyncPacket};
use log::{debug, trace};

/// Time sync packets carry this offset; frame and HID timestamps do not.
///
/// The two domains differing by 70 years looks like a bug and is not - it is what a real iPhone
/// does. Verified against a capture of one streaming to a CarPlay head unit: its `t3` read
/// `2209933588.87` while the PTS on the screen frames it sent 0.5s later read `944789.37`, exactly
/// `t3` minus this constant. A receiver learns the offset between the clocks from the sync
/// exchange - where the epoch cancels - and reads the PTS in the sender's bare monotonic domain.
///
/// So do not "fix" [`MediaClock::encode_local`] to add this. Adding it puts every frame 70 years
/// ahead of where a receiver looks for it.
pub const NTP_UNIX_EPOCH_OFFSET_NS: i128 = 2_208_988_800i128 * 1_000_000_000;

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

        Some(self.clock.decode_ns(pll.remote_to_local(pts.as_nanos())))
    }

    fn decode_local(&self, pts: NtpU64) -> Instant {
        // iPhone wants to decode "PTS" of a received HID event
        self.clock.decode_ns(pts.as_nanos())
    }

    fn encode_local(&self, pts: Instant) -> NtpU64 {
        // iPhone wants to encode PTS of a video frame
        let nanos = self.clock.encode_ns(pts);
        NtpU64::from_monotonic_nanos(nanos)
    }

    fn encode_remote(&self, pts: Instant) -> Option<NtpU64> {
        // HU wants to encode "PTS" of a HID event in iPhone-local time domain
        let nanos = self.pll.as_ref()?.local_to_remote(self.clock.encode_ns(pts));
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

    /// Pins the split documented on [`NTP_UNIX_EPOCH_OFFSET_NS`], measured off a real iPhone: a
    /// frame PTS sits exactly one epoch offset below the clock the receiver synchronizes to.
    #[test]
    fn frame_pts_sits_one_epoch_below_the_time_sync() {
        let session = MediaClockSession::<ClockMonotonic>::new();

        let sync = session.respond(RtcpTimeSyncPacket::build_request(NtpU64::ZERO));
        let pts = session.encode_local(Instant::now());

        // Compared through `diff_in_nanos` because `as_nanos` reads the seconds field back through
        // `i32`: an absolute NTP value wraps, and only a difference cancels it.
        let expected = NtpU64::from_monotonic_nanos(pts.as_nanos() + NTP_UNIX_EPOCH_OFFSET_NS);
        let skew = NtpU64::diff_in_nanos(sync.t3, expected);
        assert!(
            skew.abs() < Duration::from_secs(1).as_nanos() as i128,
            "PTS {pts:?} is not one epoch below sync {:?} (off by {skew} ns)",
            sync.t3
        );
    }
}
