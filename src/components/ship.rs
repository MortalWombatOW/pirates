use bevy::prelude::*;

/// Marker component that identifies an entity as a ship.
/// This is the primary marker for all vessels in the game, both player and AI-controlled.
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Ship;

/// Marker component that identifies an entity as the player's ship.
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

/// Marker component that identifies an entity as AI-controlled.
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct AI;

/// Marker component for ships owned by the player but controlled by AI (fleet members).
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct PlayerOwned;

/// Marker component for ships that have surrendered in combat.
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Surrendered;

/// Faction identifier for ships and ports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
pub enum FactionId {
    #[default]
    Pirates,
    NationA,
    NationB,
    NationC,
}

/// Component that assigns a faction to an entity.
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Faction(pub FactionId);

/// Ship class determines base stats and visual appearance.
/// Also used as a component to identify ship type for movement/turn rate calculations.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
#[reflect(Component)]
pub enum ShipType {
    /// Small, fast vessel. Low cargo, low firepower, high maneuverability.
    #[default]
    Sloop,
    /// Large military vessel. High firepower, moderate cargo, slow but sturdy.
    Frigate,
    /// Fast merchant vessel. Moderate cargo, low firepower, high speed.
    Schooner,
    /// Makeshift survival vessel. Minimal stats, slow, fragile.
    Raft,
}

impl ShipType {
    /// Returns the maximum turn rate in radians per second for this ship type.
    /// Smaller ships turn faster than larger ones.
    pub fn turn_rate(&self) -> f32 {
        match self {
            ShipType::Sloop => 2.5,    // ~143 degrees/sec - nimble
            ShipType::Raft => 2.0,     // ~115 degrees/sec - light but awkward
            ShipType::Schooner => 1.5, // ~86 degrees/sec - moderate
            ShipType::Frigate => 0.8,  // ~46 degrees/sec - slow to turn
        }
    }

    /// Returns the base speed for this ship type.
    pub fn base_speed(&self) -> f32 {
        match self {
            ShipType::Sloop => 300.0,
            ShipType::Raft => 150.0,
            ShipType::Schooner => 350.0,
            ShipType::Frigate => 200.0,
        }
    }
}
