use crate::packet_type;
use crate::{decoder::CsmFlag, enum_type};

packet_type! {
    pub struct StartPowerUpdates: 0xAE00 {
        #[csm_id( 0)] pub maximum_current_drawn_from_accessory: CsmFlag,
        #[csm_id( 1)] pub device_battery_will_charge_if_power_is_present: CsmFlag,
        #[csm_id( 2)] pub accessory_power_mode: CsmFlag,
        #[csm_id( 4)] pub is_external_charger_connected: CsmFlag,
        #[csm_id( 5)] pub battery_charging_state: CsmFlag,
        #[csm_id( 6)] pub battery_charge_level: CsmFlag,

    }
}

packet_type! {
    pub struct PowerUpdate: 0xAE01 {
        #[csm_id( 0)] pub maximum_current_drawn_from_accessory: Option<u16>,
        #[csm_id( 1)] pub device_battery_will_charge_if_power_is_present: Option<bool>,
        #[csm_id( 2)] pub accessory_power_mode: Option<AccessoryPowerModes>,
        #[csm_id( 4)] pub is_external_charger_connected: Option<bool>,
        #[csm_id( 5)] pub battery_charging_state: Option<BatteryChargingState>,
        #[csm_id( 6)] pub battery_charge_level: Option<u16>,
    }
}

packet_type! {
    pub struct StopPowerUpdates: 0xAE02 {
    }
}

packet_type! {
    pub struct PowerSourceUpdate: 0xAE03 {
        #[csm_id( 0)] pub available_current_for_device: Option<u16>,
        #[csm_id( 1)] pub device_battery_should_charge_if_power_is_present: Option<bool>,
    }
}

enum_type! {
    pub enum AccessoryPowerModes {
        Reserved = 0,
        LowPowered = 1,
        IntermittentHighPowerOrUltraHighPower = 2
    }
}

enum_type! {
    pub enum BatteryChargingState {
        Disabled = 0,
        Charging = 1,
        Charged = 2
    }
}
