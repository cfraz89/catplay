extern crate std;

use std::sync::OnceLock;

use crate::decoder::packets::registry::CsmPacketRegistration;

use super::CsmPacketRegistry;

static REGISTRY: OnceLock<CsmPacketRegistry> = OnceLock::new();

impl CsmPacketRegistry {
    pub fn static_registry() -> &'static CsmPacketRegistry {
        get_runtime_csm_registry()
    }
}

fn get_runtime_csm_registry() -> &'static CsmPacketRegistry {
    REGISTRY.get_or_init(|| {
        let mut registry = CsmPacketRegistry::new();

        for reg in inventory::iter::<CsmPacketRegistration> {
            (reg.register_fn)(&mut registry, reg.id);
        }
        registry
    })
}

inventory::collect!(CsmPacketRegistration);
