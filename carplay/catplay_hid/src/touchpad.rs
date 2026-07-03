pub struct TouchpadOnlyReport {
    pub transducer_state: u8,
    pub transducer_x: u16,
    pub transducer_y: u16,
}

impl TouchpadOnlyReport {
    pub fn new(transducer_state: u8, transducer_x: u16, transducer_y: u16) -> Self {
        Self {
            transducer_state,
            transducer_x,
            transducer_y,
        }
    }

    pub fn to_bytes(&self) -> [u8; 5] {
        let mut buf = [0u8; 5];
        buf[0] = self.transducer_state;
        buf[1..3].copy_from_slice(&self.transducer_x.to_le_bytes());
        buf[3..5].copy_from_slice(&self.transducer_y.to_le_bytes());
        buf
    }

    pub fn descriptor(width: u16, height: u16, width_mm: u16, height_mm: u16) -> [u8; 80] {
        let mut buf = [
            0x05, 0x0D, // Usage Page (Digitizer)
            0x09, 0x05, // Usage (Touch Pad)
            0xA1, 0x01, // Collection (Application)
            0x05, 0x0D, //   Usage Page (Digitizer)
            0x09, 0x22, //   Usage (Finger)
            0xA1, 0x02, //   Collection (Logical)
            0x05, 0x0D, //     Usage Page (Digitizer)
            0x09, 0x33, //     Usage (Touch)
            0x15, 0x00, //     Logical Minimum......... (0)
            0x25, 0x01, //     Logical Maximum......... (1)
            0x75, 0x01, //     Report Size............. (1)
            0x95, 0x01, //     Report Count............ (1)
            0x81, 0x02, //     Input...................(Data, Variable, Absolute)
            0x75, 0x07, //     Report Size............. (7)
            0x95, 0x01, //     Report Count............ (1)
            0x81, 0x01, //     Input...................(Constant)
            0x05, 0x01, //     Usage Page (Generic Desktop)
            0x09, 0x30, //     Usage (X)
            0x15, 0x00, //     Logical Minimum......... (0)
            0x26, 0xFF, 0xFF, //     Logical Maximum......... (width)
            0x35, 0x00, //     Physical Minimum........ (0)
            0x46, 0xFF, 0xFF, //     Physical Maximum........ (widthMM)
            0x55, 0x0F, //     Unit Exponent (-1)
            0x65, 0x11, //     Unit (cm)
            0x75, 0x10, //     Report Size............. (16)
            0x95, 0x01, //     Report Count............ (1)
            0x81, 0x02, //     Input...................(Data, Variable, Absolute)
            0x09, 0x31, //     Usage (Y)
            0x15, 0x00, //     Logical Minimum......... (0)
            0x26, 0xFF, 0xFF, //     Logical Maximum......... (height)
            0x35, 0x00, //     Physical Minimum........ (0)
            0x46, 0xFF, 0xFF, //     Physical Maximum........ (heightMM)
            0x55, 0x0F, //     Unit Exponent (-1)
            0x65, 0x11, //     Unit (cm)
            0x75, 0x10, //     Report Size............. (16)
            0x95, 0x01, //     Report Count............ (1)
            0x81, 0x02, //     Input...................(Data, Variable, Absolute)
            0xC0, //   End Collection
            0xC0, // End Collection
        ];

        buf[39..41].copy_from_slice(&width.to_le_bytes());
        buf[44..46].copy_from_slice(&width_mm.to_le_bytes());
        buf[61..63].copy_from_slice(&height.to_le_bytes());
        buf[66..68].copy_from_slice(&height_mm.to_le_bytes());
        buf
    }
}

pub struct TouchpadMultiCharacterReport {
    pub character1_length: u8,
    pub character1_data: [u8; 4],
    pub character1_quality: u8,
    pub character2_length: u8,
    pub character2_data: [u8; 4],
    pub character2_quality: u8,
    pub transducer_state: u8,
    pub transducer_x: u16,
    pub transducer_y: u16,
}

impl TouchpadMultiCharacterReport {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        character1_length: u8,
        character1_data: [u8; 4],
        character1_quality: u8,
        character2_length: u8,
        character2_data: [u8; 4],
        character2_quality: u8,
        transducer_state: u8,
        transducer_x: u16,
        transducer_y: u16,
    ) -> Self {
        Self {
            character1_length,
            character1_data,
            character1_quality,
            character2_length,
            character2_data,
            character2_quality,
            transducer_state,
            transducer_x,
            transducer_y,
        }
    }

    pub fn to_bytes(&self) -> [u8; 19] {
        let mut buf = [0u8; 19];
        let character1_length = self.character1_length.min(4);
        let character2_length = self.character2_length.min(4);

        buf[0] = self.transducer_state;
        buf[1..3].copy_from_slice(&self.transducer_x.to_le_bytes());
        buf[3..5].copy_from_slice(&self.transducer_y.to_le_bytes());
        buf[5..5 + character1_length as usize].copy_from_slice(&self.character1_data[..character1_length as usize]);
        buf[9] = 1; // utf8 encoding
        buf[10] = character1_length;
        buf[11] = self.character1_quality;
        buf[12..12 + character2_length as usize].copy_from_slice(&self.character2_data[..character2_length as usize]);
        buf[16] = 1; // utf8 encoding
        buf[17] = character2_length;
        buf[18] = self.character2_quality;
        buf
    }

    pub fn descriptor(width: u16, height: u16, width_mm: u16, height_mm: u16) -> [u8; 160] {
        let mut buf = [
            0x05, 0x0D, // Usage Page (Digitizer)
            0x09, 0x05, // Usage (Touch Pad)
            0xA1, 0x01, // Collection (Application)
            0x05, 0x0D, //   Usage Page (Digitizer)
            0x09, 0x22, //   Usage (Finger)
            0xA1, 0x02, //   Collection (Logical)
            0x05, 0x0D, //     Usage Page (Digitizer)
            0x09, 0x33, //     Usage (Touch)
            0x15, 0x00, //     Logical Minimum......... (0)
            0x25, 0x01, //     Logical Maximum......... (1)
            0x75, 0x01, //     Report Size............. (1)
            0x95, 0x01, //     Report Count............ (1)
            0x81, 0x02, //     Input...................(Data, Variable, Absolute)
            0x75, 0x07, //     Report Size............. (7)
            0x95, 0x01, //     Report Count............ (1)
            0x81, 0x01, //     Input...................(Constant)
            0x05, 0x01, //     Usage Page (Generic Desktop)
            0x09, 0x30, //     Usage (X)
            0x15, 0x00, //     Logical Minimum......... (0)
            0x26, 0xFF, 0xFF, //     Logical Maximum......... (width)
            0x35, 0x00, //     Physical Minimum........ (0)
            0x46, 0xFF, 0xFF, //     Physical Maximum........ (widthMM)
            0x55, 0x0F, //     Unit Exponent (-1)
            0x65, 0x11, //     Unit (cm)
            0x75, 0x10, //     Report Size............. (16)
            0x95, 0x01, //     Report Count............ (1)
            0x81, 0x02, //     Input...................(Data, Variable, Absolute)
            0x09, 0x31, //     Usage (Y)
            0x15, 0x00, //     Logical Minimum......... (0)
            0x26, 0xFF, 0xFF, //     Logical Maximum......... (height)
            0x35, 0x00, //     Physical Minimum........ (0)
            0x46, 0xFF, 0xFF, //     Physical Maximum........ (heightMM)
            0x55, 0x0F, //     Unit Exponent (-1)
            0x65, 0x11, //     Unit (cm)
            0x75, 0x10, //     Report Size............. (16)
            0x95, 0x01, //     Report Count............ (1)
            0x81, 0x02, //     Input...................(Data, Variable, Absolute)
            0xC0, //   End Collection
            0x05, 0x0D, //   Usage Page (Digitizer)
            0x09, 0x24, //   Usage (Gesture Character)
            0xA1, 0x02, //   Collection (Logical)
            0x05, 0x0D, //     Usage Page (Digitizer)
            0x09, 0x63, //     Usage (Gesture Character Data)
            0x75, 0x20, //     Report Size............. (32)
            0x95, 0x01, //     Report Count............ (1)
            0x82, 0x02, 0x01, //     Input...................(Data, Variable, Absolute, Buffered bytes)
            0x09, 0x65, //     Usage (Gesture Character Encoding UTF8)
            0x09, 0x62, //     Usage (Gesture Character Data Length)
            0x75, 0x08, //     Report Size............. (8)
            0x95, 0x02, //     Report Count............ (2)
            0x81, 0x02, //     Input...................(Data, Variable, Absolute)
            0x09, 0x61, //     Usage (Gesture Character Quality)
            0x15, 0x00, //     Logical Minimum......... (0)
            0x25, 0x64, //     Logical Maximum......... (100)
            0x75, 0x08, //     Report Size............. (8)
            0x95, 0x01, //     Report Count............ (1)
            0x81, 0x02, //     Input...................(Data, Variable, Absolute)
            0xC0, //   End Collection
            0x05, 0x0D, //   Usage Page (Digitizer)
            0x09, 0x24, //   Usage (Gesture Character)
            0xA1, 0x02, //   Collection (Logical)
            0x05, 0x0D, //     Usage Page (Digitizer)
            0x09, 0x63, //     Usage (Gesture Character Data)
            0x75, 0x20, //     Report Size............. (32)
            0x95, 0x01, //     Report Count............ (1)
            0x82, 0x02, 0x01, //     Input...................(Data, Variable, Absolute, Buffered bytes)
            0x09, 0x65, //     Usage (Gesture Character Encoding UTF8)
            0x09, 0x62, //     Usage (Gesture Character Data Length)
            0x75, 0x08, //     Report Size............. (8)
            0x95, 0x02, //     Report Count............ (2)
            0x81, 0x02, //     Input...................(Data, Variable, Absolute)
            0x09, 0x61, //     Usage (Gesture Character Quality)
            0x15, 0x00, //     Logical Minimum......... (0)
            0x25, 0x64, //     Logical Maximum......... (100)
            0x75, 0x08, //     Report Size............. (8)
            0x95, 0x01, //     Report Count............ (1)
            0x81, 0x02, //     Input...................(Data, Variable, Absolute)
            0xC0, //   End Collection
            0xC0, // End Collection
        ];

        buf[39..41].copy_from_slice(&width.to_le_bytes());
        buf[44..46].copy_from_slice(&width_mm.to_le_bytes());
        buf[61..63].copy_from_slice(&height.to_le_bytes());
        buf[66..68].copy_from_slice(&height_mm.to_le_bytes());
        buf
    }
}

pub struct TouchpadButtonsReport {
    pub buttons: u8,
}

impl TouchpadButtonsReport {
    pub fn new(select_button: bool, back_button: bool, home_button: bool) -> Self {
        Self {
            buttons: (select_button as u8) | ((back_button as u8) << 1) | ((home_button as u8) << 2),
        }
    }

    pub fn to_bytes(&self) -> [u8; 1] {
        [self.buttons]
    }

    pub fn descriptor() -> [u8; 37] {
        [
            0x05, 0x0C, // Usage Page (Consumer)
            0x09, 0x01, // Usage (Consumer Control)
            0xA1, 0x01, // Collection (Application)
            0x05, 0x09, //   Usage Page (Button)
            0x09, 0x01, //   Usage (Button 1 primary/trigger)
            0x15, 0x00, //   Logical Minimum (0)
            0x25, 0x01, //   Logical Maximum (1)
            0x75, 0x01, //   Report Size (1)
            0x95, 0x01, //   Report Count (1)
            0x81, 0x02, //   Input (Data, Variable, Absolute)
            0x05, 0x0c, //   Usage Page (Consumer)
            0x0a, 0x24, 0x02, //   Usage (AC Back)
            0x0a, 0x23, 0x02, //   Usage (AC Home)
            0x95, 0x01, //   Report Count (2)
            0x81, 0x02, //   Input (Data, Variable, Absolute)
            0x95, 0x05, //   Report Size (5)
            0x81, 0x01, //   Input (Constant)
            0xC0, // End Collection
        ]
    }
}
