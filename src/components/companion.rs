use bevy::prelude::*;

/// Marker component for a Companion entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Companion;

/// Role of a companion, determining their special ability.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompanionRole {
    /// Quartermaster: Auto-trades based on market intel.
    Quartermaster,
    /// Navigator: Auto-routes efficient paths.
    Navigator,
    /// Lookout: Increases vision radius.
    Lookout,
    /// Gunner: Reduces cannon cooldown.
    Gunner,
    /// Mystic: Enables magic abilities (future).
    Mystic,
}

impl CompanionRole {
    pub fn name(&self) -> &'static str {
        match self {
            CompanionRole::Quartermaster => "Quartermaster",
            CompanionRole::Navigator => "Navigator",
            CompanionRole::Lookout => "Lookout",
            CompanionRole::Gunner => "Gunner",
            CompanionRole::Mystic => "Mystic",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CompanionRole::Quartermaster => "Auto-trades heavily for profit.",
            CompanionRole::Navigator => "Optimizes sailing routes.",
            CompanionRole::Lookout => "Increases vision range on high seas.",
            CompanionRole::Gunner => "Reduces cannon reload time.",
            CompanionRole::Mystic => "Unlocks arcane abilities.",
        }
    }
}

/// The display name of the companion.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct CompanionName(pub String);

/// Links a companion to a specific ship entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssignedTo(pub Entity);
