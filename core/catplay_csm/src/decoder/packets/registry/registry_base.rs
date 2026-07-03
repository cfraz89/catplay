extern crate alloc;

use alloc::{collections::BTreeMap, vec::Vec};
use core::any::TypeId;

use crate::decoder::{CsmDecode, CsmPacket, CsmPacketBox, CsmParamDecode};

struct PacketEntry {
    pub create: fn() -> CsmPacketBox,
    pub decode: fn(&[u8]) -> CsmPacketBox,
}

pub struct CsmPacketRegistry {
    id_to_entry: BTreeMap<u16, PacketEntry>,
    type_to_id: BTreeMap<TypeId, u16>,
}

pub struct CsmPacketRegistration {
    pub id: u16,
    pub register_fn: fn(&mut CsmPacketRegistry, u16),
}

impl CsmPacketRegistry {
    pub fn new() -> Self {
        Self {
            id_to_entry: BTreeMap::new(),
            type_to_id: BTreeMap::new(),
        }
    }

    pub fn register_type<T>(&mut self, id: u16)
    where
        T: 'static + Default + CsmPacket + CsmParamDecode,
    {
        let entry = PacketEntry {
            create: || T::default().into(),
            decode: |data| T::decode_from_bytes(data).into(),
        };
        if self.id_to_entry.insert(id, entry).is_some() {
            panic!("Attempted duplicate CSM registration at id 0x{id:04X}");
        }
        self.type_to_id.insert(TypeId::of::<T>(), id);
    }

    pub fn all_known_ids(&self) -> Vec<u16> {
        self.id_to_entry.keys().copied().collect()
    }

    pub fn contains_id(&self, id: u16) -> bool {
        self.id_to_entry.contains_key(&id)
    }

    pub fn create_by_id(&self, id: u16) -> Option<CsmPacketBox> {
        self.id_to_entry.get(&id).map(|f| (f.create)())
    }

    pub fn create_by_id_from_bytes(&self, id: u16, data: &[u8]) -> Option<CsmPacketBox> {
        self.id_to_entry.get(&id).map(|f| (f.decode)(data))
    }

    pub fn id_of(&self, pkt: &dyn CsmPacket) -> Option<u16> {
        self.type_to_id.get(&pkt.as_any().type_id()).copied()
    }

    pub const fn as_registration<T: 'static + Default + CsmPacket + CsmParamDecode>(id: u16) -> CsmPacketRegistration {
        CsmPacketRegistration {
            id,
            register_fn: |reg, id| reg.register_type::<T>(id),
        }
    }
}

impl Default for CsmPacketRegistry {
    fn default() -> Self {
        Self::new()
    }
}
