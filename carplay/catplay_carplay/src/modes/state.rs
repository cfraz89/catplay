use crate::modes::{
    AppStateChanged, AppStateEnum, ChangeModes, EntityEnum, ModesChanged, ResourceChanged, ResourceID, ResourceState, SpeechMode,
    SpeechState,
};

/// Describes current, authoritative state of all modes, as accepted by the `Controller`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AirPlayModeState {
    /// Current user of the screen.
    pub screen: ResourceState,
    /// Current user of main audio.
    pub main_audio: ResourceState,

    // Owner of phone call.
    pub phone_call: EntityEnum,
    // Owner of speech and its mode.
    pub speech: SpeechState,
    // Owner of navigation.
    pub turn_by_turn: EntityEnum,
}

impl Default for AirPlayModeState {
    /// Default state of `modes`, granting all resources to the `Controller`.
    fn default() -> Self {
        Self {
            screen: ResourceState::ControllerHas,
            main_audio: ResourceState::ControllerHas,
            phone_call: EntityEnum::None,
            speech: SpeechState {
                entity: EntityEnum::None,
                mode: SpeechMode::None,
            },
            turn_by_turn: EntityEnum::None,
        }
    }
}

impl AirPlayModeState {
    pub fn new(modes: &ChangeModes) -> Self {
        let mut this = Self::default();

        for i in &modes.app_states {
            match i.app_state_id {
                AppStateEnum::Speech => {
                    let mode = i.speech_mode.unwrap_or(SpeechMode::None);
                    this.speech = SpeechState {
                        entity: if mode == SpeechMode::None {
                            EntityEnum::None
                        } else {
                            EntityEnum::Accessory
                        },
                        mode,
                    };
                }
                AppStateEnum::PhoneCall => {
                    this.phone_call = if i.state.unwrap_or(false) {
                        EntityEnum::Accessory
                    } else {
                        EntityEnum::None
                    };
                }
                AppStateEnum::TurnByTurn => {
                    this.turn_by_turn = if i.state.unwrap_or(false) {
                        EntityEnum::Accessory
                    } else {
                        EntityEnum::None
                    };
                }
                AppStateEnum::Invalid => {}
            }
        }
        this
    }

    /// Update authoritative state based on [ModesChanged] received from `Controller`.
    pub fn feed(&mut self, update: &ModesChanged) {
        for resource in &update.resources {
            let state = ResourceState::from_pair(resource.entity, resource.permanent_entity);

            match resource.resource_id {
                ResourceID::MainScreen => self.screen = state,
                ResourceID::MainAudio => self.main_audio = state,
            }
        }

        for app_state in &update.app_states {
            match app_state.app_state_id {
                AppStateEnum::Speech => {
                    self.speech = SpeechState {
                        entity: app_state.entity,
                        mode: app_state.speech_mode.unwrap_or(SpeechMode::None),
                    };
                }
                AppStateEnum::PhoneCall => self.phone_call = app_state.entity,
                AppStateEnum::TurnByTurn => self.turn_by_turn = app_state.entity,
                _ => {}
            }
        }
    }

    pub fn serialize(&self) -> ModesChanged {
        let mut resources = Vec::with_capacity(2);

        let (entity, permanent_entity) = self.screen.to_pair();
        resources.push(ResourceChanged {
            resource_id: ResourceID::MainScreen,
            entity,
            permanent_entity,
        });

        let (entity, permanent_entity) = self.main_audio.to_pair();
        resources.push(ResourceChanged {
            resource_id: ResourceID::MainAudio,
            entity,
            permanent_entity,
        });

        let app_states = vec![
            AppStateChanged {
                app_state_id: AppStateEnum::PhoneCall,
                entity: self.phone_call,
                speech_mode: None,
            },
            AppStateChanged {
                app_state_id: AppStateEnum::TurnByTurn,
                entity: self.turn_by_turn,
                speech_mode: None,
            },
            AppStateChanged {
                app_state_id: AppStateEnum::Speech,
                entity: self.speech.entity,
                speech_mode: Some(self.speech.mode),
            },
        ];

        ModesChanged {
            app_states,
            resources,
            reason_str: String::new(),
        }
    }
}

impl From<&ModesChanged> for AirPlayModeState {
    fn from(update: &ModesChanged) -> Self {
        let mut this = Self::default();
        this.feed(update);
        this
    }
}

impl From<&ChangeModes> for AirPlayModeState {
    fn from(modes: &ChangeModes) -> Self {
        Self::new(modes)
    }
}
