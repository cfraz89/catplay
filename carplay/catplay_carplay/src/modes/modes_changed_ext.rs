use crate::modes::{EntityEnum, ResourceTransferType, SpeechMode};

/// [ResourceState] can be derived from a given pair of `entity` and `permanent_entity`,
/// making the state (`Take`/`Borrow`) easier to understand.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResourceState {
    ControllerHas,
    AccessoryHas,
    ControllerBorrowed,
    AccessoryBorrowed,

    /// Invalid state or unclaimed by either side.
    #[default]
    Invalid,
}

impl ResourceState {
    pub const fn from_pair(entity: EntityEnum, permanent_entity: EntityEnum) -> Self {
        match (entity, permanent_entity) {
            (EntityEnum::Controller, EntityEnum::Controller) => ResourceState::ControllerHas,
            (EntityEnum::Accessory, EntityEnum::Accessory) => ResourceState::AccessoryHas,
            (EntityEnum::Controller, EntityEnum::Accessory) => ResourceState::ControllerBorrowed,
            (EntityEnum::Accessory, EntityEnum::Controller) => ResourceState::AccessoryBorrowed,
            //  This will never happen for a properly defined resource
            _ => ResourceState::Invalid,
        }
    }

    pub const fn to_pair(&self) -> (EntityEnum, EntityEnum) {
        match self {
            ResourceState::ControllerHas => (EntityEnum::Controller, EntityEnum::Controller),
            ResourceState::AccessoryHas => (EntityEnum::Accessory, EntityEnum::Accessory),
            ResourceState::ControllerBorrowed => (EntityEnum::Controller, EntityEnum::Accessory),
            ResourceState::AccessoryBorrowed => (EntityEnum::Accessory, EntityEnum::Controller),
            // Unclaimed resources are externally represented as fully owned by the `Controller`
            ResourceState::Invalid => (EntityEnum::Controller, EntityEnum::Controller),
        }
    }

    pub const fn is_borrowed(&self) -> bool {
        matches!(self, ResourceState::ControllerBorrowed | ResourceState::AccessoryBorrowed)
    }

    /// Current entity allowed to use the resource.
    pub const fn entity(&self) -> EntityEnum {
        match self {
            ResourceState::AccessoryBorrowed | ResourceState::AccessoryHas => EntityEnum::Accessory,
            ResourceState::ControllerBorrowed | ResourceState::ControllerHas => EntityEnum::Controller,
            _ => EntityEnum::None,
        }
    }

    /// Permanent owner of the resource.
    pub const fn owner(&self) -> EntityEnum {
        match self {
            ResourceState::ControllerHas | ResourceState::AccessoryBorrowed => EntityEnum::Controller,
            ResourceState::AccessoryHas | ResourceState::ControllerBorrowed => EntityEnum::Accessory,
            // Unclaimed resources are externally represented as fully owned by the `Controller`
            ResourceState::Invalid => EntityEnum::None,
        }
    }

    /// Derive [ResourceState] from a request sent from `Accessory` towards the `Controller`.
    pub const fn from_request(t: ResourceTransferType) -> Self {
        match t {
            ResourceTransferType::Take => ResourceState::AccessoryHas,
            ResourceTransferType::Untake => ResourceState::ControllerHas,
            ResourceTransferType::Borrow => ResourceState::AccessoryBorrowed,
            ResourceTransferType::Unborrow => ResourceState::ControllerHas,
        }
    }

    pub const fn is_unclaimed(&self) -> bool {
        matches!(self, ResourceState::Invalid)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpeechState {
    pub entity: EntityEnum,
    pub mode: SpeechMode,
}

impl Default for SpeechState {
    fn default() -> Self {
        Self {
            entity: EntityEnum::None,
            mode: SpeechMode::None,
        }
    }
}
