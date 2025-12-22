use bevy::prelude::*;

/// Marker component that identifies an entity as a ship.
/// This is the primary marker for all vessels in the game, both player and AI-controlled.
#[derive(Component, Debug, Default)]
pub struct Ship;

/// Marker component that identifies an entity as the player's ship.
#[derive(Component, Debug, Default)]
pub struct Player;

/// Marker component that identifies an entity as AI-controlled.
#[derive(Component, Debug, Default)]
pub struct AI;

/// Marker component for ships owned by the player but controlled by AI (fleet members).
#[derive(Component, Debug, Default)]
pub struct PlayerOwned;

/// Marker component for ships that have surrendered in combat.
#[derive(Component, Debug, Default)]
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
pub struct Faction(pub FactionId);

/// Ship class determines base stats and visual appearance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
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
