use crate::{FileTransferOp, FileTransferPayload};
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileTransferEvent {
    Setup { size: u64, file_type: u16, setup_data: Vec<u8> },
    Data { data: Vec<u8>, is_final_chunk: bool },

    Start,
    Cancel,
    Pause,
    Success,
    Failure,
}

impl TryFrom<&FileTransferPayload> for FileTransferEvent {
    type Error = ();

    fn try_from(value: &FileTransferPayload) -> Result<Self, Self::Error> {
        let mut payload = value.payload.clone();
        let ret = match value.op {
            FileTransferOp::Start => FileTransferEvent::Start,
            FileTransferOp::Cancel => FileTransferEvent::Cancel,
            FileTransferOp::Pause => FileTransferEvent::Pause,
            FileTransferOp::Setup => {
                if payload.len() < 10 {
                    return Err(());
                }

                let size = u64::from_be_bytes(payload[0..8].try_into().unwrap());
                let file_type = u16::from_be_bytes([payload[8], payload[9]]);

                FileTransferEvent::Setup {
                    size,
                    file_type,
                    setup_data: payload.split_off(10),
                }
            }
            FileTransferOp::Success => FileTransferEvent::Success,
            FileTransferOp::Failure => FileTransferEvent::Failure,

            FileTransferOp::FirstData | FileTransferOp::Data => FileTransferEvent::Data {
                data: payload,
                is_final_chunk: false,
            },
            FileTransferOp::FirstAndOnlyData | FileTransferOp::LastData => FileTransferEvent::Data {
                data: payload,
                is_final_chunk: true,
            },
        };

        Ok(ret)
    }
}
