use std::fmt;

use macaddr::MacAddr6;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CarPlayCtrlInvite {
    pub mac: Option<MacAddr6>,
}

impl CarPlayCtrlInvite {
    pub fn new(mac: MacAddr6) -> Self {
        Self { mac: Some(mac) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AirPlayMacId(pub u64);

impl AirPlayMacId {
    pub fn as_mac(&self) -> MacAddr6 {
        (*self).into()
    }
}

impl fmt::Display for AirPlayMacId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<MacAddr6> for AirPlayMacId {
    fn from(value: MacAddr6) -> Self {
        let mut val: u64 = 0;
        for byte in value.as_bytes() {
            val = (val << 8) | (*byte as u64);
        }

        Self(val)
    }
}

impl From<AirPlayMacId> for MacAddr6 {
    fn from(value: AirPlayMacId) -> Self {
        let mut bytes = [0u8; 6];
        let mut v = value.0;

        for b in bytes.iter_mut().rev() {
            *b = (v & 0xFF) as u8;
            v >>= 8;
        }

        MacAddr6::from(bytes)
    }
}

impl TryFrom<&str> for AirPlayMacId {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let val: u64 = value.parse::<u64>().map_err(|_| "invalid number")?;
        Ok(AirPlayMacId(val))
    }
}

#[test]
#[cfg(test)]
fn test_ids() {
    use crate::ctrl::AirPlayMacId;

    assert_eq!(
        AirPlayMacId::from(MacAddr6::new(0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff)),
        AirPlayMacId(187723572702975)
    );
    assert_eq!(
        AirPlayMacId::from(MacAddr6::new(0x62, 0xd8, 0x9a, 0x72, 0xd7, 0x75)),
        AirPlayMacId(108682443675509)
    );
    assert_eq!(
        AirPlayMacId::from(MacAddr6::new(0x00, 0x11, 0x22, 0x33, 0x44, 0x55)),
        AirPlayMacId(73588229205)
    );

    assert_eq!(format!("{}", AirPlayMacId(73588229205)), "73588229205");
    assert_eq!(
        AirPlayMacId(73588229205).as_mac(),
        MacAddr6::new(0x00, 0x11, 0x22, 0x33, 0x44, 0x55)
    );
}
