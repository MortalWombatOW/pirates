use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;

use crate::plugins::core::GameState;
use crate::plugins::fluid_simulation::FluidSimulationPlugin;
use crate::resources::WaterMaterial;
use crate::systems::{
    buffer_ship_input, 
    ship_physics_system, 
    debug_ship_physics,
    cannon_firing_system,
    consume_firing_input,
    projectile_system,
    projectile_collision_system,
    target_cycling_system,
    ship_destruction_system,
    handle_player_death_system,
    loot_collection_system,
    loot_timer_system,
    current_zone_system,
    spawn_test_current_zone,
    combat_victory_system,
    handle_combat_victory_system,
    // AI systems
    combat_ai_system,
    ai_firing_system,
    spawn_combat_enemies,
    AIPhysicsConfig,
    ShipInputBuffer,
    ShipPhysicsConfig,
};
use crate::systems::camera::{camera_shake_system, trigger_camera_shake_on_fire};
use crate::systems::hit_flash::{trigger_hit_flash_system, update_hit_flash_system};
use crate::resources::CannonState;

/// Plugin that manages all combat-related systems.
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        // Add fluid simulation plugin for dynamic water
        app.add_plugins((
            FluidSimulationPlugin,
            Material2dPlugin::<WaterMaterial>::default(),
        ));

        // Register events
        app.add_event::<crate::events::ShipDestroyedEvent>()
            .add_event::<crate::events::CombatEndedEvent>()
            .add_event::<crate::events::ShipHitEvent>()
            .add_event::<crate::events::CannonFiredEvent>();
        
        // Initialize resources
        app.init_resource::<ShipInputBuffer>()
            .init_resource::<ShipPhysicsConfig>()
            .init_resource::<CannonState>()
            .init_resource::<AIPhysicsConfig>();
        
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
                // AI systems - run after player physics is processed
                combat_ai_system.after(ship_physics_system),
                ai_firing_system.after(combat_ai_system),
                // TODO: Remove current_zone_system when fluid sim drift is ready
                current_zone_system.after(combat_ai_system),
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
                combat_victory_system.after(ship_destruction_system),
                handle_combat_victory_system.after(combat_victory_system),
                // Camera shake visual effects
                trigger_camera_shake_on_fire,
                camera_shake_system.after(trigger_camera_shake_on_fire),
                // Hit flash visual effects
                trigger_hit_flash_system.after(projectile_collision_system),
                update_hit_flash_system.after(trigger_hit_flash_system),
            ).run_if(in_state(GameState::Combat)),
        );

        // Spawn combat entities on enter
        // TODO: Remove spawn_test_current_zone when fluid sim is fully integrated
        app.add_systems(
            OnEnter(GameState::Combat),
            (spawn_combat_enemies, spawn_test_current_zone),
        );
    }
}

