use bevy::prelude::*;

use crate::plugins::core::GameState;
use crate::systems::ship_movement_system;

/// Plugin that manages all combat-related systems.
/// 
/// Systems run only when in `GameState::Combat`.
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        // Run movement on Update (not FixedUpdate) because input detection
        // happens in PreUpdate and may not be visible in FixedUpdate
        app.add_systems(
            Update,
            ship_movement_system.run_if(in_state(GameState::Combat)),
        );
    }
}
