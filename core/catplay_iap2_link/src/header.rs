use catplay_util::{ModSeq, ModSeq8};

use crate::{LinkControl, PacketCoderError, iap2_check_checksum, iap2_gen_checksum};

pub const IAP2_HEADER_START: u16 = 0xFF5A;
pub const IAP2_HEADER_SIZE: usize = 9;

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub length: u16,
    pub control: LinkControl,
    pub seq: ModSeq8,
    pub ack: ModSeq8,
    pub session_id: u8,
}

impl Header {
    pub fn parse(data: &[u8]) -> Result<Self, PacketCoderError> {
        if data.len() < IAP2_HEADER_SIZE {
            return Err(PacketCoderError::HeaderNotEnoughBytes { received: data.len() });
        }
        let start = u16::from_be_bytes([data[0], data[1]]);
        if start != IAP2_HEADER_START {
            return Err(PacketCoderError::HeaderInvalidMagic);
        }
        if !iap2_check_checksum(&data[..9]) {
            let expected: u8 = iap2_gen_checksum(&data[..8]);
            let actual = data[8];
            return Err(PacketCoderError::HeaderChecksumInvalid { expected, actual });
        }

        Ok(Self {
            length: u16::from_be_bytes([data[2], data[3]]),
            control: LinkControl::from_bits_truncate(data[4]),
            seq: ModSeq(data[5]),
            ack: ModSeq(data[6]),
            session_id: data[7],
        })
    }

    pub fn to_bytes(&self) -> [u8; IAP2_HEADER_SIZE] {
        let mut buf = [0u8; IAP2_HEADER_SIZE];
        buf[0..2].copy_from_slice(&IAP2_HEADER_START.to_be_bytes());
        buf[2..4].copy_from_slice(&self.length.to_be_bytes());
        buf[4] = self.control.bits();
        buf[5] = self.seq.into();
        buf[6] = self.ack.into();
        buf[7] = self.session_id;
        buf[8] = iap2_gen_checksum(&buf[..8]);
        buf
    }
}

#[cfg(test)]
mod tests {
    use catplay_util::ModSeq;

    use crate::{Header, LinkControl, iap2_check_checksum, iap2_gen_checksum};

    #[test]
    fn test_gen_checksum_matches_e2() {
        let data: [u8; 9] = [0xFF, 0x5A, 0x00, 0x1A, 0x80, 0x2B, 0x00, 0x00, 0xE2];
        let checksum = iap2_gen_checksum(&data[..8]);

        assert_eq!(checksum, 0xE2, "Expected checksum to be 0xE2, got 0x{:02X}", checksum);
        assert!(iap2_check_checksum(&data));
    }

    #[test]
    fn test_link_packet_header_parse_and_pack() {
        let raw: [u8; 9] = [
            0xFF, 0x5A, // start
            0x00, 0x1A, // length (26)
            0x80, // control (SYN)
            0x2B, // seq
            0x00, // ack
            0x00, // session ID
            0xE2, // checksum
        ];

        let ret = Header::parse(&raw);
        assert!(ret.is_ok(), "Parsing failure: {:?}", ret);
        let parsed = ret.expect("Failed to parse header: nothing returned");
        assert_eq!(parsed.length, 0x001A);
        assert_eq!(parsed.control, LinkControl::from_bits_truncate(0x80));
        assert_eq!(parsed.seq, ModSeq(0x2B));
        assert_eq!(parsed.ack, ModSeq(0x00));
        assert_eq!(parsed.session_id, 0x00);

        let packed = parsed.to_bytes();
        assert_eq!(packed, raw);
    }
}
