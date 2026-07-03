use log::debug;
use tokio::sync::watch;

use crate::carplay_rx::AirPlayReceiverHandleRef;

/// Shared AirPlay server state between different server instances and between
/// different incoming (pending) connections.
#[derive(Clone)]
pub struct AirPlayServerShared {
    locked: watch::Sender<Option<AirPlayReceiverHandleRef>>,
}

impl Default for AirPlayServerShared {
    fn default() -> Self {
        Self::new()
    }
}

impl AirPlayServerShared {
    pub fn new() -> Self {
        Self {
            locked: watch::channel(None).0,
        }
    }

    /// Observes session lock state.
    pub fn subscribe(&self) -> watch::Receiver<Option<AirPlayReceiverHandleRef>> {
        self.locked.subscribe()
    }

    pub fn borrow(&self) -> Option<AirPlayReceiverHandleRef> {
        self.locked.borrow().clone()
    }

    /// Attempt to get a session lock, or `None` if session is locked by another connection.
    ///
    /// The session will be unlocked when the returned `SessionGuard` is dropped.
    pub fn lock(&self, handle: AirPlayReceiverHandleRef) -> Option<AirPlaySessionGuard> {
        if !self.locked.send_if_modified(|x| {
            if x.is_none() {
                x.replace(handle);
                return true;
            }

            false
        }) {
            debug!("Rejecting attempt for AirPlay session lock, already locked");
            return None;
        }

        debug!("Locking AirPlay session");
        Some(AirPlaySessionGuard {
            locked: self.locked.clone(),
        })
    }
}

pub struct AirPlaySessionGuard {
    locked: watch::Sender<Option<AirPlayReceiverHandleRef>>,
}

impl Drop for AirPlaySessionGuard {
    fn drop(&mut self) {
        debug!("Unlocking AirPlay session!");
        self.locked.send_replace(None);
    }
}
