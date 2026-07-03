use catplay_plist::{plist_bitflags, plist_enum, plist_enum_repr};

plist_enum_repr! {
    #[repr(u8)]
    pub enum StreamType {
        #[default]
        Invalid = 0,

        GeneralAudio = 96,

        MainAudio = 100,
        AltAudio = 101,
        MainHighAudio = 102,
        Screen = 110,

        // Modern CarPlay
        /*BufferedAudio = 103,
        AuxOutAudio = 106,
        AuxInAudio = 107,
        AltScreen = 111,
        MainAudioWithRedundancy = 104,
        AltAudioWithRedundancy = 105,
        AuxOutAudioWithRedundancy = 108,
        AuxInAudioWithRedundancy = 109*/
    }
}

impl StreamType {
    pub fn supports_legacy_audio_aes(&self) -> bool {
        matches!(
            self,
            StreamType::GeneralAudio | StreamType::MainHighAudio /* | StreamType::BufferedAudio */
        )
    }
}

plist_enum! {
    pub enum AudioType {
        /// Main Audio, Alt Audio
        #[default]
        Default,
        /// Main Audio
        Alert,
        /// Main Audio, Main High Audio
        Media,
        /// Main Audio
        Telephony,
        /// Main Audio
        SpeechRecognition,
        /// Main Audio, Alt Audio
        Compatibility
    }
}

plist_bitflags! {
    pub struct AudioFormat: u64 {
        /// PCM, 8000 Hz, 16-Bit, Mono
        const PCM_8000_MONO     = 1 << 2;
        /// PCM, 8000 Hz, 16-Bit, Stereo
        const PCM_8000_STEREO   = 1 << 3;
        /// PCM, 16000 Hz, 16-Bit, Mono
        const PCM_16000_MONO    = 1 << 4;
        /// PCM, 16000 Hz, 16-Bit, Stereo
        const PCM_16000_STEREO  = 1 << 5;
        /// PCM, 24000 Hz, 16-Bit, Mono
        const PCM_24000_MONO    = 1 << 6;
        /// PCM, 24000 Hz, 16-Bit, Stereo
        const PCM_24000_STEREO  = 1 << 7;
        /// PCM, 32000 Hz, 16-Bit, Mono
        const PCM_32000_MONO    = 1 << 8;
        /// PCM, 32000 Hz, 16-Bit, Stereo
        const PCM_32000_STEREO  = 1 << 9;
        /// PCM, 44100 Hz, 16-Bit, Mono
        const PCM_44100_MONO    = 1 << 10;
        /// PCM, 44100 Hz, 16-Bit, Stereo
        const PCM_44100_STEREO  = 1 << 11;
        /// PCM, 44100 Hz, 24-Bit, Mono
        const PCM_44100_24_MONO = 1 << 12;
        /// PCM, 44100 Hz, 24-Bit, Stereo
        const PCM_44100_24_STEREO = 1 << 13;
        /// PCM, 48000 Hz, 16-Bit, Mono
        const PCM_48000_MONO    = 1 << 14;
        /// PCM, 48000 Hz, 16-Bit, Stereo
        const PCM_48000_STEREO  = 1 << 15;
        /// PCM, 48000 Hz, 24-Bit, Mono
        const PCM_48000_24_MONO = 1 << 16;
        /// PCM, 48000 Hz, 24-Bit, Stereo
        const PCM_48000_24_STEREO = 1 << 17;

        /// ALAC, 44100 Hz, 16-Bit, Stereo
        const ALAC_44100_16_STEREO = 1 << 18;
        /// ALAC, 44100 Hz, 24-Bit, Stereo
        const ALAC_44100_24_STEREO = 1 << 19;
        /// ALAC, 48000 Hz, 16-Bit, Stereo
        const ALAC_48000_16_STEREO = 1 << 20;
        /// ALAC, 48000 Hz, 24-Bit, Stereo
        const ALAC_48000_24_STEREO = 1 << 21;

        /// AAC-LC, 44100 Hz, Stereo
        const AAC_LC_44100_STEREO = 1 << 22;
        /// AAC-LC, 48000 Hz, Stereo
        const AAC_LC_48000_STEREO = 1 << 23;

        /// AAC-ELD, 44100 Hz, Stereo
        const AAC_ELD_44100_STEREO = 1 << 24;
        /// AAC-ELD, 48000 Hz, Stereo
        const AAC_ELD_48000_STEREO = 1 << 25;
        /// AAC-ELD, 16000 Hz, Mono
        const AAC_ELD_16000_MONO   = 1 << 26;
        /// AAC-ELD, 24000 Hz, Mono
        const AAC_ELD_24000_MONO   = 1 << 27;

        /// OPUS, 16000 Hz, Mono
        const OPUS_16000_MONO = 1 << 28;
        /// OPUS, 24000 Hz, Mono
        const OPUS_24000_MONO = 1 << 29;
        /// OPUS, 48000 Hz, Mono
        const OPUS_48000_MONO = 1 << 30;

        /// AAC-ELD, 44100 Hz, Mono
        const AAC_ELD_44100_MONO = 1 << 31;
        /// AAC-ELD, 48000 Hz, Mono
        const AAC_ELD_48000_MONO = 1 << 32;
    }
}
