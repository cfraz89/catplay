use crate::decoder::{CsmParam, CsmWriter};

/// A "flag" type with no payload, which could either exist in the message or not, but will never have any payload.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum CsmFlag {
    #[default]
    No,
    Yes,
}

pub trait CsmEncode {
    fn encode_param(&self, id: u16, out: &mut CsmWriter);
}

pub trait CsmDecode: Sized {
    fn decode_from_bytes(data: &[u8]) -> Self;
}

pub trait CsmAccum {
    fn add_param(&mut self, param: &CsmParam);

    #[allow(unused)]
    fn prealloc(&mut self, size: usize) {}
}

impl<T: CsmDecode> CsmAccum for T {
    fn add_param(&mut self, param: &CsmParam) {
        *self = T::decode_from_bytes(param.value);
    }
}
