use bevy::prelude::*;

/// Marker component that identifies an entity as a ship.
/// This is the primary marker for all vessels in the game, both player and AI-controlled.
#[derive(Component, Debug, Default)]
pub struct Ship;

/// Marker component that identifies an entity as the player's ship.
#[derive(Component, Debug, Default)]
pub struct Player;
