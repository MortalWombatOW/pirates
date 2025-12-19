use bevy::prelude::*;

use crate::plugins::core::GameState;
use crate::systems::{
    buffer_ship_input, 
    ship_physics_system, 
    debug_ship_physics,
    cannon_firing_system,
    consume_firing_input,
    projectile_system,
    projectile_collision_system,
    target_cycling_system,
    spawn_test_target,
    ship_destruction_system,
    handle_player_death_system,
    loot_collection_system,
    loot_timer_system,
    ShipInputBuffer,
    ShipPhysicsConfig,
};
use crate::resources::CannonState;

/// Plugin that manages all combat-related systems.
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        // Register events
        app.add_event::<crate::events::ShipDestroyedEvent>();
        
        // Initialize resources
        app.init_resource::<ShipInputBuffer>()
            .init_resource::<ShipPhysicsConfig>()
            .init_resource::<CannonState>();
        
        // Buffer input in Update
        app.add_systems(
            Update,
            buffer_ship_input.run_if(in_state(GameState::Combat)),
        );
        
        // Apply physics forces and firing in FixedUpdate
        app.add_systems(
            FixedUpdate,
            (
                ship_physics_system,
                cannon_firing_system,
                consume_firing_input.after(cannon_firing_system),
                target_cycling_system,
            ).run_if(in_state(GameState::Combat)),
        );
        
        // General combat systems in Update
        app.add_systems(
            Update,
            (
                projectile_system,
                projectile_collision_system,
                loot_collection_system.after(projectile_collision_system),
                loot_timer_system,
                debug_ship_physics,
                ship_destruction_system.after(projectile_collision_system),
                handle_player_death_system.after(ship_destruction_system),
            ).run_if(in_state(GameState::Combat)),
        );

        // Spawn test target on enter
        app.add_systems(
            OnEnter(GameState::Combat),
            spawn_test_target,
        );
    }
}

