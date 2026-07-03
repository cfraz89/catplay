use std::{io, sync::Arc};

#[derive(Debug, Clone, thiserror::Error)]
pub enum BasicIoError {
    #[error("I/O error: {0:?}")]
    Io(Arc<io::Error>),
}

impl From<io::Error> for BasicIoError {
    fn from(value: io::Error) -> Self {
        Self::Io(value.into())
    }
}
