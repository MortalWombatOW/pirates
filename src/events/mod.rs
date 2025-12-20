use bevy::prelude::*;

/// Event emitted when a ship is destroyed (hull HP <= 0).
#[derive(Event, Debug)]
pub struct ShipDestroyedEvent {
    /// The entity that was destroyed.
    pub entity: Entity,
    /// Whether this was the player's ship.
    pub was_player: bool,
}

/// Event emitted when combat ends (all enemies destroyed or player flees).
#[derive(Event, Debug)]
pub struct CombatEndedEvent {
    /// True if player won (all enemies destroyed), false if fled.
    pub victory: bool,
}
