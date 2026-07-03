use crate::msg::{StreamDescription, StreamDescriptionResponse};
use catplay_plist::plist_struct;

plist_struct! {
    pub struct Setup {
        pub streams: Vec<StreamDescription>,
    }
}

impl Setup {
    pub fn new(streams: &[StreamDescription]) -> Self {
        Self { streams: streams.into() }
    }
}

plist_struct! {
    pub struct SetupResponse {
        pub streams: Vec<StreamDescriptionResponse>,
    }
}

impl SetupResponse {
    pub fn new(streams: &[StreamDescriptionResponse]) -> Self {
        Self { streams: streams.into() }
    }
}
