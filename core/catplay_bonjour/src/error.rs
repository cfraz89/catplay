use std::{fmt, io};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MulticastFamily {
    Ipv4,
    Ipv6,
}

impl fmt::Display for MulticastFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipv4 => write!(f, "IPv4"),
            Self::Ipv6 => write!(f, "IPv6"),
        }
    }
}

#[derive(Debug)]
pub enum BonjourError {
    MulticastUnstable {
        iface: String,
        family: MulticastFamily,
        source: io::Error,
    },
    Io(io::Error),
}

impl fmt::Display for BonjourError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MulticastUnstable { iface, family, source } => {
                write!(f, "mDNS multicast is unstable on interface {iface} ({family}): {source}")
            }
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for BonjourError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MulticastUnstable { source, .. } => Some(source),
            Self::Io(err) => Some(err),
        }
    }
}

impl From<io::Error> for BonjourError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<BonjourError> for io::Error {
    fn from(value: BonjourError) -> Self {
        let kind = match &value {
            BonjourError::MulticastUnstable { source, .. } => source.kind(),
            BonjourError::Io(err) => err.kind(),
        };
        io::Error::new(kind, value)
    }
}
