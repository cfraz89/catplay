#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct RtpHeaderRaw {
    b0: u8,
    b1: u8,
    sequence_number: [u8; 2],
    timestamp: [u8; 4],
    ssrc: [u8; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtpHeader {
    pub version: u8,
    pub padding: bool,
    pub extension: bool,
    pub csrc_count: u8,
    pub marker: bool,
    pub payload_type: u8,
    pub sequence_number: u16,
    pub timestamp: u32,
    pub ssrc: u32,
}

const RTP_HEADER_LEN: usize = 12;
const _: [(); RTP_HEADER_LEN] = [(); core::mem::size_of::<RtpHeaderRaw>()];

#[inline(always)]
fn parse_rtp_header(buf: &[u8]) -> Option<RtpHeader> {
    if buf.len() < RTP_HEADER_LEN {
        return None;
    }

    Some(RtpHeader {
        version: (buf[0] >> 6) & 0b11,
        padding: ((buf[0] >> 5) & 0b1) != 0,
        extension: ((buf[0] >> 4) & 0b1) != 0,
        csrc_count: buf[0] & 0b1111,
        marker: (buf[1] >> 7) != 0,
        payload_type: buf[1] & 0x7F,
        sequence_number: u16::from_be_bytes(buf[2..4].try_into().unwrap()),
        timestamp: u32::from_be_bytes(buf[4..8].try_into().unwrap()),
        ssrc: u32::from_be_bytes(buf[8..12].try_into().unwrap()),
    })
}

#[inline(always)]
fn write_rtp_header(header: &RtpHeader) -> [u8; RTP_HEADER_LEN] {
    let mut out = [0u8; RTP_HEADER_LEN];

    out[0] = (header.version & 0b11) << 6 | ((header.padding as u8) << 5) | ((header.extension as u8) << 4) | (header.csrc_count & 0x0F);

    out[1] = ((header.marker as u8) << 7) | (header.payload_type & 0x7F);
    out[2..4].copy_from_slice(&header.sequence_number.to_be_bytes());
    out[4..8].copy_from_slice(&header.timestamp.to_be_bytes());
    out[8..12].copy_from_slice(&header.ssrc.to_be_bytes());

    out
}

impl RtpHeader {
    #[inline(always)]
    pub fn from_buf(buf: &[u8]) -> Option<RtpHeader> {
        parse_rtp_header(buf)
    }

    #[inline(always)]
    pub fn to_buf(&self) -> [u8; RTP_HEADER_LEN] {
        write_rtp_header(self)
    }

    #[inline(always)]
    pub const fn len() -> usize {
        RTP_HEADER_LEN
    }
}

impl From<RtpHeader> for [u8; RTP_HEADER_LEN] {
    fn from(value: RtpHeader) -> Self {
        value.to_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::RtpHeader;

    #[test]
    fn test_decode() {
        let packet: [u8; 12] = [
            0x80, 0xe0, // V=2, P=0, X=0, CC=0, M=1, PT=96
            0x12, 0x34, // Sequence Number
            0x00, 0x00, 0x00, 0x01, // Timestamp
            0xde, 0xad, 0xbe, 0xef, // SSRC
        ];
        let expected = RtpHeader {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: true,
            payload_type: 96,
            sequence_number: 4660,
            timestamp: 1,
            ssrc: 3735928559,
        };
        let decoded = RtpHeader::from_buf(&packet);
        assert_eq!(decoded, Some(expected));
    }

    #[test]
    fn test_encode() {
        let packet: [u8; 12] = [
            0x80, 0xe0, // V=2, P=0, X=0, CC=0, M=1, PT=96
            0x12, 0x34, // Sequence Number
            0x00, 0x00, 0x00, 0x01, // Timestamp
            0xde, 0xad, 0xbe, 0xef, // SSRC
        ];
        let expected = RtpHeader {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: true,
            payload_type: 96,
            sequence_number: 4660,
            timestamp: 1,
            ssrc: 3735928559,
        };
        let encoded = expected.to_buf();
        assert_eq!(encoded, packet);
    }
}
