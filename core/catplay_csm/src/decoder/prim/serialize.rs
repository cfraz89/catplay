#[derive(PartialEq, Eq, Debug)]
pub enum CsmError {
    Overflow,
    Underflow,

    PacketMagic,
    PacketUnderflow,
    PacketOverflow,
    PacketUnknown,
}

pub type CsmResult<T> = Result<T, CsmError>;

#[derive(Debug)]
pub struct CsmParam<'a> {
    pub id: u16,
    pub value: &'a [u8],
}

impl<'a> CsmParam<'a> {
    pub fn new(id: u16, value: &'a [u8]) -> Self {
        Self { id, value }
    }
}

pub struct CsmReader<'a> {
    data: &'a [u8],
    callback: &'a mut dyn FnMut(CsmParam<'a>),
    underflow: bool,
    overflow: bool,
}

impl<'a> CsmReader<'a> {
    pub fn new(data: &'a [u8], callback: &'a mut dyn FnMut(CsmParam<'a>)) -> Self {
        Self {
            data,
            callback,
            overflow: false,
            underflow: false,
        }
    }

    pub fn stream_all(&mut self) {
        let mut data = self.data;
        while data.len() >= 4 {
            let len = u16::from_be_bytes([data[0], data[1]]) as usize;
            let id = u16::from_be_bytes([data[2], data[3]]);
            if data.len() < len || len < 4 {
                self.overflow = true;
                break;
            }

            (self.callback)(CsmParam::new(id, &data[4..len]));

            data = &data[len..];
        }

        self.underflow = !data.is_empty();
    }

    pub fn take_error(&mut self) -> Option<CsmError> {
        if self.underflow {
            return Some(CsmError::Underflow);
        } else if self.overflow {
            return Some(CsmError::Overflow);
        }

        None
    }
}

pub struct CsmScanner<'a> {
    data: &'a [u8],
}

impl<'a> CsmScanner<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn count_repeating(&self, id: u16) -> usize {
        let mut count = 0;
        let mut data = self.data;

        while data.len() >= 4 {
            let len = u16::from_be_bytes([data[0], data[1]]) as usize;
            let read_id = u16::from_be_bytes([data[2], data[3]]);
            if data.len() < len || len < 4 {
                break;
            }

            if read_id == id {
                count += 1;
            }

            data = &data[len..];
        }

        count
    }
}

pub struct CsmWriter<'a> {
    callback: &'a mut dyn FnMut(&[u8]),
    pub written: usize,
    pub overflow: bool,
}

impl<'a> CsmWriter<'a> {
    pub fn new(callback: &'a mut dyn FnMut(&[u8])) -> Self {
        Self {
            callback,
            written: 0,
            overflow: false,
        }
    }

    pub fn measure<T: FnMut(&mut CsmWriter<'_>)>(mut callback: T) -> usize {
        let mut noop = |_b: &[u8]| {};
        let mut writer = CsmWriter::new(&mut noop);
        callback(&mut writer);
        writer.written
    }

    pub fn write_params(&mut self, params: &[CsmParam]) {
        for param in params {
            self.write_tlv(param.id, param.value);
        }
    }

    pub fn write_tlv(&mut self, id: u16, value: &[u8]) {
        self.write_tlv_header(id, value.len());
        self.write_data_chunk(value);
    }

    pub fn write_tlv_header(&mut self, id: u16, value_len: usize) {
        let overflow = value_len > u16::MAX as usize - 4;
        // debug_assert!(
        //     !overflow,
        //     "TLV length overflow in CsmWriter::write_tlv_header(): attempted to write header with size {}",
        //     value_len
        // );
        if overflow {
            self.overflow = true;
            return;
        }

        let len = (value_len + 4) as u16;
        self.write_data_chunk(&len.to_be_bytes());
        self.write_data_chunk(&id.to_be_bytes());
    }

    pub fn write_data_chunk(&mut self, data: &[u8]) {
        if let Some(written) = self.written.checked_add(data.len()) {
            self.written = written;
            (self.callback)(data)
        } else {
            self.overflow = true;
        }
    }

    pub fn take_error(&mut self) -> Option<CsmError> {
        if self.overflow {
            return Some(CsmError::Overflow);
        }

        None
    }
}

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
impl<'a> CsmWriter<'a> {
    pub fn with_vec<T: FnMut(&mut CsmWriter<'_>)>(vec: &mut alloc::vec::Vec<u8>, mut callback: T) -> usize {
        let mut cb = |b: &[u8]| {
            vec.extend_from_slice(b);
        };

        let mut writer = CsmWriter::new(&mut cb);
        callback(&mut writer);
        writer.written
    }

    pub fn serialize<T: FnMut(&mut CsmWriter<'_>)>(mut callback: T) -> alloc::vec::Vec<u8> {
        let size = Self::measure(|w| callback(w));
        let mut vec = alloc::vec::Vec::with_capacity(size);
        Self::with_vec(&mut vec, |w| callback(w));

        debug_assert!(
            vec.len() == size,
            "Measurement violation for Vec in CsmWriter::serialize(): {} vs {}",
            vec.len(),
            size
        );

        vec
    }
}
