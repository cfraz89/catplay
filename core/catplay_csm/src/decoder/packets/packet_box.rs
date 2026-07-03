extern crate alloc;

use alloc::boxed::Box;
use core::any::Any;
use core::fmt::Debug;

use crate::decoder::{CsmPacket, CsmParamEncodeBytes};

pub type CsmPacketBox = Box<dyn CsmPacket>;

impl<T: CsmPacket + 'static> From<T> for CsmPacketBox {
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

pub trait CsmPacketClone {
    fn clone_box(&self) -> CsmPacketBox;
}

impl<T> CsmPacketClone for T
where
    T: 'static + CsmParamEncodeBytes + Any + Debug + Send + Sync + Clone,
{
    fn clone_box(&self) -> CsmPacketBox {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn CsmPacket> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
