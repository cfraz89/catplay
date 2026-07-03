use catplay_audio::{AudioCodec, AudioStreamBasicDescription};

use crate::msg::AudioFormat;

fn airplay_format_to_asbd(fmt: AudioFormat) -> Option<AudioStreamBasicDescription> {
    Some(match fmt {
        AudioFormat::PCM_8000_MONO => AudioStreamBasicDescription::fill_pcm(8000, 16, 16, 1, false),
        AudioFormat::PCM_8000_STEREO => AudioStreamBasicDescription::fill_pcm(8000, 16, 16, 2, false),
        AudioFormat::PCM_16000_MONO => AudioStreamBasicDescription::fill_pcm(16000, 16, 16, 1, false),
        AudioFormat::PCM_16000_STEREO => AudioStreamBasicDescription::fill_pcm(16000, 16, 16, 2, false),
        AudioFormat::PCM_24000_MONO => AudioStreamBasicDescription::fill_pcm(24000, 16, 16, 1, false),
        AudioFormat::PCM_24000_STEREO => AudioStreamBasicDescription::fill_pcm(24000, 16, 16, 2, false),
        AudioFormat::PCM_32000_MONO => AudioStreamBasicDescription::fill_pcm(32000, 16, 16, 1, false),
        AudioFormat::PCM_32000_STEREO => AudioStreamBasicDescription::fill_pcm(32000, 16, 16, 2, false),
        AudioFormat::PCM_44100_MONO => AudioStreamBasicDescription::fill_pcm(44100, 16, 16, 1, false),
        AudioFormat::PCM_44100_STEREO => AudioStreamBasicDescription::fill_pcm(44100, 16, 16, 2, false),
        AudioFormat::PCM_44100_24_MONO => AudioStreamBasicDescription::fill_pcm(44100, 24, 24, 1, false),
        AudioFormat::PCM_44100_24_STEREO => AudioStreamBasicDescription::fill_pcm(44100, 24, 24, 2, false),
        AudioFormat::PCM_48000_MONO => AudioStreamBasicDescription::fill_pcm(48000, 16, 16, 1, false),
        AudioFormat::PCM_48000_STEREO => AudioStreamBasicDescription::fill_pcm(48000, 16, 16, 2, false),
        AudioFormat::PCM_48000_24_MONO => AudioStreamBasicDescription::fill_pcm(48000, 24, 24, 1, false),
        AudioFormat::PCM_48000_24_STEREO => AudioStreamBasicDescription::fill_pcm(48000, 24, 24, 2, false),

        AudioFormat::ALAC_44100_16_STEREO => AudioStreamBasicDescription::fill_alac(44100, 16, 2),
        AudioFormat::ALAC_44100_24_STEREO => AudioStreamBasicDescription::fill_alac(44100, 24, 2),
        AudioFormat::ALAC_48000_16_STEREO => AudioStreamBasicDescription::fill_alac(48000, 16, 2),
        AudioFormat::ALAC_48000_24_STEREO => AudioStreamBasicDescription::fill_alac(48000, 24, 2),

        AudioFormat::AAC_LC_44100_STEREO => AudioStreamBasicDescription::fill_aac_lc(44100, 2),
        AudioFormat::AAC_LC_48000_STEREO => AudioStreamBasicDescription::fill_aac_lc(48000, 2),

        AudioFormat::AAC_ELD_16000_MONO => AudioStreamBasicDescription::fill_aac_eld(16000, 1),
        AudioFormat::AAC_ELD_24000_MONO => AudioStreamBasicDescription::fill_aac_eld(24000, 1),
        AudioFormat::AAC_ELD_44100_STEREO => AudioStreamBasicDescription::fill_aac_eld(44100, 2),
        AudioFormat::AAC_ELD_48000_STEREO => AudioStreamBasicDescription::fill_aac_eld(48000, 2),
        AudioFormat::AAC_ELD_44100_MONO => AudioStreamBasicDescription::fill_aac_eld(44100, 1),
        AudioFormat::AAC_ELD_48000_MONO => AudioStreamBasicDescription::fill_aac_eld(48000, 1),

        AudioFormat::OPUS_16000_MONO => AudioStreamBasicDescription::fill_opus(16000, 1),
        AudioFormat::OPUS_24000_MONO => AudioStreamBasicDescription::fill_opus(24000, 1),
        AudioFormat::OPUS_48000_MONO => AudioStreamBasicDescription::fill_opus(48000, 1),

        _ => return None,
    })
}

fn asbd_to_airplay_format(asbd: AudioStreamBasicDescription) -> Option<AudioFormat> {
    use AudioCodec::*;

    match (asbd.format, asbd.sample_rate, asbd.channels_per_frame, asbd.bits_per_channel) {
        (LinearPcm, 8000, 1, 16) => Some(AudioFormat::PCM_8000_MONO),
        (LinearPcm, 8000, 2, 16) => Some(AudioFormat::PCM_8000_STEREO),
        (LinearPcm, 16000, 1, 16) => Some(AudioFormat::PCM_16000_MONO),
        (LinearPcm, 16000, 2, 16) => Some(AudioFormat::PCM_16000_STEREO),
        (LinearPcm, 24000, 1, 16) => Some(AudioFormat::PCM_24000_MONO),
        (LinearPcm, 24000, 2, 16) => Some(AudioFormat::PCM_24000_STEREO),
        (LinearPcm, 32000, 1, 16) => Some(AudioFormat::PCM_32000_MONO),
        (LinearPcm, 32000, 2, 16) => Some(AudioFormat::PCM_32000_STEREO),
        (LinearPcm, 44100, 1, 16) => Some(AudioFormat::PCM_44100_MONO),
        (LinearPcm, 44100, 2, 16) => Some(AudioFormat::PCM_44100_STEREO),
        (LinearPcm, 44100, 1, 24) => Some(AudioFormat::PCM_44100_24_MONO),
        (LinearPcm, 44100, 2, 24) => Some(AudioFormat::PCM_44100_24_STEREO),
        (LinearPcm, 48000, 1, 16) => Some(AudioFormat::PCM_48000_MONO),
        (LinearPcm, 48000, 2, 16) => Some(AudioFormat::PCM_48000_STEREO),
        (LinearPcm, 48000, 1, 24) => Some(AudioFormat::PCM_48000_24_MONO),
        (LinearPcm, 48000, 2, 24) => Some(AudioFormat::PCM_48000_24_STEREO),
        (AppleLossless, 44100, 2, 16) => Some(AudioFormat::ALAC_44100_16_STEREO),
        (AppleLossless, 44100, 2, 24) => Some(AudioFormat::ALAC_44100_24_STEREO),
        (AppleLossless, 48000, 2, 16) => Some(AudioFormat::ALAC_48000_16_STEREO),
        (AppleLossless, 48000, 2, 24) => Some(AudioFormat::ALAC_48000_24_STEREO),
        (Mpeg4Aac, 44100, 2, _) => Some(AudioFormat::AAC_LC_44100_STEREO),
        (Mpeg4Aac, 48000, 2, _) => Some(AudioFormat::AAC_LC_48000_STEREO),
        (Mpeg4AacEld, 16000, 1, _) => Some(AudioFormat::AAC_ELD_16000_MONO),
        (Mpeg4AacEld, 24000, 1, _) => Some(AudioFormat::AAC_ELD_24000_MONO),
        (Mpeg4AacEld, 44100, 1, _) => Some(AudioFormat::AAC_ELD_44100_MONO),
        (Mpeg4AacEld, 44100, 2, _) => Some(AudioFormat::AAC_ELD_44100_STEREO),
        (Mpeg4AacEld, 48000, 1, _) => Some(AudioFormat::AAC_ELD_48000_MONO),
        (Mpeg4AacEld, 48000, 2, _) => Some(AudioFormat::AAC_ELD_48000_STEREO),
        (Opus, 16000, 1, _) => Some(AudioFormat::OPUS_16000_MONO),
        (Opus, 24000, 1, _) => Some(AudioFormat::OPUS_24000_MONO),
        (Opus, 48000, 1, _) => Some(AudioFormat::OPUS_48000_MONO),
        _ => None,
    }
}

impl AudioFormat {
    pub fn as_asbd(&self) -> Option<AudioStreamBasicDescription> {
        airplay_format_to_asbd(*self)
    }
}

impl TryFrom<AudioFormat> for AudioStreamBasicDescription {
    type Error = &'static str;

    fn try_from(value: AudioFormat) -> Result<Self, Self::Error> {
        airplay_format_to_asbd(value).ok_or("unknown stream type")
    }
}

impl TryFrom<AudioStreamBasicDescription> for AudioFormat {
    type Error = &'static str;

    fn try_from(value: AudioStreamBasicDescription) -> Result<Self, Self::Error> {
        asbd_to_airplay_format(value).ok_or("unknown stream type")
    }
}
