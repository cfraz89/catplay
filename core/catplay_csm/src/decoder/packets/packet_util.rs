use crate::decoder::{AsCsmPacket, CsmPacket};

pub trait CsmPacketUtil {
    fn predicate(f: impl Fn(&Self) -> bool) -> impl Fn(&dyn AsCsmPacket) -> bool;

    fn cast(data: &dyn AsCsmPacket) -> Option<&Self>;
}

impl<T: CsmPacket> CsmPacketUtil for T {
    fn predicate(f: impl Fn(&Self) -> bool) -> impl Fn(&dyn AsCsmPacket) -> bool {
        move |b| {
            let Some(packet) = b.as_csm().cast::<Self>() else {
                return false;
            };

            f(packet)
        }
    }

    /// Casts CsmPacket to it's implementation subtype or returns None if not a match
    fn cast(data: &dyn AsCsmPacket) -> Option<&Self> {
        data.as_csm().cast()
    }
}
