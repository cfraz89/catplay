use catplay_bonjour::{BonjourEntryType, get_bonjour_txt_optional};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CarPlayCtrlBonjourEntry {
    /// SDK version
    pub srcvers: String,

    /// Bluetooth MAC address of the phone (AA:BB:CC:EE:DD)
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfoBonjourEntry {
    pub model: String,
}

impl BonjourEntryType for CarPlayCtrlBonjourEntry {
    const SERVICE_TYPE: &'static str = "_carplay-ctrl._tcp.local.";
    type Entry = Self;

    fn from_props(data: &HashMap<String, String>) -> Self {
        Self {
            srcvers: get_bonjour_txt_optional(data, "srcvers"),
            id: get_bonjour_txt_optional(data, "id"),
        }
    }

    fn to_props(data: Self, output: &mut HashMap<String, String>) {
        output.insert("srcvers".into(), data.srcvers.clone());
        output.insert("id".into(), data.id.clone());
    }
}

impl BonjourEntryType for DeviceInfoBonjourEntry {
    const SERVICE_TYPE: &'static str = "_device-info._tcp.local.";
    type Entry = Self;

    fn from_props(data: &HashMap<String, String>) -> Self {
        Self {
            model: get_bonjour_txt_optional(data, "model"),
        }
    }

    fn to_props(data: Self, output: &mut HashMap<String, String>) {
        output.insert("model".into(), data.model.clone());
    }
}
