use crate::{decoder::*, packet_type};

packet_type! {
    pub struct StartLocationInformation: 0xFFFA {
        #[csm_id( 1)]
        /// If present, the accessory may provide NMEA GPGGA sentences.
        pub global_positioning_system_fix_data: CsmFlag,
        #[csm_id( 2)]
        /// If present, the accessory may provide NMEA GPRMC sentences.
        pub recommended_minimum_specific_gps_transit_data: CsmFlag,
        #[csm_id( 3)]
        /// If present, the accessory may provide NMEA GPGSV sentences.
        pub gps_satellites_in_view: CsmFlag,
        #[csm_id( 4)]
        /// If present, the accessory may provide NMEA PASCD sentences.
        pub vehicle_speed_data: CsmFlag,
        #[csm_id( 5)]
        /// If present, the accessory may provide NMEA PAGCD sentences.
        pub vehicle_gyro_data: CsmFlag,
        #[csm_id( 6)]
        /// If present, the accessory may provide NMEA PAACD sentences.
        pub vehicle_accelerometer_data: CsmFlag,
        #[csm_id( 7)]
        /// If present, the accessory may provide NMEA GPHDT sentences.
        pub vehicle_heading_data: CsmFlag,
    }
}

impl StartLocationInformation {
    pub fn all() -> Self {
        Self {
            global_positioning_system_fix_data: CsmFlag::Yes,
            recommended_minimum_specific_gps_transit_data: CsmFlag::Yes,
            gps_satellites_in_view: CsmFlag::Yes,
            vehicle_speed_data: CsmFlag::Yes,
            vehicle_gyro_data: CsmFlag::Yes,
            vehicle_accelerometer_data: CsmFlag::Yes,
            vehicle_heading_data: CsmFlag::Yes,
        }
    }
}
packet_type! {
    pub struct GPRMCDataStatusValuesNotification: 0xFFF0 {
        #[csm_id( 0)] pub status_value_a: CsmFlag,
        #[csm_id( 1)] pub status_value_v: CsmFlag,
        #[csm_id( 2)] pub status_value_x: CsmFlag,

    }
}

packet_type! {
    pub struct LocationInformation: 0xFFFB {
        #[csm_id( 0)] pub nmea_sentence: CsmVec<CsmString>,
    }
}

packet_type! {
    pub struct StopLocationInformation: 0xFFFC {
    }
}
