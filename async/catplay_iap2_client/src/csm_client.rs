use catplay_util::AsyncShutdown;

use crate::{CsmClientHandleRef, CsmSessionStatus, tokio::AsyncClient};

// #[async_trait]
pub trait CsmClient: AsyncShutdown + Send + 'static {
    fn status(&self) -> CsmSessionStatus;

    fn handle(&self) -> CsmClientHandleRef;
}

// pub type CsmClientBox = Box<dyn CsmClient>;
pub type CsmClientBox = Box<AsyncClient>;
