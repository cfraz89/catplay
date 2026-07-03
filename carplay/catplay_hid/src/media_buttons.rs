#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MediaButton {
    None,
    Play,
    Pause,
    PlayPause,
    NextTrack,
    PrevTrack,
    ACNavigationGuidance,
}

impl From<MediaButton> for u8 {
    fn from(value: MediaButton) -> Self {
        match value {
            MediaButton::None => 0,
            MediaButton::Play => 1,
            MediaButton::Pause => 2,
            MediaButton::PlayPause => 3,
            MediaButton::NextTrack => 4,
            MediaButton::PrevTrack => 5,
            MediaButton::ACNavigationGuidance => 6,
        }
    }
}

impl From<u8> for MediaButton {
    fn from(value: u8) -> Self {
        match value {
            1 => MediaButton::Play,
            2 => MediaButton::Pause,
            3 => MediaButton::PlayPause,
            4 => MediaButton::NextTrack,
            5 => MediaButton::PrevTrack,
            6 => MediaButton::ACNavigationGuidance,
            _ => MediaButton::None,
        }
    }
}

pub struct MediaButtonsReport {
    button: u8,
}

impl MediaButtonsReport {
    pub fn new(button: MediaButton) -> Self {
        Self { button: button.into() }
    }

    pub fn to_bytes(&self) -> [u8; 1] {
        [self.button]
    }

    pub fn descriptor() -> [u8; 40] {
        [
            0x05, 0x0C, // Usage Page (Consumer)
            0x09, 0x01, // Usage 1 (Consumer Control)
            0xA1, 0x01, // Collection (Application)
            0x15, 0x00, // Logical Minimum......... (0)
            0x25, 0x06, // Logical Maximum......... (6)
            0x05, 0x0C, // Usage Page (Consumer)
            0x0A, 0x00, 0x00, // Usage 0 (0x0) 		// Unassigned
            0x0A, 0xB0, 0x00, // Usage 176 (0xb0) 	// Play
            0x0A, 0xB1, 0x00, // Usage 177 (0xb1) 	// Pause
            0x0A, 0xCD, 0x00, // Usage 205 (0xcd) 	// Play / Pause
            0x0A, 0xB5, 0x00, // Usage 181 (0xb5) 	// Scan Next Track
            0x0A, 0xB6, 0x00, // Usage 182 (0xb6) 	// Scan Previous Track
            0x0A, 0x9E, 0x02, // Usage 670 (0x29e)	// AC Navigation Guidance
            0x75, 0x08, // Report Size............. (8)
            0x95, 0x01, // Report Count............ (1)
            0x81, 0x00, // Input...................(Data, Array, Absolute)
            0xC0, // End Collection
        ]
    }
}
