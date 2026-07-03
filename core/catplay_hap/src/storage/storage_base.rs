#[cfg(feature = "std")]
extern crate std;

use alloc::{string::String, sync::Arc, vec::Vec};
#[cfg(feature = "std")]
use catplay_util::ArcBox;
use core::fmt::Write;
use ring::signature::{Ed25519KeyPair, KeyPair};
use uuid::Uuid;

pub trait HomekitIdentity: Sync + Send {
    /// A persistent keypair of this device
    fn device_ed25519_keypair(&self) -> Arc<Ed25519KeyPair>;

    /// Unique uuid of this device
    fn device_id(&self) -> Uuid;

    /// Format public key in Bonjour "pk" format
    fn public_key_as_hex(&self) -> String {
        fn bytes_to_lower_hex(bytes: &[u8]) -> String {
            let mut out = String::with_capacity(bytes.len() * 2);
            for &b in bytes {
                let _ = write!(&mut out, "{b:02x}");
            }
            out
        }

        let key = self.device_ed25519_keypair();
        bytes_to_lower_hex(key.public_key().as_ref())
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum HomekitStorageError {
    #[error("Failed to flush HomeKit state to storage")]
    Failed,
    #[cfg(feature = "std")]
    #[error("I/O: {0}")]
    Io(ArcBox<std::io::Error>),
}

#[cfg(feature = "std")]
impl From<std::io::Error> for HomekitStorageError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(ArcBox::new(value))
    }
}

/// A persistent HomeKit storage, storing paired device keypairs.
pub trait HomekitStorage: Sync + Send + HomekitIdentity {
    /// Find previously paired device (it's pubkey) by uuid
    fn find_paired_by_id(&self, id: &Uuid) -> Option<[u8; 32]>;

    /// Unpairs a device: returns true if was paired, or false if not
    fn unpair(&self, id: &Uuid) -> Result<bool, HomekitStorageError>;

    /// List of paired uuids
    fn paired_ids(&self) -> Vec<Uuid>;

    /// Save paired device
    fn add_paired(&self, id: Uuid, key: [u8; 32]) -> Result<(), HomekitStorageError>;

    /// Save state to storage.
    fn flush(&self) -> Result<(), HomekitStorageError>;
}

pub type HomekitStorageRef = Arc<dyn HomekitStorage>;
