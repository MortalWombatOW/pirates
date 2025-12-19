use bevy::prelude::*;
use avian2d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::components::{Ship, Player, Health};
use crate::plugins::input::PlayerAction;

/// Configuration for ship movement parameters.
/// These can be adjusted for game balance.
pub struct ShipMovementConfig {
    /// Base thrust force applied when W is pressed.
    pub thrust_force: f32,
    /// Base reverse force applied when S is pressed (typically lower than thrust).
    pub reverse_force: f32,
    /// Base angular velocity applied when turning (radians/second).
    pub turn_rate: f32,
    /// Maximum linear speed (units/second).
    pub max_speed: f32,
}

impl Default for ShipMovementConfig {
    fn default() -> Self {
        Self {
            thrust_force: 500.0,
            reverse_force: 250.0,
            turn_rate: 3.0,
            max_speed: 300.0,
        }
    }
}

/// System that handles ship movement based on player input.
/// Queries ships with `Player` marker and applies physics forces.
/// 
/// Tasks implemented:
/// - 2.3.1: ShipMovementSystem (queries Ship + RigidBody)
/// - 2.3.2: Thrust (W key)
/// - 2.3.3: Reverse (S key)
/// - 2.3.4: Turn (A/D keys)
/// - 2.3.5: Drag (handled by LinearDamping in spawn)
/// - 2.3.6: Anchor (Shift key)
/// - 2.3.7: Speed debuff based on sail damage
/// - 2.3.8: Turn debuff based on rudder damage
pub fn ship_movement_system(
    _time: Res<Time>,
    action_query: Query<&ActionState<PlayerAction>>,
    mut ship_query: Query<
        (
            &Health,
            &Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &mut ExternalForce,
        ),
        (With<Ship>, With<Player>),
    >,
) {
    let config = ShipMovementConfig::default();
    
    // Get action state from whatever entity has it (camera currently)
    let Ok(action_state) = action_query.get_single() else {
        return;
    };
    
    for (health, transform, mut linear_vel, mut angular_vel, mut force) in &mut ship_query {
        // Calculate damage modifiers (task 2.3.7 and 2.3.8)
        let sail_modifier = health.sails_ratio();
        let rudder_modifier = health.rudder_ratio();
        
        // Effective max speed and turn rate
        let effective_max_speed = config.max_speed * sail_modifier;
        let effective_turn_rate = config.turn_rate * rudder_modifier;
        
        // Get forward direction from ship's rotation
        let forward = transform.rotation * Vec3::Y;
        let forward_2d = Vec2::new(forward.x, forward.y);
        
        // Reset force each frame
        force.clear();
        
        // === Anchor (Shift key) - Task 2.3.6 ===
        if action_state.pressed(&PlayerAction::Anchor) {
            // Stop linear motion, but allow rotation for "whip turns"
            linear_vel.0 = Vec2::ZERO;
            
            // Still allow turning while anchored
            if action_state.pressed(&PlayerAction::TurnLeft) {
                angular_vel.0 = effective_turn_rate;
            } else if action_state.pressed(&PlayerAction::TurnRight) {
                angular_vel.0 = -effective_turn_rate;
            } else {
                angular_vel.0 = 0.0;
            }
            
            continue; // Skip thrust logic when anchored
        }
        
        // === Thrust (W key) - Task 2.3.2 ===
        if action_state.pressed(&PlayerAction::Thrust) {
            let thrust = forward_2d * config.thrust_force * sail_modifier;
            force.apply_force(thrust);
        }
        
        // === Reverse (S key) - Task 2.3.3 ===
        if action_state.pressed(&PlayerAction::Reverse) {
            let reverse = -forward_2d * config.reverse_force * sail_modifier;
            force.apply_force(reverse);
        }
        
        // === Turn (A/D keys) - Task 2.3.4 ===
        if action_state.pressed(&PlayerAction::TurnLeft) {
            angular_vel.0 = effective_turn_rate;
        } else if action_state.pressed(&PlayerAction::TurnRight) {
            angular_vel.0 = -effective_turn_rate;
        } else {
            // No turn input - let angular damping handle slowdown
            // Angular damping is set in spawn_player_ship
        }
        
        // === Speed cap - Task 2.3.7 (partial) ===
        let current_speed = linear_vel.0.length();
        if current_speed > effective_max_speed {
            linear_vel.0 = linear_vel.0.normalize() * effective_max_speed;
        }
        
        // === Drag - Task 2.3.5 ===
        // Drag is already applied via LinearDamping component in spawn_player_ship
        // No additional logic needed here
    }
}
