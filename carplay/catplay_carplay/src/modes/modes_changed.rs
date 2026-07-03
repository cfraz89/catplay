use crate::modes::{AppStateEnum, EntityEnum, ResourceID, SpeechMode};
use catplay_plist::plist_struct;

plist_struct! {
    pub struct ModesChanged {
        #[serde(default)]
        pub app_states: Vec<AppStateChanged>,
        #[serde(default)]
        pub resources: Vec<ResourceChanged>,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        pub reason_str: String,
    }
}

plist_struct! {
    pub struct AppStateChanged {
        #[serde(rename = "appStateID")]
        pub app_state_id: AppStateEnum,
        pub entity: EntityEnum,
        pub speech_mode: Option<SpeechMode>
    }
}

plist_struct! {
    pub struct ResourceChanged {
        #[serde(rename = "resourceID")]
        pub resource_id: ResourceID,
        pub entity: EntityEnum,
        pub permanent_entity: EntityEnum, // Option<EntityEnum>
    }
}
