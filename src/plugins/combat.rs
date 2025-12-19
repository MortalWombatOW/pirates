use bevy::prelude::*;

use crate::plugins::core::GameState;
use crate::systems::{
    buffer_ship_input, 
    ship_physics_system, 
    debug_ship_physics,
    ShipInputBuffer,
    ShipPhysicsConfig,
};

/// Plugin that manages all combat-related systems.
/// 
/// **Architecture:**
/// - Input is buffered in `Update` (to catch all input events)
/// - Physics forces are applied in `FixedUpdate` (for deterministic simulation)
/// - Avian physics runs in `FixedPostUpdate` (handles force integration)
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.init_resource::<ShipInputBuffer>()
            .init_resource::<ShipPhysicsConfig>();
        
        // Buffer input in Update (catches all input events)
        app.add_systems(
            Update,
            buffer_ship_input.run_if(in_state(GameState::Combat)),
        );
        
        // Apply physics forces in FixedUpdate (before Avian physics runs in FixedPostUpdate)
        app.add_systems(
            FixedUpdate,
            ship_physics_system.run_if(in_state(GameState::Combat)),
        );
        
        // Debug logging (can be removed in production)
        app.add_systems(
            Update,
            debug_ship_physics.run_if(in_state(GameState::Combat)),
        );
    }
}

