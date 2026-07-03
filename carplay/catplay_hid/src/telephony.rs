#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TelephonyButton {
    Up,
    HookSwitch,
    Flash,
    Drop,
    Mute,
    PhoneKey0,
    PhoneKey1,
    PhoneKey2,
    PhoneKey3,
    PhoneKey4,
    PhoneKey5,
    PhoneKey6,
    PhoneKey7,
    PhoneKey8,
    PhoneKey9,
    PhoneKeyStar,
    PhoneKeyPound,
    Delete,
}

impl From<TelephonyButton> for u8 {
    fn from(value: TelephonyButton) -> Self {
        value as u8
    }
}

impl From<u8> for TelephonyButton {
    fn from(value: u8) -> Self {
        match value {
            0 => TelephonyButton::Up,
            1 => TelephonyButton::HookSwitch,
            2 => TelephonyButton::Flash,
            3 => TelephonyButton::Drop,
            4 => TelephonyButton::Mute,
            5 => TelephonyButton::PhoneKey0,
            6 => TelephonyButton::PhoneKey1,
            7 => TelephonyButton::PhoneKey2,
            8 => TelephonyButton::PhoneKey3,
            9 => TelephonyButton::PhoneKey4,
            10 => TelephonyButton::PhoneKey5,
            11 => TelephonyButton::PhoneKey6,
            12 => TelephonyButton::PhoneKey7,
            13 => TelephonyButton::PhoneKey8,
            14 => TelephonyButton::PhoneKey9,
            15 => TelephonyButton::PhoneKeyStar,
            16 => TelephonyButton::PhoneKeyPound,
            17 => TelephonyButton::Delete,
            _ => TelephonyButton::Up, // Fallback
        }
    }
}

pub struct TelephonyButtonsReport {
    button: u8,
}

impl TelephonyButtonsReport {
    pub fn new(button: TelephonyButton) -> Self {
        Self { button: button.into() }
    }

    pub fn to_bytes(&self) -> [u8; 1] {
        [self.button]
    }

    pub fn descriptor() -> [u8; 57] {
        [
            0x05, 0x0B, // Usage Page (Telephony)
            0x09, 0x07, // Usage (Telephony Keypad)
            0xA1, 0x01, // Collection (Application)
            0x15, 0x00, // Logical min
            0x25, 0x11, // logical max
            0x05, 0x0B, // Usage Page (Telephony)
            0x09, 0x00, // Usage (Unassigned)
            0x09, 0x20, // Usage (Hook Switch)
            0x09, 0x21, // Usage (Flash)
            0x09, 0x26, // Usage (Drop)
            0x09, 0x2F, // Usage (Mute)
            0x09, 0xB0, // Usage (PhoneKey 0)
            0x09, 0xB1, // Usage (PhoneKey 1)
            0x09, 0xB2, // Usage (PhoneKey 2)
            0x09, 0xB3, // Usage (PhoneKey 3)
            0x09, 0xB4, // Usage (PhoneKey 4)
            0x09, 0xB5, // Usage (PhoneKey 5)
            0x09, 0xB6, // Usage (PhoneKey 6)
            0x09, 0xB7, // Usage (PhoneKey 7)
            0x09, 0xB8, // Usage (PhoneKey 8)
            0x09, 0xB9, // Usage (PhoneKey 9)
            0x09, 0xBA, // Usage (PhoneKey Star)
            0x09, 0xBB, // Usage (PhoneKey Pound)
            0x05, 0x07, // Usage Page (Keyboard/Keypad)
            0x09, 0x2A, // Usage (Keyboard DELETE)
            0x75, 0x08, // Report Size (8)
            0x95, 0x01, // Report Count (1)
            0x81, 0x00, // Input (Data, Array, Absolute)
            0xC0, // End collection
        ]
    }
}
