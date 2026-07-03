use crate::decoder::{CsmError, CsmParamEncodeBytes, CsmWriter};

const CSM_HEADER_MAGIC: u16 = 0x4040;
const CSM_HEADER_SIZE: usize = 6;

const CSM_PACKET_PAYLOAD_MAX: usize = u16::MAX as usize - CSM_HEADER_SIZE;

#[derive(Debug, Clone, Copy)]
pub struct CsmPacketHeader {
    pub payload_length: u16,
    pub id: u16,
}

impl CsmPacketHeader {
    pub fn new(payload_length: u16, id: u16) -> Self {
        Self { payload_length, id }
    }
}

pub struct CsmPacketWithPayload<'a> {
    pub header: CsmPacketHeader,
    pub payload: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for CsmPacketWithPayload<'a> {
    type Error = CsmError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        let header = CsmPacketHeader::try_from(data)?;
        let header_size = CSM_HEADER_SIZE;
        let payload_size = header.payload_length as usize;

        if data.len() < (header_size + payload_size) {
            return Err(CsmError::PacketUnderflow);
        }

        let payload = &data[header_size..header_size + payload_size];
        CsmPacketWithPayload::new(header.id, payload)
    }
}

impl TryFrom<&[u8]> for CsmPacketHeader {
    type Error = CsmError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < CSM_HEADER_SIZE {
            return Err(CsmError::PacketUnderflow);
        }

        if u16::from_be_bytes([data[0], data[1]]) != CSM_HEADER_MAGIC {
            return Err(CsmError::PacketMagic);
        }

        let length = u16::from_be_bytes([data[2], data[3]]);
        let id = u16::from_be_bytes([data[4], data[5]]);

        if length < CSM_HEADER_SIZE as u16 {
            return Err(CsmError::PacketUnderflow);
        }

        Ok(CsmPacketHeader {
            payload_length: length - CSM_HEADER_SIZE as u16,
            id,
        })
    }
}

impl<'a> CsmPacketWithPayload<'a> {
    pub fn new(id: u16, payload: &'a [u8]) -> Result<Self, CsmError> {
        // debug_assert!(
        //     payload.len() <= CSM_PACKET_PAYLOAD_MAX,
        //     "TLV length overflow in CsmPacketWithPayload::new(): attempted to create payload with size {}",
        //     payload.len()
        // );

        if payload.len() > CSM_PACKET_PAYLOAD_MAX {
            return Err(CsmError::PacketOverflow);
        }

        Ok(Self {
            header: CsmPacketHeader {
                payload_length: payload.len() as u16,
                id,
            },
            payload,
        })
    }

    pub fn size(&self) -> usize {
        CSM_HEADER_SIZE + self.payload.len()
    }

    pub fn split(self) -> (CsmPacketHeader, &'a [u8]) {
        (self.header, self.payload)
    }
}

impl<'a> CsmParamEncodeBytes for CsmPacketWithPayload<'a> {
    fn encode_to_bytes(&self, writer: &mut CsmWriter) {
        writer.write_data_chunk(&CSM_HEADER_MAGIC.to_be_bytes());
        writer.write_tlv_header(self.header.id, self.payload.len() + 2);
        writer.write_data_chunk(self.payload);
    }
}
