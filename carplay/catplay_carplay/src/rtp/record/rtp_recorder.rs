use std::{error::Error, num::NonZeroUsize, time::Instant};

use catplay_util::ModSeq;
use log::{debug, trace, warn};

use crate::{
    audio::{RingBuffer, RingConsumer, RingProducer, codec::AudioEncoder},
    msg::StreamType,
    rtp::{RTP_HEADER_SIZE, RTP_PACKET_OVERHEAD_CHACHA, RtpChaChaBuffer, RtpCipher, RtpHeader},
    rtsp_frame::RtspError,
};

pub struct RtpRecorder<T: AudioEncoder> {
    ring: RingConsumer<T::Sample>,

    cipher: RtpCipher,

    stream_type: StreamType,
    ts: ModSeq<u32>,
    seq: ModSeq<u16>,
    encoder: T,

    samples_per_packet: NonZeroUsize,
    frame_size_in_samples: NonZeroUsize,
    sent_packets: u64,
    last_send: Option<Instant>,
}

/// Critical error in RTP recorder session that should terminate it; non-critical errors
/// result in a timeline jump instead.
#[derive(thiserror::Error, Debug)]
pub enum RtpRecorderError<T: Error> {
    #[error("{0}")]
    Codec(T),
    #[error("{0}")]
    Rtsp(#[from] RtspError),
    #[error("Overflow")]
    Overflow,
}

impl<T: AudioEncoder> RtpRecorder<T> {
    const PACKETS_RING_MULTIPLIER: usize = 50;

    pub fn new(
        encoder: T,
        stream_type: StreamType,
        samples_per_packet: NonZeroUsize,
        frame_size_in_samples: NonZeroUsize,
        cipher: RtpCipher,
    ) -> (Self, RingProducer<T::Sample>) {
        let (producer, ring) = RingBuffer::spsc(samples_per_packet.get() * Self::PACKETS_RING_MULTIPLIER);

        (
            RtpRecorder {
                ring,
                cipher,
                stream_type,
                ts: ModSeq(0),
                seq: ModSeq(0),
                encoder,
                samples_per_packet,
                frame_size_in_samples,
                sent_packets: 0,
                last_send: None,
            },
            producer,
        )
    }

    pub fn encode_into(encoder: &mut T, frames: &[T::Sample], payload: &mut [u8]) -> Result<(usize, usize), RtpRecorderError<T::Error>> {
        let packet_samples = frames.len();

        match encoder.encode(frames, payload) {
            Err(err) => {
                warn!("Failed to encode audio, skipping by {packet_samples} samples: {err}");
                Err(RtpRecorderError::Codec(err))
            }
            Ok((written, consumed)) => {
                if consumed != packet_samples {
                    warn!("Encoder did not consume all samples! written={written} consumed={consumed}/{packet_samples}");
                    return Err(RtpRecorderError::Overflow);
                }

                Ok((written, consumed))
            }
        }
    }

    pub fn prepare_header(stream_type: u8, ts: ModSeq<u32>, seq: ModSeq<u16>) -> RtpHeader {
        RtpHeader {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: false,
            payload_type: stream_type,
            sequence_number: seq.0,
            timestamp: ts.0,
            ssrc: 0,
        }
    }

    pub fn prepare_packet(
        cipher: &mut RtpCipher,
        encoder: &mut T,
        frames: &[T::Sample],
        buf: &mut RtpChaChaBuffer,
        stream_type: u8,
        ts: ModSeq<u32>,
        seq: ModSeq<u16>,
    ) -> Result<(), RtpRecorderError<T::Error>> {
        // Take an aligned buffer and start writing payload past the 12 bytes RTP header mark.
        let buf_unsafe = unsafe { buf.uninit_slice_mut() };
        let buf_len = buf_unsafe.len();
        let payload = &mut buf_unsafe[RTP_HEADER_SIZE..buf_len - RTP_PACKET_OVERHEAD_CHACHA];

        let (written, _consumed) = Self::encode_into(encoder, frames, payload)?;

        let header = Self::prepare_header(stream_type, ts, seq);
        trace!("Header: {header:?}");
        let len = cipher.encode(buf_unsafe, written, &header)?.len();
        unsafe { buf.assume_written(len) }

        Ok(())
    }

    pub fn drain<F: FnMut(&[&[u8]])>(&mut self, mut callback: F) -> Result<(), RtpRecorderError<T::Error>> {
        while self.drain_batch(&mut callback)? {}
        Ok(())
    }

    fn drain_batch(&mut self, callback: &mut dyn FnMut(&[&[u8]])) -> Result<bool, RtpRecorderError<T::Error>> {
        const UDP_BATCH_SIZE: usize = 16;

        let packet_samples = self.samples_per_packet.get();
        let mut ts = self.ts;
        let mut seq = self.seq;
        let mut samples_to_commit = 0;

        let (overflow, readable) = self.ring.readable_slice();
        if overflow > 0 {
            // TODO: probably wrong
            warn!("Overflow during recording; skipping {overflow} samples");
            samples_to_commit += overflow;
            ts = ts + ModSeq(overflow as u32 / self.frame_size_in_samples.get() as u32);
        }

        let packets = readable.len() / packet_samples;

        let mut packets_prepared = 0;
        let mut batch = RtpChaChaBuffer::new_batch::<UDP_BATCH_SIZE>();

        {
            for i in 0..packets.min(batch.len()) {
                let start = i * packet_samples;
                let frames = &readable[start..start + packet_samples];
                let buffer = &mut batch[packets_prepared];

                Self::prepare_packet(&mut self.cipher, &mut self.encoder, frames, buffer, self.stream_type as _, ts, seq)?;
                let ts_jump = packet_samples / self.frame_size_in_samples.get();
                ts = ts + ModSeq(ts_jump as u32);
                seq = seq + ModSeq(1);

                samples_to_commit += packet_samples;
                packets_prepared += 1;
            }
        }

        if samples_to_commit > 0 {
            self.ring.read_commit(samples_to_commit);
        }

        self.ts = ts;
        self.seq = seq;

        let done = packets_prepared == packets;

        if packets_prepared > 0 {
            self.sent_packets += packets_prepared as u64;
            let flush_delta = self.last_send.map(|last_send| Instant::now() - last_send);

            debug!(
                "Flushing {packets_prepared} packets / sum {} / consumed samples {samples_to_commit} / flush delta {flush_delta:?}",
                self.sent_packets
            );
            let send_batch = &RtpChaChaBuffer::build_send_slices(&batch)[..packets_prepared];
            (callback)(send_batch);
            self.last_send.replace(Instant::now());
        }

        Ok(!done)
    }
}
