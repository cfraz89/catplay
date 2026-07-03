use ring::{agreement, rand::SystemRandom, signature};

pub fn create_x25519_pubkey(data: &[u8; 32]) -> agreement::UnparsedPublicKey<[u8; 32]> {
    agreement::UnparsedPublicKey::new(&agreement::X25519, *data)
}

pub fn create_ed25519_pubkey(data: &[u8; 32]) -> signature::UnparsedPublicKey<[u8; 32]> {
    signature::UnparsedPublicKey::new(&signature::ED25519, *data)
}

pub fn create_x25519_key_ephermal() -> Result<agreement::EphemeralPrivateKey, ring::error::Unspecified> {
    let rng = SystemRandom::new();
    agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng)
}

pub fn x25519_agree_ephermal(
    my_private_key: agreement::EphemeralPrivateKey,
    peer_public_key: &agreement::UnparsedPublicKey<[u8; 32]>,
) -> Result<[u8; 32], ring::error::Unspecified> {
    let mut shared_secret = [0u8; 32];
    agreement::agree_ephemeral(my_private_key, peer_public_key, |shared| {
        shared_secret.copy_from_slice(shared);
    })?;
    Ok(shared_secret)
}

pub fn key32_from_vec(data: &[u8]) -> Option<[u8; 32]> {
    <[u8; 32]>::try_from(data).ok()
}
