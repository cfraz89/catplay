use serde::Deserialize;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct AppConfig {
    pub debug: bool,
    pub persist_dir: Option<String>,

    pub mfi: MfiConfig,
    pub bluetooth: BluetoothConfig,

    pub wifi: WifiConfig,
    pub wifi_network: WifiNetwork,

    pub gadget: GadgetConfig,
}

// MFI start

#[derive(Debug, Deserialize, Default, Clone)]
pub struct MfiConfig {
    pub server: Option<MfiConfigServer>,
    pub client: Option<MfiConfigClient>,
    pub i2c: Option<MfiConfigI2C>,
    pub selftest: bool,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct MfiConfigServer {
    pub enabled: bool,
    pub bind: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct MfiConfigClient {
    pub remote: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct MfiConfigI2C {
    pub bus_offset: u32,
    pub dev_addr: u8,
    pub timeout_ms: u64,
}

// Bluetooth start

#[derive(Debug, Deserialize, Default, Clone)]
pub struct BluetoothConfig {
    pub enabled: bool,
    pub device: String,
    pub name: String,
}

// Wifi start

#[derive(Debug, Deserialize, Default, Clone)]
pub struct WifiConfig {
    pub enabled: bool,
    pub device: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub password: String,
    pub wpa: bool,
    pub channel: Option<u8>,
}

// Gadget start

#[derive(Debug, Deserialize, Default, Clone)]
pub struct GadgetConfig {
    pub enabled: bool,
    pub pinned: bool,
    pub udc_car: Option<String>,
    pub udc_extra: Option<String>,
}
