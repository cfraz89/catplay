use core::any::Any;
use core::fmt::Debug;

use super::super::*;

pub trait CsmPacketId {
    const PACKET_ID: u16;
}

pub trait CsmPacket: CsmParamEncodeBytes + Any + Debug + Send + Sync + CsmPacketClone + 'static {}
impl<T> CsmPacket for T where T: CsmParamEncodeBytes + Any + Debug + Send + Sync + CsmPacketClone + 'static {}

impl AsRef<dyn CsmPacket> for dyn CsmPacket {
    fn as_ref(&self) -> &dyn CsmPacket {
        self
    }
}

pub trait AsCsmPacket: Send + Sync {
    fn as_csm(&self) -> &dyn CsmPacket;
}

impl<T> AsCsmPacket for T
where
    T: AsRef<dyn CsmPacket> + Send + Sync,
{
    fn as_csm(&self) -> &dyn CsmPacket {
        self.as_ref()
    }
}

impl dyn CsmPacket {
    pub fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    /// Casts CsmPacket to it's implementation subtype or returns None if not a match
    pub fn cast<T: CsmPacket>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}
