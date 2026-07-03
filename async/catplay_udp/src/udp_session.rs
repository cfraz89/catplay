use std::{error::Error, io, net::SocketAddr};

use catplay_util::EventSleeper;

#[derive(Debug)]
pub enum UdpError {
    Io(io::Error),
}

pub trait UdpSocketPeer<F: UdpSession> {
    fn send(&self, data: &[u8]) -> Result<(), F::Error>;

    fn send_multiple(&self, data: &[&[u8]]) -> Result<usize, F::Error>;
}

#[allow(unused)]
pub trait UdpSession: Send + EventSleeper + 'static {
    type Error: From<io::Error> + Clone + Error + Send + Sync + 'static;

    /// Called when system forcefully closes the socket or an asynchronous ICMP response is observed indicating the remote is gone.
    ///
    /// Not guaranteed to be called if the session object is forcefully dropped.
    fn on_eof(&mut self, error: Option<Self::Error>) {}

    /// Called on a new datagram from an anonymous peer or if connect() was called, from the connected peer.
    ///
    /// The buffer is borrowed with mutation capability (like decryption) for duration of the call.
    fn on_datagram(&mut self, data: &mut [u8], peer: SocketAddr, sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error>;

    /// Called after successful call to connect().
    fn on_connect(&mut self, peer: SocketAddr, sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called after each sleep() to flush pending outgoing datagrams, reconcile internal state
    /// or communicate error state of the session by returning an error here.
    ///
    /// Peer will be `Some` if `connect()` was called at this stage.
    fn reconcile(&mut self, peer: Option<&dyn UdpSocketPeer<Self>>) -> Result<(), Self::Error> {
        Ok(())
    }
}
