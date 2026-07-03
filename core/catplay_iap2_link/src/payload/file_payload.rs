use crate::{Packet, PayloadDecodable};
use alloc::vec::Vec;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileTransferOp {
    Start = 0x01,
    Cancel = 0x02,
    Pause = 0x03,
    Setup = 0x04,
    Success = 0x05,
    Failure = 0x06,

    FirstData = 0x80,
    FirstAndOnlyData = 0xC0,
    Data = 0x00,
    LastData = 0x40,
}

impl TryFrom<u8> for FileTransferOp {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let ret = match value {
            x if x == Self::Start as _ => Self::Start,
            x if x == Self::Cancel as _ => Self::Cancel,
            x if x == Self::Pause as _ => Self::Pause,
            x if x == Self::Setup as _ => Self::Setup,
            x if x == Self::Success as _ => Self::Success,
            x if x == Self::Failure as _ => Self::Failure,

            x if x == Self::FirstData as _ => Self::FirstData,
            x if x == Self::FirstAndOnlyData as _ => Self::FirstAndOnlyData,
            x if x == Self::Data as _ => Self::Data,
            x if x == Self::LastData as _ => Self::LastData,

            _ => return Err(()),
        };

        Ok(ret)
    }
}

impl From<FileTransferOp> for u8 {
    fn from(val: FileTransferOp) -> Self {
        val as u8
    }
}

impl PayloadDecodable for FileTransferPayload {
    fn from_packet(packet: &Packet) -> Option<Self> {
        let payload = packet.payload.as_ref()?;
        if payload.len() < 2 {
            return None;
        }

        let file_id = payload[0];
        let op = payload[1].try_into().ok()?;

        Some(FileTransferPayload {
            file_id,
            op,
            payload: payload[2..].into(),
        })
    }

    fn to_bytes(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&[self.file_id, self.op.into()]);
        out.extend_from_slice(&self.payload);
    }
}

#[derive(Debug, Clone)]
pub struct FileTransferPayload {
    pub file_id: u8,
    pub op: FileTransferOp,
    pub payload: Vec<u8>,
}

impl FileTransferPayload {
    pub const HEADER_OVERHEAD: usize = 2;
}
