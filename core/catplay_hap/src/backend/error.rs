use core::{num, str};
use thiserror::Error;

/// HAP error representation.
#[derive(Debug, Error)]
pub enum Error {
    // converted errors
    // #[error("IO Error: {0}")]
    // Io(#[from] io::Error),
    // #[error("AEAD Error")]
    // Aead,
    #[error("HKDF Invalid Length Error")]
    HkdfInvalidLength,
    #[error("UTF-8 Error: {0}")]
    Utf8(#[from] str::Utf8Error),
    // #[error("Parse EUI-48 Error: {0}")]
    // ParseEui48(#[from] macaddr::ParseError),
    #[error("Parse Int Error: {0}")]
    ParseInt(#[from] num::ParseIntError),
}
