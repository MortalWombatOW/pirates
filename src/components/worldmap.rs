use bevy::prelude::*;

/// Component that defines the vision range of an entity (e.g., player ship).
/// Used by the `FogOfWarSystem` to reveal tiles on the world map.
#[derive(Component, Debug, Clone, Copy)]
pub struct Vision {
    /// The radius of vision in tiles.
    pub radius: f32,
}

impl Default for Vision {
    fn default() -> Self {
        Self { radius: 8.0 }
    }
}
