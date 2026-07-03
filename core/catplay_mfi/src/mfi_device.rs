use std::sync::Arc;

use crate::MfiResult;

pub trait MfiDevice: Send + Sync {
    fn read_certificate(&self) -> MfiResult<Vec<u8>>;

    fn generate_challenge_response(&self, challenge: &[u8]) -> MfiResult<Vec<u8>>;
}

pub type MfiDeficeRef = Arc<dyn MfiDevice>;
