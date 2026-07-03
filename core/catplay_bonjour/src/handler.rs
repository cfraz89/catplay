use crate::BonjourMeta;

pub trait BonjourHandler: Send + Sync {
    fn service_type(&self) -> &'static str;
    fn on_resolved(&self, meta: BonjourMeta) -> bool;
    fn on_removed(&self, service_type: &str, key: &str) -> bool;
    fn clear_cache(&self);
}
