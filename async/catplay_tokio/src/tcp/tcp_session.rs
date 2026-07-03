use std::{
    io,
    net::{SocketAddr, TcpStream},
};

use async_trait::async_trait;
use bytes::BytesMut;
use catplay_util::{AsyncShutdown, EventSleeper};

pub use tokio_util::codec::Decoder;
pub use tokio_util::codec::Encoder;

use crate::TcpSink;

pub type CError<C> = <C as Decoder>::Error;
pub type CErrorEnc<C> = <C as Encoder<CItem<C>>>::Error;

pub type CItem<C> = <C as Decoder>::Item;

#[async_trait]
#[allow(unused)]
pub trait TcpSession: Send + AsyncShutdown + EventSleeper + 'static
where
    CItem<Self::Codec>: Send + 'static + std::fmt::Debug,
    CError<Self::Codec>: Send + std::fmt::Debug,
    CErrorEnc<Self::Codec>: Send,
{
    type Codec: Decoder + Encoder<CItem<Self::Codec>> + Send + Sync + 'static;
    type Error: Send
        + Sync
        + Clone
        + std::fmt::Debug
        + From<CErrorEnc<Self::Codec>>
        + From<CError<Self::Codec>>
        + From<io::Error>
        + Into<io::Error>;

    /// Called to allow configuration of socket-specific options on raw FD.
    fn init_stream(&mut self, _stream: &mut TcpStream) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Creates new instance of codec.
    fn init_codec(&mut self) -> Result<Self::Codec, Self::Error>;

    /// Reconciles session state after `on_msg` or `sleep`,
    /// with option to terminate the session prematurely by returning an error here.
    async fn reconcile(&mut self, _sink: &mut dyn TcpSink<Self>) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn on_connected(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn on_peer_addr(&mut self, peer_addr: SocketAddr) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn on_local_addr(&mut self, local_addr: SocketAddr) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called before socket is closed with final error or non-error state.
    /// No further `reconcile` calls are expected after this callback; and it's call is not guaranteed if the session is dropped.
    async fn on_eof(&mut self, status: Option<Self::Error>) {}

    /// Called on each received message with an option to write a response, or to close the channel by returning an error status.
    async fn on_msg(&mut self, sink: &mut dyn TcpSink<Self>, msg: CItem<Self::Codec>) -> Result<(), Self::Error>;
}

pub trait EncoderComposite<Item> {
    type Error: From<io::Error>;

    fn encode_composite(&mut self, item: Item, callback: &mut dyn FnMut(BytesMut)) -> Result<(), Self::Error>;

    fn encode_forced(&mut self, item: Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.encode_composite(item, &mut |b| dst.unsplit(b))
    }
}
