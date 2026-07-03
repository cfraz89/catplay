use alloc::vec::Vec;

use super::PayloadDecodable;
use crate::packet::Packet;

#[derive(Debug, Clone, PartialEq)]
pub struct EAKPayload {
    pub psns: Vec<u8>,
}

impl EAKPayload {
    pub fn new(psns: Vec<u8>) -> Self {
        Self { psns }
    }
}

impl PayloadDecodable for EAKPayload {
    fn from_packet(packet: &Packet) -> Option<Self> {
        if packet.header.control.is_eak()
            && let Some(payload) = &packet.payload
        {
            return Some(EAKPayload::new(payload.clone()));
        }

        None
    }

    fn to_bytes(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.psns);
    }
}
