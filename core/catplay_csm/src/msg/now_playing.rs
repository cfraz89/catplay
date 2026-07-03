use crate::{
    decoder::{CsmFlag, CsmString},
    enum_type, group_type,
    msg::MediaItem,
    packet_type,
};

packet_type! {
    pub struct StartNowPlayingUpdates: 0x5000 {
        #[csm_id( 0)] pub attributes: Option<StartNowPlayingMediaItemAttributes>,
        #[csm_id( 1)] pub playback_attributes: Option<StartNowPlayingPlaybackAttributes>,
        #[csm_id( 2)] pub playback_queue_list_content_transfer_info_request: Option<PlaybackQueueListContentTransferInfoRequest>,
    }
}

impl StartNowPlayingUpdates {
    pub fn all() -> Self {
        StartNowPlayingUpdates {
            attributes: Some(StartNowPlayingMediaItemAttributes::all()),
            playback_attributes: Some(StartNowPlayingPlaybackAttributes::all()),
            playback_queue_list_content_transfer_info_request: Some(PlaybackQueueListContentTransferInfoRequest::all()),
        }
    }
}
group_type! {
    pub struct StartNowPlayingMediaItemAttributes {
        #[csm_id(0)] pub persistent_id: CsmFlag,
        #[csm_id( 1)] pub title: CsmFlag,
        #[csm_id( 4)] pub playback_duration_in_ms: CsmFlag,
        #[csm_id( 6)] pub album_title: CsmFlag,
        #[csm_id( 7)] pub album_track_number: CsmFlag,
        #[csm_id( 8)] pub album_track_count: CsmFlag,
        #[csm_id( 9)] pub album_disc_number: CsmFlag,
        #[csm_id( 10)] pub album_disc_count: CsmFlag,
        #[csm_id( 12)] pub artist: CsmFlag,
        #[csm_id( 16)] pub genre: CsmFlag,
        #[csm_id( 18)] pub composer: CsmFlag,
        #[csm_id( 21)] pub is_like_supported: CsmFlag,
        #[csm_id( 22)] pub is_ban_supported: CsmFlag,
        #[csm_id( 23)] pub is_liked: CsmFlag,
        #[csm_id( 24)] pub is_banned: CsmFlag,
        #[csm_id( 26)] pub artwork_file_transfer_id: CsmFlag,
        #[csm_id( 27)] pub chapter_count: CsmFlag,
    }
}

impl StartNowPlayingMediaItemAttributes {
    pub fn all() -> Self {
        Self {
            persistent_id: CsmFlag::Yes,
            title: CsmFlag::Yes,
            playback_duration_in_ms: CsmFlag::Yes,
            album_title: CsmFlag::Yes,
            album_track_number: CsmFlag::Yes,
            album_track_count: CsmFlag::Yes,
            album_disc_number: CsmFlag::Yes,
            album_disc_count: CsmFlag::Yes,
            artist: CsmFlag::Yes,
            genre: CsmFlag::Yes,
            composer: CsmFlag::Yes,
            is_like_supported: CsmFlag::Yes,
            is_ban_supported: CsmFlag::Yes,
            is_liked: CsmFlag::Yes,
            is_banned: CsmFlag::Yes,
            artwork_file_transfer_id: CsmFlag::Yes,
            chapter_count: CsmFlag::Yes,
        }
    }
}

group_type! {
    pub struct StartNowPlayingPlaybackAttributes {
        #[csm_id(0)]  pub playback_status: CsmFlag,
        #[csm_id(1)]  pub playback_elapsed_time_in_ms: CsmFlag,
        #[csm_id(2)]  pub playback_queue_index: CsmFlag,
        #[csm_id(3)]  pub playback_queue_count: CsmFlag,
        #[csm_id(4)]  pub playback_queue_chapter_index: CsmFlag,
        #[csm_id(5)]  pub playback_shuffle_mode: CsmFlag,
        #[csm_id(6)]  pub playback_repeat_mode: CsmFlag,
        #[csm_id(7)]  pub playback_app_name: CsmFlag, // required
        #[csm_id(8)]  pub playback_media_library_unique_identifier: CsmFlag,
        #[csm_id(9)]  pub pb_apple_music_radio_ad: CsmFlag,
        #[csm_id(10)] pub pb_apple_music_radio_station_name: CsmFlag,
        #[csm_id(11)] pub pb_apple_music_radio_station_media_playlist_id: CsmFlag,
        #[csm_id(12)] pub playback_speed: CsmFlag,
        #[csm_id(13)] pub set_elapsed_time_available: CsmFlag,
        #[csm_id(14)] pub playback_queue_list_avail: CsmFlag,
        #[csm_id(15)] pub playback_queue_list_transfer_id: CsmFlag,
        #[csm_id(16)] pub playback_app_bundle_id: CsmFlag, // required
        #[csm_id(17)] pub playback_queue_list_content_transfer_size: Option<u32>,
    }
}

impl StartNowPlayingPlaybackAttributes {
    pub fn all() -> Self {
        Self {
            playback_status: CsmFlag::Yes,
            playback_elapsed_time_in_ms: CsmFlag::Yes,
            playback_queue_index: CsmFlag::Yes,
            playback_queue_count: CsmFlag::Yes,
            playback_queue_chapter_index: CsmFlag::Yes,
            playback_shuffle_mode: CsmFlag::Yes,
            playback_repeat_mode: CsmFlag::Yes,
            playback_app_name: CsmFlag::Yes,
            playback_media_library_unique_identifier: CsmFlag::Yes,
            pb_apple_music_radio_ad: CsmFlag::Yes,
            pb_apple_music_radio_station_name: CsmFlag::Yes,
            pb_apple_music_radio_station_media_playlist_id: CsmFlag::Yes,
            playback_speed: CsmFlag::Yes,
            set_elapsed_time_available: CsmFlag::Yes,
            playback_queue_list_avail: CsmFlag::Yes,
            playback_queue_list_transfer_id: CsmFlag::Yes,
            playback_app_bundle_id: CsmFlag::Yes,
            playback_queue_list_content_transfer_size: Some(0), // unlimited
        }
    }
}

group_type! {
    pub struct PlaybackQueueListContentTransferInfoRequest {
        #[csm_id(0)]  pub pid: CsmFlag,
        #[csm_id(1)]  pub title: CsmFlag,
        #[csm_id(6)]  pub album_title: CsmFlag,
        #[csm_id(12)] pub artist: CsmFlag,
        #[csm_id(14)] pub album_artist: CsmFlag,
        #[csm_id(16)] pub genre: CsmFlag,
        #[csm_id(18)] pub composer: CsmFlag,
    }
}

impl PlaybackQueueListContentTransferInfoRequest {
    pub fn all() -> Self {
        Self {
            pid: CsmFlag::Yes,
            title: CsmFlag::Yes,
            album_title: CsmFlag::Yes,
            artist: CsmFlag::Yes,
            album_artist: CsmFlag::Yes,
            genre: CsmFlag::Yes,
            composer: CsmFlag::Yes,
        }
    }
}

packet_type! {
    pub struct NowPlayingUpdate: 0x5001 {
        #[csm_id( 0)] pub media_item: Option<MediaItem>,
        #[csm_id( 1)] pub playback_attributes: Option<PlaybackAttributes>,
    }
}

impl NowPlayingUpdate {
    pub fn deep_merge(&mut self, update: Self) {
        if let Some(media_item) = update.media_item {
            match &mut self.media_item {
                Some(current) => merge_media_item(current, media_item),
                None => self.media_item = Some(media_item),
            }
        }

        if let Some(playback_attributes) = update.playback_attributes {
            match &mut self.playback_attributes {
                Some(current) => merge_playback_attributes(current, playback_attributes),
                None => self.playback_attributes = Some(playback_attributes),
            }
        }
    }
}

macro_rules! merge_some_fields {
    ($current:expr, $update:expr, $($field:ident),+ $(,)?) => {
        $(
            if let Some(value) = $update.$field {
                $current.$field = Some(value);
            }
        )+
    };
}

fn merge_media_item(current: &mut MediaItem, update: MediaItem) {
    merge_some_fields!(
        current,
        update,
        persistent_id,
        title,
        rating,
        playback_duration_in_ms,
        album_persistent_id,
        album_title,
        album_track_number,
        album_track_count,
        album_disc_number,
        album_disc_count,
        artist_persistent_id,
        artist,
        album_artist_persistent_id,
        album_artist,
        genre_persistent_id,
        genre,
        composer_persistent_id,
        composer,
        is_part_of_compilation,
        is_like_supported,
        is_ban_supported,
        is_liked,
        is_banned,
        is_resident_on_device,
        artwork_file_transfer_id,
        chapter_count,
    );

    if !update.media_type.is_empty() {
        current.media_type = update.media_type;
    }
}

enum_type! {
    pub enum PlaybackStatus {
        Stopped = 0,
        Playing = 1,
        Paused = 2,
        SeekForward = 3,
        SeekBackward = 4
    }
}

enum_type! {
    pub enum PlaybackShuffle {
        Off = 0,
        Songs = 1,
        Albums = 2,
    }
}

enum_type! {
    pub enum PlaybackRepeat {
        Off = 0,
        One = 1,
        All = 2,
    }
}

group_type! {
    pub struct PlaybackAttributes {
        #[csm_id(0)]  pub playback_status: Option<PlaybackStatus>,
        #[csm_id(1)]  pub playback_elapsed_time_ms: Option<u32>,
        #[csm_id(2)]  pub playback_queue_index: Option<u32>,
        #[csm_id(3)]  pub playback_queue_count: Option<u32>,
        #[csm_id(4)]  pub playback_queue_chapter_index: Option<u32>,
        #[csm_id(5)]  pub playback_shuffle_mode: Option<PlaybackShuffle>,
        #[csm_id(6)]  pub playback_repeat_mode: Option<PlaybackRepeat>,
        #[csm_id(7)]  pub playback_app_name: Option<CsmString>,
        #[csm_id(8)]  pub pb_media_library_unique_identifier: Option<CsmString>,
        #[csm_id(9)]  pub pb_apple_music_radio_ad: Option<bool>,
        #[csm_id(10)] pub pb_apple_music_radio_station_name: Option<CsmString>,
        #[csm_id(11)] pub pb_apple_music_radio_station_media_playlist_id: Option<u64>,
        #[csm_id(12)] pub playback_speed: Option<u16>,
        #[csm_id(13)] pub set_elapsed_time_available: Option<bool>,
        #[csm_id(14)] pub playback_queue_list_available: Option<bool>,
        #[csm_id(15)] pub playback_queue_list_transfer_id: Option<u8>,
        #[csm_id(16)] pub playback_app_bundle_id: Option<CsmString>,
        #[csm_id(17)] pub playback_queue_list_content_transfer: CsmFlag,
    }
}

fn merge_playback_attributes(current: &mut PlaybackAttributes, update: PlaybackAttributes) {
    merge_some_fields!(
        current,
        update,
        playback_status,
        playback_elapsed_time_ms,
        playback_queue_index,
        playback_queue_count,
        playback_queue_chapter_index,
        playback_shuffle_mode,
        playback_repeat_mode,
        playback_app_name,
        pb_media_library_unique_identifier,
        pb_apple_music_radio_ad,
        pb_apple_music_radio_station_name,
        pb_apple_music_radio_station_media_playlist_id,
        playback_speed,
        set_elapsed_time_available,
        playback_queue_list_available,
        playback_queue_list_transfer_id,
        playback_app_bundle_id,
    );

    if update.playback_queue_list_content_transfer == CsmFlag::Yes {
        current.playback_queue_list_content_transfer = CsmFlag::Yes;
    }
}

packet_type! {
    pub struct StopNowPlayingUpdates: 0x5002 {}
}

packet_type! {
    pub struct SetNowPlayingInformation: 0x5003 {
        #[csm_id(0)] pub elapsed_time: Option<u32>,
        #[csm_id(1)] pub playback_queue_index: Option<u32>,
        #[csm_id(2)] pub playback_queue_list_content_transfer_start_index: Option<u32>,
    }
}
