use crate::packet::Packet;
use alloc::vec::Vec;

use super::PayloadDecodable;

#[derive(Debug, Clone, PartialEq)]
pub struct SessionPayload {
    pub data: Vec<u8>,
}

impl SessionPayload {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl PayloadDecodable for SessionPayload {
    fn from_packet(packet: &Packet) -> Option<Self> {
        if packet.header.control.is_ack()
            && let Some(payload) = &packet.payload
        {
            return Some(SessionPayload::new(payload.clone()));
        }

        None
    }

    fn to_bytes(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.data);
    }
}
