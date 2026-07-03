use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use catplay_tokio::{UdpSession, UdpSocketPeer};
use catplay_util::{EventSleeper, EventToken};
use log::debug;

use crate::{
    audio::{AudioPlayerBox, codec::AudioDecoder},
    cipher::AirPlayStreamEncryption,
    rtp::{
        AsRtpPacket, RtpAesCbcDecoder, RtpChaChaDecoder, RtpCipher,
        play::{RtpConsumer, RtpPlayer},
    },
    rtsp_frame::{RtspError, RtspResult},
};

pub struct RtpSession<C: AudioDecoder> {
    codec: RtpCipher,

    player: Option<RtpPlayer<C>>,
    sink: Option<AudioPlayerBox<C::Sample>>,
    playing: bool,
}

impl<C: AudioDecoder> RtpSession<C> {
    pub fn new(
        encryption: AirPlayStreamEncryption,
        stream_connection_id: u64,
        latency_ms: Duration,
        codec: C,

        sink: AudioPlayerBox<C::Sample>,
        server: bool,
    ) -> RtspResult<Self> {
        let cipher = match encryption {
            AirPlayStreamEncryption::Unconfigured | AirPlayStreamEncryption::None => RtpCipher::None,
            AirPlayStreamEncryption::Aes { key, iv } => RtpCipher::AesCbc(RtpAesCbcDecoder::new(&key, &iv)),
            AirPlayStreamEncryption::ChaCha { shared_secret } => {
                RtpCipher::ChaCha(RtpChaChaDecoder::new(shared_secret, stream_connection_id, server))
            }
        };
        Self::new_with_cipher(cipher, latency_ms, codec, sink)
    }

    fn new_with_cipher(cipher: RtpCipher, latency_ms: Duration, codec: C, mut sink: AudioPlayerBox<C::Sample>) -> RtspResult<Self> {
        let sample_rate = codec.output_type().sample_rate as usize;
        let frame_size_in_samples: usize = codec.output_type().channels_per_frame as _;

        let start = Instant::now();
        // RtpPlayer -> feeds audio ring as producer
        let (player, ring) = RtpPlayer::new(
            codec,
            sample_rate.try_into().unwrap(),
            frame_size_in_samples.try_into().unwrap(),
            latency_ms,
            start,
        );
        // RtpConsumer -> owns audio ring consumer side and provides data to AudioPlayer (sink)
        let consumer = RtpConsumer::new(
            ring,
            sample_rate.try_into().unwrap(),
            frame_size_in_samples.try_into().unwrap(),
            start,
        );

        sink.init(Box::new(consumer))?;

        let mut me = Self {
            codec: cipher,
            player: Some(player),
            sink: Some(sink),
            playing: false,
        };

        me.play_or_pause_if_needed();
        Ok(me)
    }

    fn play_or_pause_if_needed(&mut self) {
        let Some(player) = self.player.as_mut() else {
            return;
        };

        if let Some(sink) = self.sink.as_mut() {
            if player.should_play() && !self.playing {
                debug!("Sending play command to sink");
                sink.start();
                self.playing = true
            } else if !player.should_play() && self.playing {
                debug!("Sending pause command to sink");
                sink.stop(true);
                self.playing = false;
            }
        }
    }

    fn on_recv(&mut self, data: &mut [u8]) -> RtspResult<()> {
        // TODO: detect player `read_head` stall, rewind and restart
        // TODO: timeout when there are no incoming packets for too long

        let packet = self.codec.decode(data)?;
        debug!("New RTP packet: {:?}", packet.header());

        if let Some(player) = self.player.as_mut() {
            player.push(&packet);
        }
        self.play_or_pause_if_needed();

        Ok(())
    }
}

impl<C: AudioDecoder> UdpSession for RtpSession<C> {
    type Error = RtspError;

    fn on_datagram(&mut self, data: &mut [u8], _peer: SocketAddr, _sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
        self.on_recv(data)
    }

    fn on_eof(&mut self, error: Option<Self::Error>) {
        debug!("Detected Eof on RTP socket: {error:?}");
    }

    fn on_connect(&mut self, peer: SocketAddr, _sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
        debug!("Connected to remote {peer}");
        Ok(())
    }
}

impl<C: AudioDecoder> EventSleeper for RtpSession<C> {
    async fn sleep(&mut self) -> Option<EventToken> {
        None
    }
}

impl<C: AudioDecoder> Drop for RtpSession<C> {
    fn drop(&mut self) {
        // Stop the player
        if let Some(sink) = self.sink.as_mut() {
            sink.stop(true);
        }
    }
}
