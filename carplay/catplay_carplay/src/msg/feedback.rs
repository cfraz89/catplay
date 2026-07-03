use crate::msg::StreamFeedback;
use catplay_plist::plist_struct;

plist_struct! {
    pub struct FeedbackPayload {
        #[serde(default)]
        pub streams: Vec<StreamFeedback>,
    }
}

impl FeedbackPayload {
    pub fn new(streams: &[StreamFeedback]) -> Self {
        Self { streams: streams.into() }
    }
}
