use crate::decoder::CsmWriter;

use super::data::*;

// Primitive encoders

impl CsmEncode for bool {
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        let b = match self {
            true => 1,
            false => 0,
        };

        out.write_tlv(id, &[b]);
    }
}

impl CsmEncode for u8 {
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        out.write_tlv(id, &[*self]);
    }
}

impl CsmEncode for u16 {
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        out.write_tlv(id, &self.to_be_bytes());
    }
}

impl CsmEncode for u32 {
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        out.write_tlv(id, &self.to_be_bytes());
    }
}

impl CsmEncode for u64 {
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        out.write_tlv(id, &self.to_be_bytes());
    }
}

impl<const N: usize> CsmEncode for [u8; N] {
    #[inline]
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        out.write_tlv(id, self);
    }
}

impl CsmEncode for &[u8] {
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        out.write_tlv(id, self);
    }
}

impl CsmEncode for &str {
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        let bytes = self.as_bytes();
        out.write_tlv_header(id, bytes.len() + 1);
        out.write_data_chunk(bytes);
        out.write_data_chunk(&[0]);
    }
}

// Special case - CSM flag with no payload
impl CsmEncode for CsmFlag {
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        if *self == CsmFlag::Yes {
            out.write_tlv(id, &[]);
        }
    }
}

// Special case - Option<T>
impl<T: CsmEncode> CsmEncode for Option<T> {
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        if let Some(v) = self {
            v.encode_param(id, out);
        }
    }
}

// Special case - Vec<T> - multiple params of the same id
pub fn encode_repeating_params<T: CsmEncode>(id: u16, out: &mut CsmWriter, data: &[T]) {
    for item in data {
        item.encode_param(id, out);
    }
}
