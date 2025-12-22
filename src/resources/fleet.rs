use bevy::prelude::*;
use crate::components::Cargo;

/// Data structure to persist a ship's state across game states (Combat -> High Seas).
#[derive(Clone, Debug, Reflect)]
pub struct ShipData {
    /// Visual representation (sprite path).
    pub sprite_path: String,
    /// Current hull health.
    pub hull_health: f32,
    /// Max hull health.
    pub max_hull_health: f32,
    /// Current cargo.
    pub cargo: Option<Cargo>,
    /// Name of the ship.
    pub name: String,
}

impl Default for ShipData {
    fn default() -> Self {
        Self {
            sprite_path: "sprites/ships/round_ship_small.png".to_string(),
            hull_health: 100.0,
            max_hull_health: 100.0,
            cargo: None,
            name: "Captured Ship".to_string(),
        }
    }
}

/// Resource that tracks the player's fleet of ships.
#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct PlayerFleet {
    /// List of ships owned by the player (excluding the flagship).
    pub ships: Vec<ShipData>,
}

/// Resource mapping PlayerFleet indices to spawned Entity IDs.
/// Populated when entering HighSeas, cleared when leaving.
#[derive(Resource, Default)]
pub struct FleetEntities {
    /// Spawned entities corresponding to PlayerFleet.ships by index.
    pub entities: Vec<Entity>,
}
