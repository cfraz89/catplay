use std::sync::Arc;

use crate::{CsmRemote, CsmSessionResult, CsmSessionStatus};
use catplay_csm::decoder::{AsCsmPacket, CsmPacketBox};

pub trait CsmClientHandle: Send + Sync {
    /// For Bluetooth transport, this closes the RFCOMM connection.
    ///
    /// For USB gadget(accessory), it stops signaling read interest to the other side until reconnect.
    ///
    /// For USB host(iPhone), it halts the endpoint.
    ///
    /// In other modes, behavior is undefined.
    fn disconnect(&self);

    /// Gets stream direction (iPhone vs accessory).
    fn is_server(&self) -> bool;

    /// Checks if channel if closed.
    fn is_closed(&self) -> bool;

    /// Checks if channel is writable.
    ///
    /// A channel is writable after passing link detect, negotitation and as long as
    /// enough previous packets were acknowledged by the other side.
    fn is_writable(&self) -> bool;

    /// Gets remote address.
    fn remote(&self) -> &CsmRemote;

    /// Gets session status.
    fn status(&self) -> CsmSessionStatus;

    /// Sends a single packet.
    ///
    /// If channel is unwritable at time of call, packet will be buffered until it becomes writable again.
    ///
    /// I/O error will only be returned if packet is not serializable or channel is already closed at the time of call.
    fn send(&self, packet: &dyn AsCsmPacket) -> CsmSessionResult<()>;

    /// Sends multiple packets.
    fn send_all(&self, packets: &[CsmPacketBox]) -> CsmSessionResult<()>;

    fn send_file_reserve(&self) -> Option<u8>;

    fn send_file(&self, file_id: u8, file_type: u16, setup_data: &[u8], source: Vec<u8>);
}

pub type CsmClientHandleRef = Arc<dyn CsmClientHandle>;
