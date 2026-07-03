use std::time::Duration;

use bitflags::bitflags;

#[derive(Debug, Clone, Copy)]
pub struct AudioStreamBasicDescription {
    pub sample_rate: u32,
    pub format: AudioCodec,
    pub format_flags: AudioFormatFlags,
    pub bytes_per_packet: u32,
    pub frames_per_packet: u32,
    pub bytes_per_frame: u32,
    pub channels_per_frame: u32,
    pub bits_per_channel: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCodec {
    LinearPcm,
    AppleLossless,
    Mpeg4Aac,
    Mpeg4AacEld,
    Opus,
}

impl AudioCodec {
    pub fn is_aac(&self) -> bool {
        matches!(self, AudioCodec::Mpeg4Aac | AudioCodec::Mpeg4AacEld)
    }

    pub fn is_opus(&self) -> bool {
        *self == AudioCodec::Opus
    }

    pub fn is_pcm(&self) -> bool {
        *self == AudioCodec::LinearPcm
    }

    pub fn is_alac(&self) -> bool {
        *self == AudioCodec::AppleLossless
    }
}

pub const AUDIO_SAMPLES_PER_PACKET_AAC_ELD: u32 = 480;
pub const AUDIO_SAMPLES_PER_PACKET_AAC_LC: u32 = 1024;
pub const AUDIO_MS_PER_PACKET_OPUS: u32 = 20;
pub const AUDIO_SAMPLES_PER_PACKET_MAX: u32 = 352;
pub const AUDIO_SAMPLES_PER_PACKET_ALAC: u32 = 352;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct AudioFormatFlags: u32 {
        const IS_FLOAT         = 1 << 0;
        const IS_BIG_ENDIAN    = 1 << 1;
        const IS_SIGNED_INT    = 1 << 2;
        const IS_PACKED        = 1 << 3;
        const IS_ALIGNED_HIGH  = 1 << 4;
        const IS_NON_INTERLEAVED = 1 << 5;
        const IS_NON_MIXABLE   = 1 << 6;
    }
}

#[cfg(target_endian = "big")]
pub const AUDIO_FORMAT_NATIVE_ENDIAN: AudioFormatFlags = AudioFormatFlags::IS_BIG_ENDIAN;

#[cfg(target_endian = "little")]
pub const AUDIO_FORMAT_NATIVE_ENDIAN: AudioFormatFlags = AudioFormatFlags::empty();

impl AudioStreamBasicDescription {
    pub fn with_sample_rate(&self, sample_rate: u32) -> AudioStreamBasicDescription {
        let mut me = *self;
        me.sample_rate = sample_rate;
        me
    }

    pub fn fill_pcm(rate: u32, valid_bits: u32, total_bits: u32, channels: u8, float: bool) -> Self {
        let bytes = channels as u32 * (total_bits / 8);
        let mut flags = AUDIO_FORMAT_NATIVE_ENDIAN;

        if float {
            flags |= AudioFormatFlags::IS_FLOAT
        } else {
            flags |= AudioFormatFlags::IS_SIGNED_INT
        };

        if valid_bits == total_bits {
            flags |= AudioFormatFlags::IS_PACKED;
        } else {
            flags |= AudioFormatFlags::IS_ALIGNED_HIGH;
        }

        Self {
            sample_rate: rate,
            format: AudioCodec::LinearPcm,
            format_flags: flags,
            bytes_per_packet: bytes,
            frames_per_packet: 1,
            bytes_per_frame: bytes,
            channels_per_frame: channels as u32,
            bits_per_channel: valid_bits,
        }
    }

    pub fn fill_aac_lc(rate: u32, channels: u32) -> Self {
        Self {
            sample_rate: rate,
            format: AudioCodec::Mpeg4Aac,
            format_flags: AudioFormatFlags::empty(),
            bytes_per_packet: 0,
            frames_per_packet: AUDIO_SAMPLES_PER_PACKET_AAC_LC,
            bytes_per_frame: 0,
            channels_per_frame: channels,
            bits_per_channel: 0,
        }
    }

    pub fn fill_aac_eld(rate: u32, channels: u32) -> Self {
        Self {
            sample_rate: rate,
            format: AudioCodec::Mpeg4AacEld,
            format_flags: AudioFormatFlags::empty(),
            bytes_per_packet: 0,
            frames_per_packet: AUDIO_SAMPLES_PER_PACKET_AAC_ELD,
            bytes_per_frame: 0,
            channels_per_frame: channels,
            bits_per_channel: 0,
        }
    }

    pub fn fill_opus(rate: u32, channels: u32) -> Self {
        let frames = (rate * AUDIO_MS_PER_PACKET_OPUS) / 1000;

        Self {
            sample_rate: rate,
            format: AudioCodec::Opus,
            format_flags: AudioFormatFlags::empty(),
            bytes_per_packet: 0,
            frames_per_packet: frames,
            bytes_per_frame: 0,
            channels_per_frame: channels,
            bits_per_channel: 0,
        }
    }

    pub fn fill_alac(rate: u32, bits_per_channel: u32, channels: u32) -> Self {
        Self {
            sample_rate: rate,
            format: AudioCodec::AppleLossless,
            format_flags: AudioFormatFlags::empty(),
            bytes_per_packet: 0,
            frames_per_packet: AUDIO_SAMPLES_PER_PACKET_ALAC,
            bytes_per_frame: 0,
            channels_per_frame: channels,
            bits_per_channel,
        }
    }
}

impl AudioStreamBasicDescription {
    pub fn ms_to_samples(&self, ms: Duration) -> u32 {
        let numer = ms.as_millis() as u32 * self.sample_rate;
        numer.div_ceil(1000)
    }

    pub fn samples_to_ms(&self, samples: u32) -> Duration {
        let total_nanos = (samples as u128) * 1_000_000_000u128 / (self.sample_rate as u128);
        Duration::from_nanos(total_nanos as u64)
    }

    pub fn channels(&self) -> u8 {
        match self.channels_per_frame {
            2 => 2,
            _ => 1,
        }
    }
}
