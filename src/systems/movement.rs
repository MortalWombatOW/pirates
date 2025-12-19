use bevy::prelude::*;
use avian2d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::components::{Ship, Player, Health};
use crate::plugins::input::PlayerAction;

/// Configuration for ship movement parameters.
pub struct ShipMovementConfig {
    /// Acceleration when thrusting (units/second^2)
    pub thrust_accel: f32,
    /// Acceleration when reversing (units/second^2)
    pub reverse_accel: f32,
    /// Turn rate (radians/second)
    pub turn_rate: f32,
    /// Maximum linear speed (units/second)
    pub max_speed: f32,
    /// Drag coefficient (0-1, how much speed is retained per second)
    pub drag: f32,
}

impl Default for ShipMovementConfig {
    fn default() -> Self {
        Self {
            thrust_accel: 200.0,
            reverse_accel: 100.0,
            turn_rate: 3.0,
            max_speed: 300.0,
            drag: 0.98, // Retain 98% of speed per second
        }
    }
}

/// System that handles ship movement based on player input.
/// Uses direct velocity modification for responsive controls.
pub fn ship_movement_system(
    time: Res<Time>,
    mut debug_timer: Local<f32>,
    action_query: Query<&ActionState<PlayerAction>>,
    mut ship_query: Query<
        (
            Entity,
            &Health,
            &Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
        ),
        (With<Ship>, With<Player>),
    >,
) {
    let config = ShipMovementConfig::default();
    let dt = time.delta_secs();
    
    *debug_timer += dt;
    let should_log = *debug_timer > 1.0;
    if should_log {
        *debug_timer = 0.0;
    }
    
    // Get action state
    let action_state = match action_query.get_single() {
        Ok(state) => state,
        Err(_) => {
            if should_log {
                println!("[MOVE] ERROR: No ActionState found!");
            }
            return;
        }
    };
    
    for (entity, health, transform, mut linear_vel, mut angular_vel) in &mut ship_query {
        let sail_modifier = health.sails_ratio();
        let rudder_modifier = health.rudder_ratio();
        let effective_max_speed = config.max_speed * sail_modifier;
        let effective_turn_rate = config.turn_rate * rudder_modifier;
        
        // Calculate forward direction
        let forward = transform.rotation * Vec3::Y;
        let forward_2d = Vec2::new(forward.x, forward.y);
        
        // Check inputs
        let thrust_pressed = action_state.pressed(&PlayerAction::Thrust);
        let reverse_pressed = action_state.pressed(&PlayerAction::Reverse);
        let anchor_pressed = action_state.pressed(&PlayerAction::Anchor);
        let turn_left = action_state.pressed(&PlayerAction::TurnLeft);
        let turn_right = action_state.pressed(&PlayerAction::TurnRight);
        
        if should_log {
            println!("[MOVE] Entity {:?} | Pos: ({:.1}, {:.1}) | Vel: ({:.1}, {:.1}) | Speed: {:.1}", 
                entity, 
                transform.translation.x, transform.translation.y,
                linear_vel.0.x, linear_vel.0.y,
                linear_vel.0.length());
            println!("[MOVE] Input: Thrust:{} Rev:{} Anchor:{} TurnL:{} TurnR:{}", 
                thrust_pressed, reverse_pressed, anchor_pressed, turn_left, turn_right);
        }
        
        // === Anchor ===
        if anchor_pressed {
            linear_vel.0 = Vec2::ZERO;
            if turn_left {
                angular_vel.0 = effective_turn_rate;
            } else if turn_right {
                angular_vel.0 = -effective_turn_rate;
            } else {
                angular_vel.0 = 0.0;
            }
            continue;
        }
        
        // === Apply drag (before acceleration so it doesn't fight thrust) ===
        let drag_factor = config.drag.powf(dt);
        linear_vel.0 *= drag_factor;
        
        // === Thrust ===
        if thrust_pressed {
            let accel = forward_2d * config.thrust_accel * sail_modifier * dt;
            linear_vel.0 += accel;
            if should_log {
                println!("[MOVE] Thrust accel: {:?}", accel);
            }
        }
        
        // === Reverse ===
        if reverse_pressed {
            let accel = -forward_2d * config.reverse_accel * sail_modifier * dt;
            linear_vel.0 += accel;
            if should_log {
                println!("[MOVE] Reverse accel: {:?}", accel);
            }
        }
        
        // === Speed cap ===
        let speed = linear_vel.0.length();
        if speed > effective_max_speed {
            linear_vel.0 = linear_vel.0.normalize() * effective_max_speed;
        }
        
        // === Turn ===
        if turn_left {
            angular_vel.0 = effective_turn_rate;
        } else if turn_right {
            angular_vel.0 = -effective_turn_rate;
        } else {
            // Apply angular drag
            angular_vel.0 *= 0.9_f32.powf(dt * 60.0);
        }
    }
}


