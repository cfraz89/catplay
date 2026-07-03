use std::sync::{Arc, Mutex};

use log::info;

use crate::BonjourMeta;

#[derive(Clone)]
pub struct BonjourEntryHandle {
    meta: Arc<BonjourMeta>,
    reannounce_fn: Arc<dyn Fn() + Send + Sync>,
    close_fn: Arc<dyn Fn() + Send + Sync>,
    closed: Arc<Mutex<bool>>,
}

impl BonjourEntryHandle {
    pub(crate) fn new(meta: BonjourMeta, reannounce_fn: Arc<dyn Fn() + Send + Sync>, close_fn: Arc<dyn Fn() + Send + Sync>) -> Self {
        Self {
            meta: Arc::new(meta),
            reannounce_fn,
            close_fn,
            closed: Arc::new(Mutex::new(false)),
        }
    }

    /// Force reannouncment but without unregistering.
    pub fn reannounce(&self) {
        if *self.closed.lock().unwrap() {
            return;
        }

        (self.reannounce_fn)();
    }

    /// Prevent further calls to [reannounce] from doing anything and unregister the service.
    pub fn close(&self) {
        let mut closed = self.closed.lock().unwrap();
        if !*closed {
            info!(
                "Stopping advertisement of Bonjour '{}' at {}",
                self.meta.instance_name, self.meta.service_type
            );

            (self.close_fn)();
            *closed = true;
        }
    }
}

impl Drop for BonjourEntryHandle {
    fn drop(&mut self) {
        // Auto-close only when the very last handle clone is dropped.
        // Dropping transient clones (e.g. inviter-owned) must not disable reannounce
        // on the primary owner.
        if Arc::strong_count(&self.meta) == 1 {
            self.close();
        }
    }
}
