use std::fmt::Debug;

use log::debug;

const PT_TIMESYNC_REQUEST: u8 = 0xD2;
const PT_TIMESYNC_RESPONSE: u8 = 0xD3;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct NtpU64(pub u64);

impl NtpU64 {
    pub const ZERO: NtpU64 = NtpU64(0);

    pub fn from_monotonic_nanos(nanos: i128) -> Self {
        let secs = nanos.div_euclid(1_000_000_000);
        let frac_nanos = nanos.rem_euclid(1_000_000_000);
        let frac = ((frac_nanos as u128) << 32) / 1_000_000_000u128;
        NtpU64(((secs as i64 as u64) << 32) | (frac as u64))
    }

    pub fn diff_in_nanos(a: NtpU64, b: NtpU64) -> i128 {
        a.as_nanos() - b.as_nanos()
    }

    pub fn as_nanos(&self) -> i128 {
        let secs = (self.0 >> 32) as i32 as i128;

        let frac = (self.0 & 0xFFFF_FFFF) as u128;
        let frac_nanos = ((frac * 1_000_000_000u128) >> 32) as i128;

        secs * 1_000_000_000 + frac_nanos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_roundtrip() {
        let vals = [0i128, 123_456_789i128, 999_999_999i128, 1_000_000_000i128, 12_345_678_901_234i128];

        for &ns in &vals {
            let ntp = NtpU64::from_monotonic_nanos(ns);
            let back = ntp.as_nanos();
            assert!((back - ns).abs() <= 1, "roundtrip mismatch: {} -> {} -> {}", ns, ntp.0, back);
        }
    }

    #[test]
    fn test_negative_roundtrip() {
        let vals = [
            -1i128,
            -123_456_789i128,
            -999_999_999i128,
            -1_000_000_000i128,
            -12_345_678_901_234i128,
        ];

        for &ns in &vals {
            let ntp = NtpU64::from_monotonic_nanos(ns);
            let back = ntp.as_nanos();
            assert!(
                (back - ns).abs() <= 1,
                "negative roundtrip mismatch: {} -> {} -> {}",
                ns,
                ntp.0,
                back
            );
        }
    }

    #[test]
    fn test_transition_across_second_boundary() {
        let before = -1_000_000_001i128;
        let after = -999_999_999i128;

        let ntp_before = NtpU64::from_monotonic_nanos(before);
        let ntp_after = NtpU64::from_monotonic_nanos(after);

        let rt_before = ntp_before.as_nanos();
        let rt_after = ntp_after.as_nanos();

        assert!(
            (rt_after - rt_before).abs() <= 10,
            "second-boundary jump incorrect: {} -> {} (delta = {})",
            rt_before,
            rt_after,
            (rt_after - rt_before)
        );
    }
}

impl Debug for NtpU64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NtpU64({} ns)", self.as_nanos())
    }
}
// ===== RTCP TimeSync (Apple) =====

// #[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct RtcpHeader {
    v_p_m: u8, // 2b version, 1b padding, 5b count/mode (unused)
    pt: u8,    // packet type
    #[allow(unused)]
    length: u16, // (len/4)-1
}

// #[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct RtcpTimeSyncPacket {
    header: RtcpHeader,
    /// ignored by Apple
    rtp_ts: u32,
    /// ntp_orig
    pub t1: NtpU64,
    /// ntp_recv
    pub t2: NtpU64,
    /// ntp_xmit
    pub t3: NtpU64,
}

impl RtcpTimeSyncPacket {
    pub fn rtt_ns(&self, t4: NtpU64) -> i128 {
        (NtpU64::diff_in_nanos(t4, self.t1) - NtpU64::diff_in_nanos(self.t3, self.t2)).max(0)
    }

    pub fn delta_srv_ns(&self) -> i128 {
        NtpU64::diff_in_nanos(self.t3, self.t2).max(0)
    }

    pub fn offset_ns(&self, t4: NtpU64) -> i128 {
        (NtpU64::diff_in_nanos(self.t2, self.t1) + NtpU64::diff_in_nanos(self.t3, t4)) / 2
    }

    pub fn offset_fallback_ns(&self) -> i128 {
        NtpU64::diff_in_nanos(self.t2, self.t1)
    }
}

impl RtcpTimeSyncPacket {
    const BYTES: usize = std::mem::size_of::<Self>();

    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < Self::BYTES {
            debug!("dropping timesync: too short at {}", buf.len());
            return None;
        }
        let v_p_m = buf[0];
        if (v_p_m >> 6) & 0b11 != 2 {
            debug!("dropping timesync: invalid v_p_m at {v_p_m}");
            return None;
        }
        let pt = buf[1];
        let length = u16::from_be_bytes([buf[2], buf[3]]);
        let expected = ((Self::BYTES / 4) - 1) as u16;
        if length != expected {
            debug!("dropping timesync: length mismatch {length} vs {expected}");
            return None;
        }

        let rtp_ts = u32::from_be_bytes(buf[4..8].try_into().unwrap());
        let ntp_orig = u64::from_be_bytes(buf[8..16].try_into().unwrap());
        let ntp_recv = u64::from_be_bytes(buf[16..24].try_into().unwrap());
        let ntp_xmit = u64::from_be_bytes(buf[24..32].try_into().unwrap());

        Some(Self {
            header: RtcpHeader { v_p_m, pt, length },
            rtp_ts,
            t1: NtpU64(ntp_orig),
            t2: NtpU64(ntp_recv),
            t3: NtpU64(ntp_xmit),
        })
    }

    pub fn serialize(&self) -> [u8; Self::BYTES] {
        let mut out = [0u8; Self::BYTES];
        let length_words_minus1 = ((Self::BYTES / 4) - 1) as u16;
        out[0] = self.header.v_p_m;
        out[1] = self.header.pt;
        out[2..4].copy_from_slice(&length_words_minus1.to_be_bytes());
        out[4..8].copy_from_slice(&self.rtp_ts.to_be_bytes());
        out[8..16].copy_from_slice(&self.t1.0.to_be_bytes());
        out[16..24].copy_from_slice(&self.t2.0.to_be_bytes());
        out[24..32].copy_from_slice(&self.t3.0.to_be_bytes());
        out
    }

    pub fn build_response(t1: NtpU64, t2: NtpU64, t3: NtpU64) -> RtcpTimeSyncPacket {
        // v_p_m: ver=2 (10xxxxxx), no padding, rest is 0
        let v_p_m = 2u8 << 6;
        let length_words_minus1 = ((RtcpTimeSyncPacket::BYTES / 4) - 1) as u16;

        RtcpTimeSyncPacket {
            header: RtcpHeader {
                v_p_m,
                pt: PT_TIMESYNC_RESPONSE,
                length: length_words_minus1.to_be(),
            },
            rtp_ts: 0,
            t1,
            t2,
            t3,
        }
    }

    pub fn build_request(t1: NtpU64) -> Self {
        let v_p_m = 2u8 << 6; // version=2

        RtcpTimeSyncPacket {
            header: RtcpHeader {
                v_p_m,
                pt: PT_TIMESYNC_REQUEST,
                length: 0,
            },
            rtp_ts: 0,
            t1: NtpU64::ZERO,
            t2: NtpU64::ZERO,
            t3: t1,
        }
    }
}
