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

/// Event emitted when a hostile encounter is triggered on the High Seas.
#[derive(Event, Debug)]
pub struct CombatTriggeredEvent {
    /// The enemy entity that triggered the encounter.
    pub enemy_entity: Entity,
    /// The faction of the enemy.
    pub enemy_faction: crate::components::FactionId,
}

/// Event emitted when a trade is executed at a port.
#[derive(Event, Debug)]
pub struct TradeExecutedEvent {
    /// The port entity where the trade occurred.
    pub port_entity: Entity,
    /// The type of good being traded.
    pub good_type: crate::components::cargo::GoodType,
    /// The quantity of goods traded.
    pub quantity: u32,
    /// True if buying from port, false if selling to port.
    pub is_buy: bool,
}
