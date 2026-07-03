use crate::decoder::{CsmDecode, CsmEncode, CsmParam, CsmReader, CsmScanner, CsmWriter};

/// A struct needs to implement this.
pub trait CsmParamDecode {
    fn feed_param(&mut self, param: &CsmParam);

    fn prealloc(&mut self, reader: &CsmScanner);
}

/// A struct needs to implement this.
pub trait CsmParamEncode {
    fn encode_to_params(&self, out: &mut CsmWriter);
}

/// Basic bytes->params implementation fitting the same shared CsmDecode interface.
/// So for a struct it decodes bytes into struct, for primitive it decodes TLV values into a primitive.
impl<T: CsmParamDecode + Default> CsmDecode for T {
    fn decode_from_bytes(data: &[u8]) -> Self {
        let mut ret = T::default();
        ret.prealloc(&CsmScanner::new(data));

        let mut callback = |b: CsmParam| ret.feed_param(&b);
        let mut reader = CsmReader::new(data, &mut callback);

        reader.stream_all();
        ret
    }
}

#[cfg(feature = "alloc")]
extern crate alloc;

pub trait CsmParamEncodeBytes {
    fn encode_to_bytes(&self, writer: &mut CsmWriter);

    fn measure(&self) -> usize {
        // Measurement pass
        CsmWriter::measure(|w| self.encode_to_bytes(w))
    }

    #[cfg(feature = "alloc")]
    fn serialize(&self) -> alloc::vec::Vec<u8> {
        CsmWriter::serialize(|w| self.encode_to_bytes(w))
    }
}

/// Serializes main structs into bytes.
impl<T: CsmParamEncode> CsmParamEncodeBytes for T {
    fn encode_to_bytes(&self, writer: &mut CsmWriter) {
        self.encode_to_params(writer);
    }
}

/// Serializes embedded structs into TLV params.
impl<T: CsmParamEncode> CsmEncode for T {
    fn encode_param(&self, id: u16, writer: &mut CsmWriter) {
        let len = self.measure();

        writer.write_tlv_header(id, len);
        self.encode_to_bytes(writer);
    }
}
