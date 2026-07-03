use crate::msg::StreamType;
use catplay_plist::plist_struct;

plist_struct! {
    pub struct TeardownPayload {
        #[serde(default)]
        pub streams: Vec<TeardownStream>,
    }
}

impl TeardownPayload {
    pub fn new(streams: &[StreamType]) -> Self {
        Self {
            streams: streams.iter().map(|s| TeardownStream::new(*s)).collect(),
        }
    }

    pub fn screen(stream_type: StreamType, uuid: impl Into<String>) -> Self {
        Self {
            streams: vec![TeardownStream::screen(stream_type, uuid)],
        }
    }
}

plist_struct! {
    pub struct TeardownStream {
        #[serde(rename = "type")]
        pub stream_type: StreamType,
        pub uuid: Option<String>
    }
}

impl TeardownStream {
    pub fn new(stream_type: StreamType) -> Self {
        Self { stream_type, uuid: None }
    }

    pub fn screen(stream_type: StreamType, uuid: impl Into<String>) -> Self {
        Self {
            stream_type,
            uuid: Some(uuid.into()),
        }
    }
}
