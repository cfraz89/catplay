#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CsmFileTransferEvent {
    Offer {
        size: u64,
        file_type: u16,
        setup_data: Vec<u8>,
    },
    Completed {
        data: Vec<u8>,
        size: u64,
        file_type: u16,
        setup_data: Vec<u8>,
    },
    Cancelled,
}
