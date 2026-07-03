use super::data::*;

// Primitive decoders

impl CsmDecode for bool {
    fn decode_from_bytes(data: &[u8]) -> Self {
        data.first().copied().unwrap_or(0) == 1
    }
}

impl CsmDecode for u8 {
    fn decode_from_bytes(data: &[u8]) -> Self {
        data.first().copied().unwrap_or(0)
    }
}

impl CsmDecode for u16 {
    fn decode_from_bytes(data: &[u8]) -> Self {
        if data.len() < 2 {
            return 0u16;
        }

        u16::from_be_bytes([data[0], data[1]])
    }
}

impl CsmDecode for u32 {
    fn decode_from_bytes(data: &[u8]) -> Self {
        if data.len() < 4 {
            return 0u32;
        }

        u32::from_be_bytes([data[0], data[1], data[2], data[3]])
    }
}

impl CsmDecode for u64 {
    fn decode_from_bytes(data: &[u8]) -> Self {
        if data.len() < 8 {
            return 0u64;
        }

        u64::from_be_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]])
    }
}

impl<const N: usize> CsmDecode for [u8; N] {
    fn decode_from_bytes(data: &[u8]) -> Self {
        let mut out = [0u8; N];
        let len = data.len().min(N);

        // cut extra bytes if too long, zero-fill if too short
        out[..len].copy_from_slice(&data[..len]);
        out
    }
}

// Special case - CSM flag with no payload
impl CsmDecode for CsmFlag {
    fn decode_from_bytes(_data: &[u8]) -> Self {
        CsmFlag::Yes
    }
}

// Special case: Option<T>
impl<T: CsmDecode> CsmDecode for Option<T> {
    fn decode_from_bytes(data: &[u8]) -> Self {
        Some(T::decode_from_bytes(data))
    }
}
