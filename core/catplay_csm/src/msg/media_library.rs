use crate::{
    decoder::{CsmString, CsmVec},
    enum_type, group_type,
};

group_type! {
    pub struct MediaItem {
        #[csm_id(0)]  pub persistent_id: Option<u64>,
        #[csm_id(1)]  pub title: Option<CsmString>,
        #[csm_id(2)]  pub media_type: CsmVec<MediaType>,
        #[csm_id(3)]  pub rating: Option<u8>, // 0..5
        #[csm_id(4)]  pub playback_duration_in_ms: Option<u32>,
        #[csm_id(5)]  pub album_persistent_id: Option<u64>,
        #[csm_id(6)]  pub album_title: Option<CsmString>,
        #[csm_id(7)]  pub album_track_number: Option<u16>,
        #[csm_id(8)]  pub album_track_count: Option<u16>,
        #[csm_id(9)]  pub album_disc_number: Option<u16>,
        #[csm_id(10)] pub album_disc_count: Option<u16>,

        #[csm_id(11)] pub artist_persistent_id: Option<u64>,
        #[csm_id(12)] pub artist: Option<CsmString>,
        #[csm_id(13)] pub album_artist_persistent_id: Option<u64>,
        #[csm_id(14)] pub album_artist: Option<CsmString>,

        #[csm_id(15)] pub genre_persistent_id: Option<u64>,
        #[csm_id(16)] pub genre: Option<CsmString>,

        #[csm_id(17)] pub composer_persistent_id: Option<u64>,
        #[csm_id(18)] pub composer: Option<CsmString>,

        #[csm_id(19)] pub is_part_of_compilation: Option<bool>,

        #[csm_id(21)] pub is_like_supported: Option<bool>,
        #[csm_id(22)] pub is_ban_supported: Option<bool>,
        #[csm_id(23)] pub is_liked: Option<bool>,
        #[csm_id(24)] pub is_banned: Option<bool>,
        #[csm_id(25)] pub is_resident_on_device: Option<bool>,

        #[csm_id(26)] pub artwork_file_transfer_id: Option<u8>,
        #[csm_id(27)] pub chapter_count: Option<u16>,
    }
}

enum_type! {
    pub enum MediaType {
        Music = 0,
        Podcast = 1,
        AudioBook = 2,
        ITunesU = 3,
    }
}
