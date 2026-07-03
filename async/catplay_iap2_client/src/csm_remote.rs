use macaddr::MacAddr6;

/// Represents remote side of iAP2 connection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CsmRemote {
    UsbHost,
    UsbGadget,
    Bluetooth { remote_mac: MacAddr6, local_mac: MacAddr6 },
    AirPlay,
    Unknown,
}

impl CsmRemote {
    pub fn usb_host() -> Self {
        CsmRemote::UsbHost
    }

    pub fn usb_gadget() -> Self {
        CsmRemote::UsbGadget
    }

    pub fn bluetooth(remote_mac: MacAddr6, local_mac: MacAddr6) -> Self {
        CsmRemote::Bluetooth { remote_mac, local_mac }
    }

    pub fn unknown() -> Self {
        CsmRemote::Unknown
    }

    pub fn airplay() -> Self {
        CsmRemote::AirPlay
    }

    pub fn bluetooth_mac(&self) -> Option<MacAddr6> {
        match self {
            CsmRemote::Bluetooth { remote_mac, .. } => Some(*remote_mac),
            _ => None,
        }
    }
}
