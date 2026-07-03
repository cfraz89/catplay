use crate::enum_type;

enum_type! {
    pub enum PowerProvidingCapability {
        None = 0,
        Reserved = 1,
        Advanced = 2
    }
}

enum_type! {
    pub enum MatchAction {
        NoAction = 0,
        OptionalAction = 1,
        NoAlert = 2,
        NoCommProto = 3
    }
}

enum_type! {
    pub enum USBDeviceModeAudioSampleRate {
        Hz8000 = 0,
        Hz11025 = 1,
        Hz12000 = 2,
        Hz16000 = 3,
        Hz22050 = 4,
        Hz24000 = 5,
        Hz32000 = 6,
        Hz44100 = 7,
        Hz48000 = 8,
    }
}

enum_type! {
    pub enum EngineTypes {
        Gasoline = 0,
        Diesel = 1,
        Electric = 2,
        CNG = 3
    }
}

enum_type! {
    pub enum HIDComponentFunction {
        Keyboard = 0,
        MediaPlaybackRemote = 1,
        AssistiveTouchPointer = 2,
        Reserved = 3,
        GamepadFF = 4,
        GamepadNFF = 6,
        AssistiveSwitchControl = 7,
        Headset = 8
    }
}
