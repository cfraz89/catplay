use core::fmt;

#[repr(u16)]
pub enum FileTransferSetupMetaType {
    NowPlayingArtworkData = 0x02,
    NowPlayingPlaybackQueueContents = 0x03,
    MediaLibraryUpdatePlaylistContents = 0x04,
    MediaItemListNowPlayingPlaybackQueueContents = 0x06,
    MediaItemListMediaLibraryUpdatePlaylistContents = 0x07,
    AppDiscoveryIconData = 0x08,
}

impl TryFrom<u16> for FileTransferSetupMetaType {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let ret = match value {
            x if x == Self::NowPlayingArtworkData as _ => Self::NowPlayingArtworkData,
            x if x == Self::NowPlayingPlaybackQueueContents as _ => Self::NowPlayingPlaybackQueueContents,
            x if x == Self::MediaLibraryUpdatePlaylistContents as _ => Self::MediaLibraryUpdatePlaylistContents,
            x if x == Self::MediaItemListNowPlayingPlaybackQueueContents as _ => Self::MediaItemListNowPlayingPlaybackQueueContents,
            x if x == Self::MediaItemListMediaLibraryUpdatePlaylistContents as _ => Self::MediaItemListMediaLibraryUpdatePlaylistContents,
            x if x == Self::AppDiscoveryIconData as _ => Self::AppDiscoveryIconData,

            _ => return Err(()),
        };
        Ok(ret)
    }
}

impl fmt::Debug for FileTransferSetupMetaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NowPlayingArtworkData => write!(f, "NowPlayingArtworkData"),
            Self::NowPlayingPlaybackQueueContents => write!(f, "NowPlayingPlaybackQueueContents"),
            Self::MediaLibraryUpdatePlaylistContents => write!(f, "MediaLibraryUpdatePlaylistContents"),
            Self::MediaItemListNowPlayingPlaybackQueueContents => write!(f, "MediaItemListNowPlayingPlaybackQueueContents"),
            Self::MediaItemListMediaLibraryUpdatePlaylistContents => write!(f, "MediaItemListMediaLibraryUpdatePlaylistContents"),
            Self::AppDiscoveryIconData => write!(f, "AppDiscoveryIconData"),
        }
    }
}

impl From<FileTransferSetupMetaType> for u16 {
    fn from(val: FileTransferSetupMetaType) -> Self {
        val as u16
    }
}

pub enum FileTransferSetupMeta {
    NowPlayingArtworkData,
    NowPlayingPlaybackQueueContents,
    MediaLibraryUpdatePlaylistContents {/* PlaylistPID (uint64) + LibraryUID */},
    MediaItemListNowPlayingPlaybackQueueContents,
    MediaItemListMediaLibraryUpdatePlaylistContents {/* PlaylistPID (uint64) + LibraryUID */},
    AppDiscoveryIconData {/* AppBundleID (utf8) */},
}

impl TryFrom<(u16, &[u8])> for FileTransferSetupMeta {
    type Error = ();

    fn try_from(value: (u16, &[u8])) -> Result<Self, Self::Error> {
        let opcode = FileTransferSetupMetaType::try_from(value.0)?;
        let ret = match opcode {
            FileTransferSetupMetaType::NowPlayingArtworkData => Self::NowPlayingArtworkData,
            FileTransferSetupMetaType::NowPlayingPlaybackQueueContents => Self::NowPlayingPlaybackQueueContents,
            FileTransferSetupMetaType::MediaLibraryUpdatePlaylistContents => Self::MediaLibraryUpdatePlaylistContents {},
            FileTransferSetupMetaType::MediaItemListNowPlayingPlaybackQueueContents => Self::MediaItemListNowPlayingPlaybackQueueContents,
            FileTransferSetupMetaType::MediaItemListMediaLibraryUpdatePlaylistContents => {
                Self::MediaItemListMediaLibraryUpdatePlaylistContents {}
            }
            FileTransferSetupMetaType::AppDiscoveryIconData => Self::AppDiscoveryIconData {},
        };
        Ok(ret)
    }
}
