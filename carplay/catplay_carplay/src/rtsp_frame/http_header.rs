use std::fmt;
use std::str::FromStr;

use crate::rtsp_frame::RtspString;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HttpHeader {
    Accept,
    AcceptLanguage,
    AcceptRanges,
    Authorization,
    Connection,
    ContentLength,
    ContentRange,
    ContentType,
    CSeq, // RTSP
    Date,
    Host,
    LastModified,
    Location,
    Public,
    Range,
    RTPInfo, // RTSP
    Session, // RTSP
    Server,
    TransferEncoding,
    Transport,
    Upgrade,
    UserAgent,
    WWWAuthenticate,

    // Apple headers
    AirPlayReceiverDeviceID,
    HomeKitPairing,
    EncryptionType,
    PairDerive,

    Other(RtspString),
}

impl HttpHeader {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Accept => "Accept",
            Self::AcceptLanguage => "Accept-Language",
            Self::AcceptRanges => "Accept-Ranges",
            Self::Authorization => "Authorization",
            Self::Connection => "Connection",
            Self::ContentLength => "Content-Length",
            Self::ContentRange => "Content-Range",
            Self::ContentType => "Content-Type",
            Self::CSeq => "CSeq",
            Self::Date => "Date",
            Self::Host => "Host",
            Self::LastModified => "Last-Modified",
            Self::Location => "Location",
            Self::Public => "Public",
            Self::Range => "Range",
            Self::RTPInfo => "RTP-Info",
            Self::Session => "Session",
            Self::Server => "Server",
            Self::TransferEncoding => "Transfer-Encoding",
            Self::Transport => "Transport",
            Self::Upgrade => "Upgrade",
            Self::UserAgent => "User-Agent",
            Self::WWWAuthenticate => "WWW-Authenticate",

            Self::AirPlayReceiverDeviceID => "AirPlay-Receiver-Device-ID",
            Self::HomeKitPairing => "X-Apple-HKP",
            Self::EncryptionType => "X-Apple-ET",
            Self::PairDerive => "X-Apple-PD",

            Self::Other(str) => str.as_str(),
        }
    }
}

impl fmt::Display for HttpHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for HttpHeader {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for HttpHeader {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "Accept" => Ok(Self::Accept),
            "Accept-Language" => Ok(Self::AcceptLanguage),
            "Accept-Ranges" => Ok(Self::AcceptRanges),
            "Authorization" => Ok(Self::Authorization),
            "Connection" => Ok(Self::Connection),
            "Content-Length" => Ok(Self::ContentLength),
            "Content-Range" => Ok(Self::ContentRange),
            "Content-Type" => Ok(Self::ContentType),
            "CSeq" => Ok(Self::CSeq),
            "Date" => Ok(Self::Date),
            "Host" => Ok(Self::Host),
            "Last-Modified" => Ok(Self::LastModified),
            "Location" => Ok(Self::Location),
            "Public" => Ok(Self::Public),
            "Range" => Ok(Self::Range),
            "RTP-Info" => Ok(Self::RTPInfo),
            "Session" => Ok(Self::Session),
            "Server" => Ok(Self::Server),
            "Transfer-Encoding" => Ok(Self::TransferEncoding),
            "Transport" => Ok(Self::Transport),
            "Upgrade" => Ok(Self::Upgrade),
            "User-Agent" => Ok(Self::UserAgent),
            "WWW-Authenticate" => Ok(Self::WWWAuthenticate),

            "AirPlay-Receiver-Device-ID" => Ok(Self::AirPlayReceiverDeviceID),
            "X-Apple-HKP" => Ok(Self::EncryptionType),
            "X-Apple-ET" => Ok(Self::EncryptionType),
            "X-Apple-PD" => Ok(Self::PairDerive),

            v => Ok(Self::Other(v.to_string().into())),
            // _ => Err(()),
        }
    }
}
