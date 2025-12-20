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
