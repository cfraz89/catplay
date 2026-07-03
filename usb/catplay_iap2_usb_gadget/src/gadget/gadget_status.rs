use std::fmt;

use catplay_iap2_usb::GadgetError;

#[derive(Debug, Clone, PartialEq)]
pub enum GadgetStatus {
    Initial,
    Bind,
    Enabled,
    Disabled,
    Suspended,

    // Final states
    Unbind,
    Shutdown,
    Error(GadgetError),
}

impl Default for GadgetStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl GadgetStatus {
    pub fn new() -> Self {
        GadgetStatus::Initial
    }

    pub fn is_final(&self) -> bool {
        matches!(self, GadgetStatus::Unbind | GadgetStatus::Shutdown | GadgetStatus::Error(_))
    }

    pub fn as_str(&self) -> String {
        format!("{self:?}")
    }
}

impl fmt::Display for GadgetStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
