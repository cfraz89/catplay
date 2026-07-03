use super::RtpHeader;

use bytes::BytesMut;
use catplay_tokio::calc_buffer_pad;

pub const RTP_PACKET_MAX: usize = 1500;
pub const RTP_HEADER_SIZE: usize = RtpHeader::len();
pub const RTP_PACKET_ALIGN_OFFSET: usize = RtpHeader::len();
pub const RTP_BUFFER_PAD: usize = calc_buffer_pad(RTP_PACKET_ALIGN_OFFSET);

pub trait AsRtpPacket {
    fn header(&self) -> &RtpHeader;

    fn payload(&self) -> &[u8];

    fn payload_mut(&mut self) -> &mut [u8];

    fn split(&self) -> (&RtpHeader, &[u8]) {
        (self.header(), self.payload())
    }

    fn to_owned_packet(&self) -> RtpPacket {
        RtpPacket::new(*self.header(), self.payload())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RtpPacketBorrow<'a> {
    header: RtpHeader,
    payload: &'a mut [u8],
}

impl<'a> AsRtpPacket for RtpPacketBorrow<'a> {
    fn header(&self) -> &RtpHeader {
        &self.header
    }

    fn payload(&self) -> &[u8] {
        self.payload
    }

    fn payload_mut(&mut self) -> &mut [u8] {
        self.payload
    }
}

impl<'a> RtpPacketBorrow<'a> {
    pub fn new(header: RtpHeader, payload: &'a mut [u8]) -> Self {
        Self { header, payload }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RtpPacket {
    header: RtpHeader,
    payload: BytesMut,
}

impl AsRtpPacket for RtpPacket {
    fn header(&self) -> &RtpHeader {
        &self.header
    }

    fn payload(&self) -> &[u8] {
        &self.payload
    }

    fn payload_mut(&mut self) -> &mut [u8] {
        &mut self.payload
    }

    fn to_owned_packet(&self) -> RtpPacket {
        self.clone()
    }
}

impl RtpPacket {
    pub fn new(header: RtpHeader, payload: impl Into<BytesMut>) -> Self {
        Self {
            header,
            payload: payload.into(),
        }
    }
}
