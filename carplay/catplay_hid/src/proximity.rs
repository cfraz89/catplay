pub struct ProximityReport {
    pub presence: u8,
}

impl ProximityReport {
    pub fn new(presence: bool) -> Self {
        Self { presence: presence as u8 }
    }

    pub fn to_bytes(&self) -> [u8; 1] {
        [self.presence]
    }

    pub fn descriptor() -> [u8; 22] {
        [
            0x05, 0x20, // Usage Page (Sensor)
            0x09, 0x11, // Usage 0x11 (Biometric Human Presence)
            0xA1, 0x01, // Collection (Application)
            // Input Reports
            0x05, 0x20, //   Usage Page (Sensor)
            0x0A, 0xB1, 0x04, //   Usage 0x4B1 (Sensor Data Biometric Human Presence)
            0x15, 0x00, //   Logical Minimum......... (0)
            0x25, 0x01, //   Logical Maximum......... (1)
            0x75, 0x08, //   Report Size............. (8)
            0x95, 0x01, //   Report Count............ (1)
            0x81, 0x02, //   Input................... (Data, Variable, Absolute)
            0xC0, // End collection
        ]
    }
}
