pub struct TouchScreenSingleReport {
    pub touch: u8,
    pub x: u16,
    pub y: u16,
}

impl TouchScreenSingleReport {
    pub fn new(touch: bool, x: u16, y: u16) -> Self {
        Self { touch: touch as u8, x, y }
    }

    pub fn is_touched(&self) -> bool {
        self.touch != 0
    }

    pub fn to_bytes(&self) -> [u8; 5] {
        let mut buf = [0u8; 5];
        buf[0] = self.touch & 0x01;
        buf[1..3].copy_from_slice(&self.x.to_le_bytes());
        buf[3..5].copy_from_slice(&self.y.to_le_bytes());
        buf
    }

    pub fn descriptor(width: u16, height: u16) -> [u8; 62] {
        let mut buf = [
            0x05, 0x0D, // Usage Page (Digitizer)
            0x09, 0x04, // Usage (Touch Screen)
            0xA1, 0x01, // Collection (Application)
            0x05, 0x0D, // Usage Page (Digitizer)
            0x09, 0x22, // Usage (Finger)
            0xA1, 0x02, // Collection (Logical)
            // Finger
            0x05, 0x0D, // Usage Page (Digitizer)
            0x09, 0x33, // Usage (Touch)
            0x15, 0x00, // Logical Minimum (0)
            0x25, 0x01, // Logical Maximum (1)
            0x75, 0x01, // Report Size (1)
            0x95, 0x01, // Report Count (1)
            0x81, 0x02, // Input (Data, Variable, Absolute)
            // Constant
            0x75, 0x07, // Report Size (7)
            0x95, 0x01, // Report Count (1)
            0x81, 0x01, // Input (Constant)
            // X Y
            0x05, 0x01, // Usage Page (Generic Desktop)
            0x09, 0x30, // Usage (X)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x7f, // Logical Maximum (width)
            0x75, 0x10, // Report Size (16)
            0x95, 0x01, // Report Count (1)
            0x81, 0x02, // Input (Data, Variable, Absolute)
            0x09, 0x31, // Usage (Y)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x7f, // Logical Maximum (height)
            0x75, 0x10, // Report Size (16)
            0x95, 0x01, // Report Count (1)
            0x81, 0x02, // Input (Data, Variable, Absolute)
            0xC0, // End Collection
            0xC0, // End Collection
        ];

        buf[0x27..0x29].copy_from_slice(&width.to_le_bytes());
        buf[0x34..0x36].copy_from_slice(&height.to_le_bytes());
        buf
    }
}

pub struct TouchScreenSingleCancelReport {
    pub buttons: u8,
    pub x: u16,
    pub y: u16,
}

impl TouchScreenSingleCancelReport {
    pub fn new(touch: bool, untouch: bool, x: u16, y: u16) -> Self {
        Self {
            buttons: (touch as u8) | ((untouch as u8) << 1),
            x,
            y,
        }
    }

    pub fn to_bytes(&self) -> [u8; 5] {
        let mut buf = [0u8; 5];
        buf[0] = self.buttons & 0x03;
        buf[1..3].copy_from_slice(&self.x.to_le_bytes());
        buf[3..5].copy_from_slice(&self.y.to_le_bytes());
        buf
    }

    pub fn descriptor(width: u16, height: u16) -> [u8; 66] {
        let mut buf = [
            0x05, 0x0D, // Usage Page (Digitizer)
            0x09, 0x04, // Usage (Touch Screen)
            0xA1, 0x01, // Collection (Application)
            0x05, 0x0D, // Usage Page (Digitizer)
            0x09, 0x22, // Usage (Finger)
            0xA1, 0x02, // Collection (Logical)
            // Finger
            0x05, 0x0D, // Usage Page (Digitizer)
            0x09, 0x33, // Usage (Touch)
            0x15, 0x00, // Logical Minimum (0)
            0x25, 0x01, // Logical Maximum (1)
            0x75, 0x01, // Report Size (1)
            0x95, 0x01, // Report Count (1)
            0x81, 0x02, // Input (Data, Variable, Absolute)
            // Cancel touch
            0x09, 0x34, // Usage (Untouch)
            0x81, 0x06, // Input (Data, Variable, Relative)
            // Constant
            0x75, 0x06, // Report Size (6)
            0x95, 0x01, // Report Count (1)
            0x81, 0x01, // Input (Constant)
            // X Y
            0x05, 0x01, // Usage Page (Generic Desktop)
            0x09, 0x30, // Usage (X)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x7f, // Logical Maximum (width)
            0x75, 0x10, // Report Size (16)
            0x95, 0x01, // Report Count (1)
            0x81, 0x02, // Input (Data, Variable, Absolute)
            0x09, 0x31, // Usage (Y)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x7f, // Logical Maximum (height)
            0x75, 0x10, // Report Size (16)
            0x95, 0x01, // Report Count (1)
            0x81, 0x02, // Input (Data, Variable, Absolute)
            0xC0, // End Collection
            0xC0, // End Collection
        ];

        buf[0x2B..0x2D].copy_from_slice(&width.to_le_bytes());
        buf[0x38..0x3A].copy_from_slice(&height.to_le_bytes());
        buf
    }
}
