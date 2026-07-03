use crate::{decoder::*, packet_type};

packet_type! {
    pub struct DeviceInformationUpdate: 0x4E09 {
        #[csm_id( 0)] pub device_name: Option<CsmString>,
    }
}

packet_type! {
    pub struct DeviceLanguageUpdate: 0x4E0A {
        #[csm_id( 0)] pub device_language: Option<CsmString>,
    }
}

packet_type! {
    pub struct DeviceTimeUpdate: 0x4E0B {
        #[csm_id( 0)] pub seconds_since_epoch: u64,
        #[csm_id( 1)] pub timezone_offset_minutes: u16,
        #[csm_id( 2)] pub daylight_savings_offset_minutes: u8,
    }
}

packet_type! {
    pub struct DeviceUUIDUpdate: 0x4E0C {
        #[csm_id( 0)] pub uuid: CsmString,
    }
}

packet_type! {
    pub struct WirelessCarPlayUpdate: 0x4E0D {
        #[csm_id( 0)] pub available: bool,
    }
}

packet_type! {
    pub struct DeviceTransportIdentifierNotification: 0x4E0E {
        #[csm_id( 0)] pub bluetooth_transport_identifier: Option<CsmString>,
        #[csm_id( 1)] pub usb_transport_identifier: Option<CsmString>,
    }
}
