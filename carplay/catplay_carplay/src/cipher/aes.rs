use catplay_hap::sha512_parts;

pub fn derive_aes_stream_keys(session_key: &[u8; 16], stream_connection_id: u64) -> ([u8; 16], [u8; 16]) {
    fn digest16(prefix: &str, stream_connection_id: u64, audio_aes_key: &[u8; 16]) -> [u8; 16] {
        sha512_parts(&[prefix.as_bytes(), stream_connection_id.to_string().as_bytes(), audio_aes_key])[..16]
            .try_into()
            .unwrap()
    }

    (
        digest16("AirPlayStreamKey", stream_connection_id, session_key),
        digest16("AirPlayStreamIV", stream_connection_id, session_key),
    )
}
