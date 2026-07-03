use std::{error::Error, fmt::Debug, io};

use catplay_util::ArcBox;

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum GadgetError {
    #[error("I/O error: {0}")]
    Io(ArcBox<io::Error>),
    #[error("USB error: {0}")]
    Usb(#[from] rusb::Error),

    #[error("Operation timeout")]
    Timeout,

    #[error("Operation requires a working UDC controller which is missing on this system")]
    MissingUdc,

    #[error("Operation unsupported with this device/context")]
    Unsupported,

    #[error("NCM network interface disappeared from the system")]
    NcmInterfaceDisappeared,

    #[error("NCM network interface failed to bind({usb_iface} @ {bind_path}): {error}")]
    NcmInterfaceFailedToBind {
        usb_iface: String,
        bind_path: String,
        error: ArcBox<io::Error>,
    },

    #[error("The system is missing cdc_ncm kernel driver")]
    NcmDriverMissing,

    #[error("Failed to perform ip link command: {0}")]
    FailedIpLinkSetup(String),

    #[error("{0}")]
    Other(ArcBox<dyn Error + Send + Sync>),

    #[error("{0}")]
    OtherString(String),

    #[error("Gadget register returned error(missing kernel modules/root access?): {0}")]
    FailedGadgetRegister(ArcBox<io::Error>),

    #[error("Failed OTG borrow for UDC {udc}: {err}")]
    FailedOtgBorrow { udc: String, err: ArcBox<io::Error> },

    #[error("Failed OTG restore for UDC {udc}: {err}")]
    FailedOtgRestore { udc: String, err: ArcBox<io::Error> },

    #[error("Unexpected OTG system state for UDC {udc}")]
    FailedOtg { udc: String },

    #[error("Failed to inspect UDC state: {0}")]
    FailedUdcStatusCheck(ArcBox<io::Error>),

    #[error("Failed to modify/inspect gadget state: {0}")]
    FailedGadgetStatusCheck(ArcBox<io::Error>),

    #[error("Expected UDC '{udc}' to be binding to function '{expected}', instead found '{found}'")]
    UnexpectedGadgetFunction { udc: String, expected: String, found: String },
}

pub type GadgetResult<T> = Result<T, GadgetError>;

impl From<&str> for GadgetError {
    fn from(s: &str) -> Self {
        GadgetError::OtherString(s.to_string())
    }
}

impl From<String> for GadgetError {
    fn from(s: String) -> Self {
        GadgetError::OtherString(s)
    }
}

impl From<io::Error> for GadgetError {
    fn from(s: io::Error) -> Self {
        GadgetError::Io(s.into())
    }
}
