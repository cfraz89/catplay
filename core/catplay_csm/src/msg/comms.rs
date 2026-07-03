use crate::{decoder::*, enum_type, packet_type};

packet_type! {
    pub struct StartCallStateUpdates: 0x4154 {
        #[csm_id(0)]  pub remote_id: CsmFlag,
        #[csm_id(1)]  pub display_name: CsmFlag,
        #[csm_id(2)]  pub status: CsmFlag,
        #[csm_id(3)]  pub direction: CsmFlag,
        #[csm_id(4)]  pub call_uuid: CsmFlag,
        #[csm_id(6)]  pub address_book_id: CsmFlag,
        #[csm_id(7)]  pub label: CsmFlag,
        #[csm_id(8)]  pub service: CsmFlag,
        #[csm_id(9)]  pub is_conferenced: CsmFlag,
        #[csm_id(10)] pub conference_group: CsmFlag,
        #[csm_id(11)] pub disconnect_reason: CsmFlag,
        #[csm_id(12)] pub start_timestamp: CsmFlag,

    }
}

packet_type! {
    pub struct CallStateUpdate: 0x4155 {
        #[csm_id(0)]  pub remote_id: Option<CsmString>,
        #[csm_id(1)]  pub display_name: Option<CsmString>,
        #[csm_id(2)]  pub status: CallStateUpdateStatus,
        #[csm_id(3)]  pub direction: Option<CallStateUpdateDirection>,
        #[csm_id(4)]  pub call_uuid: Option<CsmString>,
        #[csm_id(6)]  pub address_book_id: Option<CsmString>,
        #[csm_id(7)]  pub label: Option<CsmString>,
        #[csm_id(8)]  pub service: Option<CallStateUpdateService>,
        #[csm_id(9)]  pub is_conferenced: Option<bool>,
        #[csm_id(10)] pub conference_group: Option<u8>,
        #[csm_id(11)] pub disconnect_reason: Option<CallStateUpdateDisconnectReason>,
        #[csm_id(12)] pub start_seconds_since_epoch: Option<u64>,
    }
}

enum_type! {
    pub enum CallStateUpdateStatus {
        Disconnected = 0,
        Sending = 1,
        Ringing = 2,
        Connecting = 3,
        Active = 4,
        Held = 5,
        Disconnecting = 6
    }
}

enum_type! {
    pub enum CallStateUpdateDirection {
        Unknown = 0,
        Incoming = 1,
        Outgoing = 2
    }
}

enum_type! {
    pub enum CallStateUpdateService {
        Unknown = 0,
        Telephony = 1,
        FaceTimeAudio = 2,
        FaceTimeVideo = 3
    }
}

enum_type! {
    pub enum CallStateUpdateDisconnectReason {
        NoReason = 0,
        CallDeclined = 1,
        CallFailed = 2
    }
}

enum_type! {
    pub enum CallStateUpdateStatusLegacy {
        Disconnected = 0,
        Active = 1,
        Held = 2,
        RingingSending = 3
    }
}

enum_type! {
    pub enum CallStateUpdateDirectionLegacy {
        Incoming = 0,
        Outgoing = 1,
        Unknown = 2
    }
}

packet_type! {
    pub struct StopCallStateUpdates: 0x4156 {
    }
}

packet_type! {
    pub struct StartCommunicationsUpdates: 0x4157 {
        #[csm_id(0)]  pub signal_strength: CsmFlag,
        #[csm_id(1)]  pub registration_status: CsmFlag,
        #[csm_id(2)]  pub airplane_mode_status: CsmFlag,
        #[csm_id(4)]  pub carrier_name: CsmFlag,
        #[csm_id(5)]  pub cellular_supported: CsmFlag,
        #[csm_id(6)]  pub telephony_enabled: CsmFlag,
        #[csm_id(7)]  pub facetime_audio_enabled: CsmFlag,
        #[csm_id(8)]  pub facetime_video_enabled: CsmFlag,
        #[csm_id(9)]  pub mute_status: CsmFlag,

        #[csm_id(10)] pub current_call_count: CsmFlag,
        #[csm_id(11)] pub new_voicemail_count: CsmFlag,
        #[csm_id(12)] pub initiate_call_available: CsmFlag,
        #[csm_id(13)] pub end_and_accept_available: CsmFlag,
        #[csm_id(14)] pub hold_and_accept_available: CsmFlag,
        #[csm_id(15)] pub swap_available: CsmFlag,
        #[csm_id(16)] pub merge_available: CsmFlag,
        #[csm_id(17)] pub hold_available: CsmFlag,
    }
}

enum_type! {
    pub enum CommunicationsUpdateSignalStrength {
        Bars0 = 0,
        Bars1 = 1,
        Bars2 = 2,
        Bars3 = 3,
        Bars4 = 4,
        Bars5 = 5
    }
}

enum_type! {
    pub enum CommunicationsUpdateRegistrationStatus {
        Unknown = 0,
        NotRegistered = 1,
        Searching = 2,
        Denied = 3,
        RegisteredHome = 4,
        RegisteredRoaming = 5,
        EmergencyCallsOnly = 6
    }
}

packet_type! {
    pub struct CommunicationsUpdate: 0x4158 {
        #[csm_id(0)]  pub signal_strength: Option<CommunicationsUpdateSignalStrength>,
        #[csm_id(1)]  pub registration_status: Option<CommunicationsUpdateRegistrationStatus>,
        #[csm_id(2)]  pub airplane_mode_status: Option<bool>,
        #[csm_id(4)]  pub carrier_name: Option<CsmString>,
        #[csm_id(5)]  pub cellular_supported: Option<bool>,
        #[csm_id(6)]  pub telephony_enabled: Option<bool>,
        #[csm_id(7)]  pub facetime_audio_enabled: Option<bool>,
        #[csm_id(8)]  pub facetime_video_enabled: Option<bool>,
        #[csm_id(9)]  pub mute_status: Option<bool>,

        #[csm_id(10)] pub current_call_count: Option<u8>,
        #[csm_id(11)] pub new_voicemail_count: Option<u8>,
        #[csm_id(12)] pub initiate_call_available: Option<bool>,
        #[csm_id(13)] pub end_and_accept_available: Option<bool>,
        #[csm_id(14)] pub hold_and_accept_available: Option<bool>,
        #[csm_id(15)] pub swap_available: Option<bool>,
        #[csm_id(16)] pub merge_available: Option<bool>,
        #[csm_id(17)] pub hold_available: Option<bool>,

    }
}

packet_type! {
    pub struct StopCommunicationsUpdates: 0x4159 {
    }
}

enum_type! {
    pub enum InitiateCallType {
        Destination = 0,
        Voicemail = 1,
        Redial = 2
    }
}

enum_type! {
    pub enum InitiateCallService {
        Telephony = 1,
        FaceTimeAudio = 2,
        FaceTimeVideo = 3
    }
}

packet_type! {
    pub struct InitiateCall: 0x415A {
        #[csm_id(0)]  pub call_type: InitiateCallType,
        #[csm_id(1)]  pub destination_id: Option<CsmString>,
        #[csm_id(2)]  pub service: Option<InitiateCallService>,
        #[csm_id(3)]  pub address_book_id: Option<CsmString>
    }
}

enum_type! {
    pub enum AcceptCallAcceptAction {
        AcceptOrHoldAndAccept = 0,
        EndAndAccept = 1
    }
}

packet_type! {
    pub struct AcceptCall: 0x415B {
        #[csm_id(0)]  pub accept_action: AcceptCallAcceptAction,
        #[csm_id(1)]  pub call_uuid: Option<CsmString>,
    }
}

enum_type! {
    pub enum EndCallEndAction {
        EndOrDecline = 0,
        EndAll = 1
    }
}

packet_type! {
    pub struct EndCall: 0x415C {
        #[csm_id(0)]  pub end_action: EndCallEndAction,
        #[csm_id(1)]  pub call_uuid: Option<CsmString>,
    }
}

packet_type! {
    pub struct SwapCalls: 0x415D {
    }
}

packet_type! {
    pub struct MergeCalls: 0x415E {
    }
}

packet_type! {
    pub struct HoldStatusUpdate: 0x415F {
        #[csm_id(0)]  pub hold_status: bool,
        #[csm_id(1)]  pub call_uuid: Option<CsmString>,
    }
}

packet_type! {
    pub struct MuteStatusUpdate: 0x4160 {
        #[csm_id(0)]  pub mute_status: bool,

    }
}

enum_type! {
    pub enum SendDTMFTone {
        Number0 = 0,
        Number1 = 1,
        Number2 = 2,
        Number3 = 3,
        Number4 = 4,
        Number5 = 5,
        Number6 = 6,
        Number7 = 7,
        Number8 = 8,
        Number9 = 9,
        Star = 10,
        Pound = 11
    }
}

packet_type! {
    pub struct SendDTMF: 0x4161 {
        #[csm_id(0)]  pub tone: SendDTMFTone,
        #[csm_id(1)]  pub call_uuid: Option<CsmString>,
    }
}
