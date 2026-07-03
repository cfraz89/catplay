use catplay_plist::PlistError;

#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    #[error("Unknown AirPlay command: {0}")]
    UnknownCommand(String),
    #[error("Failed to deserialize command '{0}': {0}")]
    FailedDeserialize(String, PlistError),
    #[error("Invalid command payload: {0}")]
    InvalidPayload(PlistError),
}
