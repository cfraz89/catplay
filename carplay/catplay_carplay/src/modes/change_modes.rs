use catplay_plist::{plist_enum_repr, plist_struct};

use crate::modes::ModesChanged;

plist_struct! {
    pub struct ChangeModes {
        #[serde(default)]
        pub app_states: Vec<AppState>,
        #[serde(default)]
        pub resources: Vec<Resource>,
        #[serde(default)]
        pub reason_str: String,

        #[serde(default)]
        pub initial_permanent_entity: Vec<InitialPermanentEntity>
    }
}

plist_struct! {
    pub struct ChangeModesResponse {
        // Always 0 (no error)
        pub status: u8,
        // Snapshot of updated modes
        #[serde(default)]
        pub params: Option<ModesChanged>
    }
}

impl ChangeModesResponse {
    pub fn new(params: ModesChanged) -> Self {
        Self {
            status: 0,
            params: Some(params),
        }
    }

    pub fn error(status: u8) -> Self {
        Self { status, params: None }
    }

    pub fn is_error(&self) -> bool {
        self.status != 0
    }
}

impl ChangeModes {
    /// Simple declaration of initial modes granting all resources to the `Controller`.
    pub fn initial() -> Self {
        Self {
            app_states: vec![
                AppState::new(AppStateEnum::PhoneCall, false),
                AppState::new(AppStateEnum::TurnByTurn, false),
                AppState::speech(SpeechMode::None),
            ],
            resources: vec![
                Resource::take(
                    ResourceID::MainScreen,
                    ResourceTransferPriority::NiceToHave,
                    ResourceConstraint::Anytime,
                    ResourceConstraint::Anytime,
                ),
                Resource::take(
                    ResourceID::MainAudio,
                    ResourceTransferPriority::NiceToHave,
                    ResourceConstraint::Anytime,
                    ResourceConstraint::Anytime,
                ),
            ],
            reason_str: "initial".into(),
            initial_permanent_entity: vec![
                InitialPermanentEntity::controller(ResourceID::MainAudio),
                InitialPermanentEntity::controller(ResourceID::MainScreen),
                // InitialPermanentEntity::accessory(ResourceID::MainScreen, ResourceConstraint::Anytime, ResourceConstraint::Anytime),
                // InitialPermanentEntity::accessory(ResourceID::MainAudio, ResourceConstraint::Anytime, ResourceConstraint::Anytime),
            ],
        }
    }
}

plist_struct! {
    pub struct AppState {
        #[serde(rename = "appStateID")]
        pub app_state_id: AppStateEnum,
        pub state: Option<bool>,
        pub speech_mode: Option<SpeechMode>
    }
}

impl AppState {
    pub fn new(id: AppStateEnum, state: bool) -> Self {
        Self {
            app_state_id: id,
            state: Some(state),
            speech_mode: None,
        }
    }

    pub fn speech(speech_mode: SpeechMode) -> Self {
        Self {
            app_state_id: AppStateEnum::Speech,
            state: None,
            speech_mode: Some(speech_mode),
        }
    }
}

plist_struct! {
    pub struct Resource {
        #[serde(rename = "resourceID")]
        pub resource_id: ResourceID,
        pub transfer_type: ResourceTransferType,

        pub transfer_priority: Option<ResourceTransferPriority>,
        pub take_constraint: Option<ResourceConstraint>,
        pub borrow_constraint: Option<ResourceConstraint>,
        pub unborrow_constraint: Option<ResourceConstraint>,
    }
}

impl Resource {
    pub fn unborrow(resource: ResourceID) -> Self {
        Self {
            resource_id: resource,
            transfer_type: ResourceTransferType::Unborrow,
            transfer_priority: None,
            take_constraint: None,
            borrow_constraint: None,
            unborrow_constraint: None,
        }
    }

    pub fn untake(resource: ResourceID) -> Self {
        Self {
            resource_id: resource,
            transfer_type: ResourceTransferType::Untake,
            transfer_priority: None,
            take_constraint: None,
            borrow_constraint: None,
            unborrow_constraint: None,
        }
    }

    pub fn borrow(resource: ResourceID, priority: ResourceTransferPriority, unborrow: ResourceConstraint) -> Self {
        Self {
            resource_id: resource,
            transfer_type: ResourceTransferType::Borrow,
            transfer_priority: Some(priority),
            take_constraint: None,
            borrow_constraint: None,
            unborrow_constraint: Some(unborrow),
        }
    }

    pub fn take(resource: ResourceID, priority: ResourceTransferPriority, take: ResourceConstraint, borrow: ResourceConstraint) -> Self {
        Self {
            resource_id: resource,
            transfer_type: ResourceTransferType::Take,
            transfer_priority: Some(priority),
            take_constraint: Some(take),
            borrow_constraint: Some(borrow),
            unborrow_constraint: None,
        }
    }
}

plist_enum_repr! {
    #[repr(u8)]
    pub enum EntityEnum {
        #[default]
        None = 0,
        Controller = 1,
        Accessory = 2
    }
}

plist_enum_repr! {
    #[repr(u8)]
    pub enum AppStateEnum {
        #[default]
        Invalid = 0,
        /// A device is recording audio for the purpose of speech.
        Speech = 1,
        /// A device is on a phone call.
        PhoneCall = 2,
        /// A device is performing turn-by-turn navigation.
        TurnByTurn = 3
    }
}

plist_enum_repr! {
    #[repr(i8)]
    pub enum SpeechMode {
        #[default]
        Invalid = 0,
        None = -1,
        Speaking = 1,
        RecognizingSpeech = 2
    }
}

plist_enum_repr! {
    #[repr(u8)]
    pub enum ResourceID {
        #[default]
        MainScreen = 1,
        MainAudio = 2,
    }
}

plist_enum_repr! {
    #[repr(u8)]
    pub enum ResourceTransferType {
        #[default]
        Take = 1,
        Untake = 2,
        Borrow = 3,
        Unborrow = 4
    }
}

impl ResourceTransferType {
    pub fn reverse(&self) -> Self {
        match self {
            ResourceTransferType::Take => ResourceTransferType::Untake,
            ResourceTransferType::Untake => ResourceTransferType::Take,
            ResourceTransferType::Borrow => ResourceTransferType::Unborrow,
            ResourceTransferType::Unborrow => ResourceTransferType::Borrow,
        }
    }
}

plist_enum_repr! {
    #[repr(u32)]
    pub enum ResourceTransferPriority {
        #[default]
        NiceToHave = 100,
        UserInitiated = 500,
    }
}

plist_enum_repr! {
    #[repr(u32)]
    pub enum ResourceConstraint {
        #[default]
        Anytime = 100,
        UserInitiated = 500,
        Never = 1000
    }
}

plist_struct! {
    pub struct InitialPermanentEntity {
        pub permanent_entity: EntityEnum,
        #[serde(rename = "resourceID")]
        pub resource_id: ResourceID,
        pub take_constraint: Option<ResourceConstraint>,
        pub borrow_constraint: Option<ResourceConstraint>,
    }
}

impl InitialPermanentEntity {
    pub fn accessory(id: ResourceID, take: ResourceConstraint, borrow: ResourceConstraint) -> Self {
        Self {
            permanent_entity: EntityEnum::Accessory,
            resource_id: id,
            take_constraint: Some(take),
            borrow_constraint: Some(borrow),
        }
    }

    pub fn controller(id: ResourceID) -> Self {
        Self {
            permanent_entity: EntityEnum::Controller,
            resource_id: id,
            take_constraint: None,
            borrow_constraint: None,
        }
    }
}
