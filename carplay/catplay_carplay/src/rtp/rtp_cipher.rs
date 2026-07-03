use crate::{
    rtp::{RTP_HEADER_SIZE, RTP_PACKET_MAX, RtpAesCbcDecoder, RtpChaChaDecoder, RtpHeader, RtpPacketBorrow},
    rtsp_frame::{RtspError, RtspResult},
};

#[allow(clippy::large_enum_variant)]
pub enum RtpCipher {
    ChaCha(RtpChaChaDecoder),
    AesCbc(RtpAesCbcDecoder),
    None,
}

impl RtpCipher {
    pub fn decode<'a>(&mut self, src: &'a mut [u8]) -> RtspResult<RtpPacketBorrow<'a>> {
        match self {
            RtpCipher::ChaCha(cipher) => cipher.decode_rtp_payload(src),
            RtpCipher::AesCbc(cipher) => cipher.decode_rtp_payload(src),
            RtpCipher::None => {
                let n = src.len();

                if n > RTP_PACKET_MAX {
                    return Err(RtspError::PayloadTooBig(n, RTP_PACKET_MAX));
                }

                if n < RTP_HEADER_SIZE {
                    return Err(RtspError::PayloadTooShort(n, RTP_HEADER_SIZE));
                }

                let header = RtpHeader::from_buf(src).ok_or(RtspError::RtpInvalidHeader)?;
                let payload = &mut src[RTP_HEADER_SIZE..];
                if payload.is_empty() {
                    return Err(RtspError::Empty);
                }

                Ok(RtpPacketBorrow::new(header, payload))
            }
        }
    }

    pub fn encode<'a>(&mut self, output: &'a mut [u8], plaintext_len: usize, header: &RtpHeader) -> RtspResult<&'a mut [u8]> {
        match self {
            RtpCipher::ChaCha(cipher) => cipher.encode_rtp_payload_in_place(output, plaintext_len, header),
            RtpCipher::AesCbc(cipher) => cipher.encode_rtp_payload_in_place(output, plaintext_len, header),
            RtpCipher::None => {
                let total_len = RTP_HEADER_SIZE + plaintext_len;
                if total_len > RTP_PACKET_MAX {
                    return Err(RtspError::PayloadTooBig(total_len, RTP_PACKET_MAX));
                }

                if output.len() < total_len {
                    return Err(RtspError::RtpTooBigForBuffer(plaintext_len, output.len()));
                }

                let header_buf = header.to_buf();
                output[..RTP_HEADER_SIZE].copy_from_slice(&header_buf);

                Ok(&mut output[..total_len])
            }
        }
    }
}
