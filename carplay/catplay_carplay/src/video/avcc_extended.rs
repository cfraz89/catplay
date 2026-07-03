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
}
