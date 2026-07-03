use core::fmt;

use crate::rtsp_frame::{RtspRequest, RtspResponse};

#[derive(Debug, Clone)]
pub enum RtspFrame {
    Request(RtspRequest),
    Response(RtspResponse),
}

impl From<RtspRequest> for RtspFrame {
    fn from(value: RtspRequest) -> Self {
        RtspFrame::Request(value)
    }
}

impl From<RtspResponse> for RtspFrame {
    fn from(value: RtspResponse) -> Self {
        RtspFrame::Response(value)
    }
}

impl fmt::Display for RtspFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RtspFrame::Request(r) => r.fmt(f),
            RtspFrame::Response(r) => r.fmt(f),
        }
    }
}
