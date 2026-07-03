pub struct KnobReport {
    pub button: u8,
    pub x: i8,
    pub y: i8,
    pub wheel: i8,
}

impl KnobReport {
    pub fn new(button: bool, home: bool, back: bool, x: i8, y: i8, wheel: i8) -> Self {
        Self {
            button: (button as u8) | ((home as u8) << 1) | ((back as u8) << 2),
            x,
            y,
            wheel,
        }
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        [self.button, self.x as u8, self.y as u8, self.wheel as u8]
    }

    pub fn descriptor() -> [u8; 70] {
        [
            0x05, 0x01, // Usage Page (Generic Desktop)
            0x09, 0x08, // Usage (MultiAxisController)
            0xA1, 0x01, // Collection (Application)
            0x05, 0x09, // Usage Page (Button)
            0x09, 0x01, // Usage (Button 1 primary/trigger)
            0x15, 0x00, // Logical Minimum (0)
            0x25, 0x01, // Logical Maximum (1)
            0x75, 0x01, // Report Size (1)
            0x95, 0x01, // Report Count (1)
            0x81, 0x02, // Input (Data, Variable, Absolute)
            0x05, 0x0c, // Usage Page (Consumer)
            0x0a, 0x23, 0x02, // Usage (AC Home)
            0x0a, 0x24, 0x02, // Usage (AC Back)
            0x95, 0x02, // Report Count (2)
            0x81, 0x02, // Input (Data, Variable, Absolute)
            0x95, 0x05, // Report Size (5)
            0x81, 0x01, // Input (Constant)
            0x05, 0x01, // Usage Page (Generic Desktop)
            0x09, 0x01, // Usage (Pointer)
            0xA1, 0x00, // Collection (Physical)
            0x09, 0x30, // Usage (X)
            0x09, 0x31, // Usage (Y)
            0x15, 0x81, // Logical Minimum (-127)
            0x25, 0x7f, // Logical Maximum (127)
            0x75, 0x08, // Report Size (8)
            0x95, 0x02, // Report Count (2)
            0x81, 0x02, // Input (Data, Variable, Absolute)
            0xC0, // End Collection
            0x09, 0x38, // Usage (Wheel)
            0x15, 0x81, // Logical Minimum (-127)
            0x25, 0x7f, // Logical Maximum (127)
            0x75, 0x08, // Report Size (8)
            0x95, 0x01, // Report Count (1)
            0x81, 0x06, // Input (Data, Variable, Relative)
            0xC0, // End Collection
        ]
    }
}

pub struct KnobBasicReport {
    pub button: u8,
    pub wheel: i8,
}

impl KnobBasicReport {
    pub fn new(button: bool, home: bool, back: bool, wheel: i8) -> Self {
        Self {
            button: (button as u8) | ((home as u8) << 1) | ((back as u8) << 2),
            wheel,
        }
    }

    pub fn to_bytes(&self) -> [u8; 2] {
        [self.button, self.wheel as u8]
    }

    pub fn descriptor() -> [u8; 51] {
        [
            0x05, 0x01, // Usage Page (Generic Desktop)
            0x09, 0x08, // Usage (MultiAxisController)
            0xA1, 0x01, // Collection (Application)
            0x05, 0x09, // Usage Page (Button)
            0x09, 0x01, // Usage (Button 1 primary/trigger)
            0x15, 0x00, // Logical Minimum (0)
            0x25, 0x01, // Logical Maximum (1)
            0x75, 0x01, // Report Size (1)
            0x95, 0x01, // Report Count (1)
            0x81, 0x02, // Input (Data, Variable, Absolute)
            0x05, 0x0c, // Usage Page (Consumer)
            0x0a, 0x23, 0x02, // Usage (AC Home)
            0x0a, 0x24, 0x02, // Usage (AC Back)
            0x95, 0x03, // Report Count (2)
            0x81, 0x02, // Input (Data, Variable, Absolute)
            0x95, 0x05, // Report Size (5)
            0x81, 0x01, // Input (Constant)
            0x05, 0x01, // Usage Page (Generic Desktop)
            0x09, 0x38, // Usage (Wheel)
            0x15, 0x81, // Logical Minimum (-127)
            0x25, 0x7f, // Logical Maximum (127)
            0x75, 0x08, // Report Size (8)
            0x95, 0x01, // Report Count (1)
            0x81, 0x06, // Input (Data, Variable, Relative)
            0xC0, // End Collection
        ]
    }
}

pub struct KnobMinimalReport {
    pub button: u8,
    pub wheel: i8,
}

impl KnobMinimalReport {
    pub fn new(button: bool, wheel: i8) -> Self {
        Self {
            button: button as u8,
            wheel,
        }
    }

    pub fn to_bytes(&self) -> [u8; 2] {
        [self.button, self.wheel as u8]
    }

    pub fn descriptor() -> [u8; 39] {
        [
            0x05, 0x01, // Usage Page (Generic Desktop)
            0x09, 0x08, // Usage (MuultiAxisController)
            0xA1, 0x01, // Collection (Application)
            0x05, 0x09, // Usage Page (Button)
            0x09, 0x01, // Usage (Button 1 primary/trigger)
            0x15, 0x00, // Logical Minimum (0)
            0x25, 0x01, // Logical Maximum (1)
            0x75, 0x01, // Report Size (1)
            0x95, 0x01, // Report Count (1)
            0x81, 0x02, // Input (Data, Variable, Absolute)
            0x95, 0x07, // Report Size (7)
            0x81, 0x01, // Input (Constant)
            0x05, 0x01, // Usage Page (Generic Desktop)
            0x09, 0x38, // Usage (Wheel)
            0x15, 0x81, // Logical Minimum (-127)
            0x25, 0x7f, // Logical Maximum (127)
            0x75, 0x08, // Report Size (8)
            0x95, 0x01, // Report Count (1)
            0x81, 0x06, // Input (Data, Variable, Relative)
            0xC0, // End Collection
        ]
    }
}
