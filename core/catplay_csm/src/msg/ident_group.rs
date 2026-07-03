use super::*;
use crate::{decoder::*, group_type};

group_type! {
    pub struct ExternalAccessoryProtocol {
        #[csm_id( 0)] pub external_accessory_protocol_identifier: u8,
        #[csm_id( 1)] pub external_accessory_protocol_name: CsmString,
        #[csm_id( 2)] pub external_accessory_protocol_match_action: MatchAction,
        #[csm_id( 3)] pub native_transport_component_identifier: Option<u16>,
        #[csm_id( 4)] pub external_accessory_protocol_car_play: Option<u16>
    }
}

group_type! {
    pub struct USBDeviceTransportComponent {
        #[csm_id( 0)] pub transport_component_identifier: u16,
        #[csm_id( 1)] pub transport_component_name: CsmString,
        #[csm_id( 2)] pub transport_supports_iap2_connection: CsmFlag,
        #[csm_id( 3)] pub usbdevice_supported_audio_sample_rate: CsmVec<USBDeviceModeAudioSampleRate>,
    }
}

group_type! {
    pub struct USBHostTransportComponent {
        #[csm_id( 0)] pub transport_component_identifier: u16,
        #[csm_id( 1)] pub transport_component_name: CsmString,
        #[csm_id( 2)] pub transport_supports_iap2_connection: CsmFlag,
        #[csm_id( 3)] pub usbhost_transport_car_play_interface_number: Option<u8>,
        #[csm_id( 4)] pub transport_supports_car_play: CsmFlag,
    }
}

group_type! {
    pub struct SerialTransportComponent {
        #[csm_id( 0)] pub transport_component_identifier: u16,
        #[csm_id( 1)] pub transport_component_name: CsmString,
        #[csm_id( 2)] pub transport_supports_iap2_connection: CsmFlag
    }
}

group_type! {
    pub struct BluetoothTransportComponent {
        #[csm_id( 0)] pub transport_component_identifier: u16,
        #[csm_id( 1)] pub transport_component_name: CsmString,
        #[csm_id( 2)] pub transport_supports_iap2_connection: CsmFlag,
        #[csm_id( 3)] pub bluetooth_transport_mac_address: CsmByteArray
    }
}

group_type! {
    pub struct IAP2HIDComponent {
        #[csm_id( 0)] pub hidcomponent_identifier: u16,
        #[csm_id( 1)] pub hidcomponent_name: CsmString,
        #[csm_id( 2)] pub hidcomponent_function: HIDComponentFunction,
    }
}

group_type! {
    pub struct VehicleInformationComponent {
        #[csm_id( 0)] pub identifier: u16,
        #[csm_id( 1)] pub name: CsmString,
        #[csm_id( 2)] pub engine_type: EngineTypes,
        #[csm_id( 6)] pub display_name: CsmString
        }
}

group_type! {
    pub struct VehicleStatusComponent {
        #[csm_id( 0)] pub identifier: u16,
        #[csm_id( 1)] pub name: CsmString,
        #[csm_id( 3)] pub range: CsmFlag,
        #[csm_id( 4)] pub outside_temperature: CsmFlag,
        #[csm_id( 6)] pub range_warning: CsmFlag,

    }
}

group_type! {
    pub struct LocationInformationComponent {
        #[csm_id( 0)] pub identifier: u16,
        #[csm_id( 1)] pub name: CsmString,
        #[csm_id(17)] pub global_positioning_system_fix_data: CsmFlag,
        #[csm_id(18)] pub recommended_minimum_specific_gpstransit_data: CsmFlag,
        #[csm_id(19)] pub gpssatellites_in_view: CsmFlag,
        #[csm_id(20)] pub vehicle_speed_data: CsmFlag,
        #[csm_id(21)] pub vehicle_gyro_data: CsmFlag,
        #[csm_id(22)] pub vehicle_accelerometer_data: CsmFlag,
        #[csm_id(23)] pub vehicle_heading_data: CsmFlag,
    }
}

group_type! {
    pub struct USBHostHIDComponent {
        #[csm_id( 0)] pub hidcomponent_identifier: u16,
        #[csm_id( 1)] pub hidcomponent_name: CsmString,
        #[csm_id( 2)] pub hidcomponent_function: HIDComponentFunction,
        #[csm_id( 3)] pub usbhost_transport_component_identifier: u32,
        #[csm_id( 4)] pub usbhost_transport_interface_number: u32
    }
}

group_type! {
    pub struct WirelessCarPlayTransportComponent {
        #[csm_id( 0)] pub transport_component_identifier: u16,
        #[csm_id( 1)] pub transport_component_name: CsmString,
        #[csm_id( 2)] pub transport_supports_iap2_connection: CsmFlag,
        #[csm_id( 4)] pub transport_supports_car_play: CsmFlag,
        /* Seems to be a direct trigger for marking device as "CarPlay Ultra" on CarPlay device list */
        #[csm_id( 5)] pub transport_supports_mutual_auth: CsmFlag,
    }

}

group_type! {
    pub struct BluetoothHIDComponent {
        #[csm_id( 0)] pub hidcomponent_identifier: u16,
        #[csm_id( 1)] pub hidcomponent_name: CsmString,
        #[csm_id( 2)] pub hidcomponent_function: HIDComponentFunction,
        #[csm_id( 3)] pub bluetooth_transport_component_identifier: u16
    }
}
