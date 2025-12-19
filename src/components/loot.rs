use bevy::prelude::*;
use crate::components::GoodType;

/// Marker component for floating loot entities.
/// Loot spawns when ships are hit and can be collected by the player.
#[derive(Component, Debug, Clone)]
pub struct Loot {
    /// Gold value of this loot item.
    pub value: u32,
    /// Optional cargo type (if this loot represents goods rather than gold).
    pub good_type: Option<GoodType>,
}

impl Loot {
    /// Creates a gold loot item with the specified value.
    pub fn gold(value: u32) -> Self {
        Self {
            value,
            good_type: None,
        }
    }

    /// Creates a cargo loot item with the specified good type and value.
    pub fn cargo(good_type: GoodType, value: u32) -> Self {
        Self {
            value,
            good_type: Some(good_type),
        }
    }
}

impl Default for Loot {
    fn default() -> Self {
        Self::gold(10)
    }
}

/// Timer component for loot despawning after a period.
#[derive(Component)]
pub struct LootTimer(pub Timer);

impl Default for LootTimer {
    fn default() -> Self {
        // Loot despawns after 30 seconds if not collected
        Self(Timer::from_seconds(30.0, TimerMode::Once))
    }
}
