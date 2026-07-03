use std::{error::Error as StdError, str::Utf8Error};

use catplay_hap::cipher::HomeKitCipherError;
use catplay_util::ArcBox;

use crate::{msg::StreamType, rtsp_frame::HttpStatus};
use catplay_plist::PlistError;

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum RtspError {
    #[error("Generic error - unexpected state")]
    Unknown,

    #[error("Closing session now as requested interactively by the user")]
    UserClosing,

    #[error("HTTP status: {0}")]
    Code(HttpStatus),
    #[error("Unparsable plist: {0}")]
    UnparsablePlist(PlistError),
    #[error("Failed to serialize plist: {0}")]
    SerializationFailed(PlistError),

    #[error("Unexpected state: {0}")]
    UnexpectedState(String),

    #[error("Request queue has overflown")]
    Overflow,

    #[error("Channel was closed")]
    Closed,

    #[error("Server is busy")]
    Busy,

    #[error("Session teardown")]
    Teardown,

    #[error("Non-monotonic CSeq jump({0} vs last seen {1})")]
    CSeqSanity(u32, u32),

    #[error("Protocol violation")]
    ProtocolViolationGeneric,
    #[error("Protocol violation: {0}")]
    ProtocolViolationString(String),
    #[error("Protocol violation: {0}")]
    ProtocolViolation(&'static str),

    #[error("Overflow during headers streaming - headers too big")]
    HeadersTooBig,

    #[error("Received payload sized larger than the limit: {0} vs limit {1}")]
    PayloadTooBig(usize, usize),

    #[error("Received payload too short to decode: {0} vs minimum {1}")]
    PayloadTooShort(usize, usize),

    #[error("Closing channel now as requested")]
    DisconnectNow,

    #[error("I/O error: {0:?}")]
    Io(ArcBox<std::io::Error>),

    #[error("Request/channel timeout")]
    Timeout,

    #[error("Crypto error: {0}")]
    Crypto(#[from] HomeKitCipherError),

    #[error("This request requires an encrypted connection")]
    NotEncrypted,

    #[error("Operation not supported in this context")]
    NotSupported,

    #[error("Unicode violation: {0}")]
    Utf8Error(#[from] Utf8Error),

    #[error("Remote has stopped responding to keep alives")]
    KeepAliveTimeout,

    #[error("Expected payload to be non-empty")]
    Empty,

    #[error("Attempted to open conflicting stream: {0:?}")]
    StreamConflict(StreamType),

    #[error("RTP payload is too big to fit into UDP buffer: payload size {0} vs packet limit {0}")]
    RtpTooBigForBuffer(usize, usize),

    #[error("Invalid RTP header")]
    RtpInvalidHeader,

    #[error("Session has idled for too long without active media streams")]
    Idle,

    #[error("This connection does not support sending events towards the Controller")]
    EventsUnsupported,

    #[error("{0}")]
    Foreign(ArcBox<dyn StdError + Send + Sync>),
}

impl RtspError {
    pub fn foreign<T>(value: T) -> Self
    where
        T: StdError + Send + Sync + 'static,
    {
        RtspError::Foreign(ArcBox::from_box(Box::new(value)))
    }

    pub fn to_code(&self) -> HttpStatus {
        match self {
            RtspError::Code(code) => *code,
            RtspError::UnparsablePlist(_) => HttpStatus::BadRequest,
            RtspError::UnexpectedState(_) => HttpStatus::PreconditionFailed,
            RtspError::Busy => HttpStatus::NotEnoughBandwidth,
            RtspError::ProtocolViolationGeneric => HttpStatus::BadRequest,
            RtspError::ProtocolViolationString(_) => HttpStatus::BadRequest,
            RtspError::ProtocolViolation(_) => HttpStatus::BadRequest,
            RtspError::HeadersTooBig => HttpStatus::BadRequest,
            RtspError::PayloadTooBig(_, _) => HttpStatus::BadRequest,
            RtspError::PayloadTooShort(_, _) => HttpStatus::BadRequest,
            RtspError::NotEncrypted => HttpStatus::Forbidden,
            RtspError::Utf8Error(_) => HttpStatus::BadRequest,
            RtspError::Empty => HttpStatus::BadRequest,
            _ => HttpStatus::InternalServerError,
        }
    }
}

impl From<Box<dyn StdError + Send + Sync>> for RtspError {
    fn from(value: Box<dyn StdError + Send + Sync>) -> Self {
        RtspError::Foreign(ArcBox::from_box(value))
    }
}

impl From<ArcBox<dyn StdError + Send + Sync>> for RtspError {
    fn from(value: ArcBox<dyn StdError + Send + Sync>) -> Self {
        RtspError::Foreign(value)
    }
}

impl From<&str> for RtspError {
    fn from(value: &str) -> Self {
        RtspError::UnexpectedState(value.into())
    }
}

impl From<tokio::time::error::Elapsed> for RtspError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        Self::Timeout
    }
}

impl From<std::io::Error> for RtspError {
    fn from(value: std::io::Error) -> Self {
        RtspError::Io(ArcBox::new(value))
    }
}

impl From<HttpStatus> for RtspError {
    fn from(value: HttpStatus) -> Self {
        RtspError::Code(value)
    }
}

impl From<RtspError> for std::io::Error {
    fn from(value: RtspError) -> Self {
        std::io::Error::other(format!("{value}"))
    }
}

impl From<Option<RtspError>> for RtspError {
    fn from(value: Option<RtspError>) -> Self {
        match value {
            None => RtspError::Closed,
            Some(v) => v,
        }
    }
}

pub type RtspResult<T> = Result<T, RtspError>;
