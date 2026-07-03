use crate::{
    common::AirPlayEncryptionType,
    msg::{ControllerFeature, StreamDescription, StreamDescriptionResponse},
};
use catplay_plist::{PlistByteArray, plist_struct};

plist_struct! {
    pub struct InitialSetup {
        /// Bluetooth MAC address (preferably)
        #[serde(rename = "deviceID")]
        pub device_id: String,
        #[serde(default)]
        pub mac_address: String,
        pub model: String,
        pub name: String,
        pub os_build_version: String,

        #[serde(rename = "sessionUUID")]
        pub session_uuid: String,
        pub source_version: String,

        pub streams: Option<Vec<StreamDescription>>,

        #[serde(default)]
        pub stats_collection_enabled: bool,
        pub timing_port: u16,

        // Modern extra keys
        pub os_version: Option<String>,
        pub os_name: Option<String>,
        #[serde(rename = "sessionCorrelationUUID")]
        pub session_correlation_uuid: Option<String>,
        #[serde(default)]
        pub update_session_request: bool,
        #[serde(default)]
        pub keep_alive_low_power: bool,
        #[serde(default)]
        pub features: Vec<ControllerFeature>,

        // Non-CarPlay keys
        pub et: Option<AirPlayEncryptionType>,
        pub ekey: Option<PlistByteArray>,
        pub eiv: Option<PlistByteArray>,
        pub timing_protocol: Option<String>,

        #[serde(default)]
        pub is_screen_mirroring_session: bool

    }
}

plist_struct! {
    pub struct InitialSetupResponse {
        pub event_port: u16,
        pub keep_alive_port: Option<u16>,
        pub streams: Option<Vec<StreamDescriptionResponse>>,
        pub timing_port: u16,

        // Modern extra keys
        #[serde(default)]
        pub enabled_features: Vec<ControllerFeature>,
    }
}
