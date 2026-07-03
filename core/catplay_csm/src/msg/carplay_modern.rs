use crate::csm_struct;
use crate::decoder::*;
use crate::{decoder::CsmFlag, packet_type};

packet_type! {
    pub struct CarPlayAvailability: 0x4300 {
        #[csm_id(1)]
        pub carplay_capability: CsmFlag,

        #[csm_id(2)]
        pub bluetooth_connected: CsmFlag,

        #[csm_id(3)]
        pub usb_connected: CsmFlag,

        #[csm_id(4)]
        pub wifi_supported: CsmFlag,

        #[csm_id(5)]
        pub wired_supported: CsmFlag,

        #[csm_id(6)]
        pub wifi_networks: CsmVec<CsmString>,

        #[csm_id(7)]
        pub supports_mutual_auth: bool
    }
}

csm_struct! {
    pub struct WirelessConnectionInfo {
        #[csm_id(0)] pub ssid: Option<CsmString>,
        #[csm_id(1)] pub password: Option<CsmString>,
        #[csm_id(2)] pub channel: Option<u8>,
        #[csm_id(3)] pub ip_addresses: CsmVec<CsmString>,
        #[csm_id(4)] pub security_type: Option<u8>,
    }
}

packet_type! {
    pub struct CarPlayStartSession: 0x4301 {
        #[csm_id(0)]
        pub wired_ip_addresses: CsmVec<CsmString>,

        #[csm_id(1)]
        pub wireless_connection_info: CsmVec<WirelessConnectionInfo>,

        #[csm_id(2)]
        pub port: Option<u32>,

        #[csm_id(3)]
        pub source_version: Option<CsmString>,

        #[csm_id(4)]
        pub password_legacy: Option<CsmString>,

        #[csm_id(5)]
        pub security_type: Option<CsmString>,

        #[csm_id(6)]
        pub sdk_version: Option<CsmString>,

        // #[csm_id(7)]
        // pub cluster_assets: CsmVec<ClusterAssetInfo>,

        #[csm_id(8)]
        pub supports_mutual_auth: bool,
    }
}

packet_type! {
    pub struct AvailableDigitalCarKeys: 0x4302 {

    }
}

packet_type! {
    pub struct MatchedDigitalCarKeys: 0x4303 {

    }
}
