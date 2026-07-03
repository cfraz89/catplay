use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RtspMethod {
    Announce,
    Setup,
    Record,
    Pause,
    Flush,
    FlushBuffered,
    Teardown,
    Options,

    GetParameter,
    SetParameter,

    SetRateAnchorTime,
    SetPeers,

    Unknown,

    // HTTP compat
    Get,
    Post,
    Put,
    Delete,
}

impl AsRef<str> for RtspMethod {
    fn as_ref(&self) -> &'static str {
        match self {
            RtspMethod::Setup => "SETUP",
            RtspMethod::Record => "RECORD",
            RtspMethod::Flush => "FLUSH",
            RtspMethod::FlushBuffered => "FLUSHBUFFERED",
            RtspMethod::Teardown => "TEARDOWN",
            RtspMethod::Options => "OPTIONS",
            RtspMethod::GetParameter => "GET_PARAMETER",
            RtspMethod::SetParameter => "SET_PARAMETER",
            RtspMethod::SetRateAnchorTime => "SETRATEANCHORTIME",
            RtspMethod::SetPeers => "SETPEERS",
            RtspMethod::Unknown => "UNKNOWN",
            RtspMethod::Get => "GET",
            RtspMethod::Post => "POST",
            RtspMethod::Announce => "ANNOUNCE",
            RtspMethod::Pause => "PAUSE",
            RtspMethod::Put => "PUT",
            RtspMethod::Delete => "DELETE",
        }
    }
}

impl FromStr for RtspMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ret = match s {
            "SETUP" => RtspMethod::Setup,
            "RECORD" => RtspMethod::Record,
            "FLUSH" => RtspMethod::Flush,
            "FLUSHBUFFERED" => RtspMethod::FlushBuffered,
            "TEARDOWN" => RtspMethod::Teardown,
            "OPTIONS" => RtspMethod::Options,
            "GET_PARAMETER" => RtspMethod::GetParameter,
            "SET_PARAMETER" => RtspMethod::SetParameter,
            "SETRATEANCHORTIME" => RtspMethod::SetRateAnchorTime,
            "SETPEERS" => RtspMethod::SetPeers,
            "UNKNOWN" => RtspMethod::Unknown,
            "GET" => RtspMethod::Get,
            "POST" => RtspMethod::Post,
            "ANNOUNCE" => RtspMethod::Announce,
            "PAUSE" => RtspMethod::Pause,
            "PUT" => RtspMethod::Put,
            "DELETE" => RtspMethod::Delete,

            _ => return Err(()),
        };
        Ok(ret)
    }
}

impl fmt::Display for RtspMethod {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_ref())
    }
}
