use std::time::Duration;

use crate::video::AvccConfig;

#[derive(Debug, PartialEq, Clone)]
pub struct AvccConfigExtended {
    pub hevc: bool,
    pub avcc: AvccConfig,
    pub video_latency: Duration,
    pub width: u32,
    pub height: u32,
    pub respect_timestamps: bool,
    /// Where the decoded video belongs on the panel, when it is not the whole of it. Left `None`,
    /// the view parameters go out zeroed.
    pub view: Option<VideoView>,
}

/// Origin and size of the video within the panel, in pixels.
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct VideoView {
    pub origin_x: f32,
    pub origin_y: f32,
    pub width: f32,
    pub height: f32,
}
