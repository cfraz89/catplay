use alloc::vec::Vec;

use crate::packet::Packet;

pub trait PayloadDecodable: Sized {
    fn from_packet(packet: &Packet) -> Option<Self>;
    fn to_bytes(&self, out: &mut Vec<u8>);
}
