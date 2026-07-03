use crate::{decoder::*, enum_type, group_type, packet_type};

group_type! {
    pub struct RecentsListProperties {
        #[csm_id(0)] pub index: CsmFlag,
        #[csm_id(1)] pub remote_id: CsmFlag,
        #[csm_id(2)] pub display_name: CsmFlag,
        #[csm_id(3)] pub label: CsmFlag,
        #[csm_id(4)] pub address_book_id: CsmFlag,
        #[csm_id(5)] pub service: CsmFlag,
        #[csm_id(6)] pub prop_type: CsmFlag,
        #[csm_id(7)] pub unix_timestamp: CsmFlag,
        #[csm_id(8)] pub duration: CsmFlag,
        #[csm_id(9)] pub occurrences: CsmFlag,
    }
}

group_type! {
    pub struct FavoritesListProperties {
        #[csm_id(0)] pub index: CsmFlag,
        #[csm_id(1)] pub remote_id: CsmFlag,
        #[csm_id(2)] pub display_name: CsmFlag,
        #[csm_id(3)] pub label: CsmFlag,
        #[csm_id(4)] pub address_book_id: CsmFlag,
        #[csm_id(5)] pub service: CsmFlag,
    }
}

enum_type! {
    pub enum ListUpdateService {
        Unknown = 0,
        Telephony = 1,
        FaceTimeAudio = 2,
        FaceTimeVideo = 3
    }
}

enum_type! {
    pub enum ListUpdateRecentsListType {
        Unknown = 0,
        Incoming = 1,
        Outgoing = 2,
        Missed = 3
    }
}

group_type! {
    pub struct RecentsList {
        #[csm_id(0)] pub index: u16,
        #[csm_id(1)] pub remote_id: CsmString,
        #[csm_id(2)] pub display_name: CsmString,
        #[csm_id(3)] pub label: Option<CsmString>,
        #[csm_id(4)] pub address_book_id: Option<CsmString>,
        #[csm_id(5)] pub service: ListUpdateService,
        #[csm_id(6)] pub list_type: ListUpdateRecentsListType,
        #[csm_id(7)] pub unix_timestamp: Option<u64>,
        #[csm_id(8)] pub duration: Option<u32>,
        #[csm_id(9)] pub occurrences: u8,
    }
}

group_type! {
    pub struct FavoritesList {
        #[csm_id(0)] pub index: u16,
        #[csm_id(1)] pub remote_id: CsmString,
        #[csm_id(2)] pub display_name: CsmString,
        #[csm_id(3)] pub label: Option<CsmString>,
        #[csm_id(4)] pub address_book_id: Option<CsmString>,
        #[csm_id(5)] pub service: ListUpdateService,
    }
}

packet_type! {
    pub struct StartListUpdates: 0x4170 {
        #[csm_id(1)]  pub recents_list_properties: Option<RecentsListProperties>,
        #[csm_id(3)]  pub recents_list_max: Option<u16>,
        #[csm_id(4)]  pub recents_list_combine: Option<bool>,
        #[csm_id(6)]  pub favorites_list_properties: Option<FavoritesListProperties>,
        #[csm_id(8)]  pub favorites_list_max: Option<u16>,
    }
}

packet_type! {
    pub struct ListUpdate: 0x4171 {
        #[csm_id(0)] pub recents_list_available: Option<bool>,
        #[csm_id(1)] pub recents_list: CsmVec<RecentsList>,
        #[csm_id(2)] pub recents_list_count: Option<u16>,
        #[csm_id(5)] pub favorites_list_available: Option<bool>,
        #[csm_id(6)] pub favorites_list: CsmVec<FavoritesList>,
        #[csm_id(7)] pub favorites_list_count: Option<u16>,
    }
}

packet_type! {
    pub struct StopListUpdates: 0x4172 {
    }
}
