use crate::enum_type;
use crate::packet_type;

packet_type! {
    pub struct StartExternalAccessoryProtocolSession: 0xEA00 {
        #[csm_id( 0)] pub external_accessory_protocol_identifier: u8,
        #[csm_id( 1)] pub external_accessory_protocol_session_identifier: u16,
    }
}

packet_type! {
    pub struct StopExternalAccessoryProtocolSession: 0xEA01 {
        #[csm_id( 0)] pub external_accessory_protocol_session_identifier: u16,
    }
}

packet_type! {
    pub struct StatusExternalAccessoryProtocolSession: 0xEA03 {
        #[csm_id( 0)] pub external_accessory_protocol_session_identifier: u16,
        #[csm_id( 1)] pub external_accessory_protocol_session_status: ExternalAccessoryProtocolSessionStatus,
    }
}

enum_type! {
    pub enum ExternalAccessoryProtocolSessionStatus {
        SessionStatusOK = 0,
        SessionClose = 1
    }
}
