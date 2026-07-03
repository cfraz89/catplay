use crate::enum_type;
use crate::{decoder::*, packet_type};

enum_type! {
    pub enum WiFiRequestStatus {
        Success = 0,
        UserDefined = 1,
        NetworkInfoUnavailable = 2
    }
}

enum_type! {
    pub enum WiFiSecurityType {
        None = 0,
        WEP = 1,
        WpaOrWpa2 = 2
    }
}

packet_type! {
    pub struct RequestWiFiInformation: 0x5700 {
    }
}

packet_type! {
    pub struct WiFiInformation: 0x5701 {
        #[csm_id( 0)] pub request_status: WiFiRequestStatus,
        #[csm_id( 2)] pub wifi_ssid: Option<CsmString>,
        #[csm_id( 3)] pub wifi_passphrase: Option<CsmString>,
    }
}

packet_type! {
    pub struct RequestAccessoryWiFiConfigurationInformation: 0x5702 {
    }
}

packet_type! {
    pub struct AccessoryWiFiConfigurationInformation: 0x5703 {
        #[csm_id( 1)] pub wifi_ssid: CsmString,
        #[csm_id( 2)] pub passphrase: Option<CsmString>,
        #[csm_id( 3)] pub security_type: Option<WiFiSecurityType>,
        #[csm_id( 4)] pub channel: Option<u8>,
    }
}
