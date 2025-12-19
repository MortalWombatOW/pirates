use bevy::prelude::*;
use avian2d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::components::{Ship, Player, Health};
use crate::plugins::input::PlayerAction;

/// Ship physics configuration.
/// 
/// Forces are modeled as:
/// - **Thrust**: Continuous force applied in ship's forward direction (Newtons)
/// - **Water Drag**: Handled by Avian's LinearDamping component (coefficient)
/// - **Angular Drag**: Handled by Avian's AngularDamping component (coefficient)
/// - **Wind**: Will be added in future phase per README §4.4
#[derive(Resource)]
pub struct ShipPhysicsConfig {
    /// Maximum thrust force when sails are at 100% (Newtons)
    pub max_thrust: f32,
    /// Maximum reverse thrust (typically less than forward)
    pub max_reverse_thrust: f32,
    /// Torque applied when turning (Newton-meters)
    pub turn_torque: f32,
    /// Maximum angular speed (radians/second)
    pub max_angular_speed: f32,
}

impl Default for ShipPhysicsConfig {
    fn default() -> Self {
        Self {
            max_thrust: 300000.0,     // 300,000 N (Balanced for 1000kg mass + 0.8 damping)
            max_reverse_thrust: 100000.0,
            turn_torque: 250000.0,    // 250,000 Nm (Balanced for 20000 inertia + 2.5 damping)
            max_angular_speed: 3.5,   // Slightly increased turning cap
        }
    }
}

/// Buffered input state for physics systems running in FixedUpdate.
/// This captures input state each frame so FixedUpdate systems can access it reliably.
#[derive(Resource, Default)]
pub struct ShipInputBuffer {
    pub thrust: bool,
    pub reverse: bool,
    pub turn_left: bool,
    pub turn_right: bool,
    pub anchor: bool,
}

/// System that captures input state for use by physics systems.
/// Runs in Update to catch all input events.
pub fn buffer_ship_input(
    action_query: Query<&ActionState<PlayerAction>>,
    mut input_buffer: ResMut<ShipInputBuffer>,
) {
    if let Ok(action_state) = action_query.get_single() {
        input_buffer.thrust = action_state.pressed(&PlayerAction::Thrust);
        input_buffer.reverse = action_state.pressed(&PlayerAction::Reverse);
        input_buffer.turn_left = action_state.pressed(&PlayerAction::TurnLeft);
        input_buffer.turn_right = action_state.pressed(&PlayerAction::TurnRight);
        input_buffer.anchor = action_state.pressed(&PlayerAction::Anchor);
    }
}

/// Physics-based ship movement system.
/// 
/// Runs in FixedUpdate for deterministic physics.
/// Applies forces based on buffered input:
/// 
/// **Force Model:**
/// ```text
/// F_total = F_thrust + F_drag + F_wind (future)
/// 
/// F_thrust = thrust_force * forward_direction * sail_health_ratio
/// F_drag   = -linear_damping_coefficient * velocity (handled by Avian)
/// ```
/// 
/// **Torque Model:**
/// ```text
/// τ_total = τ_turn + τ_angular_drag (handled by Avian)
/// 
/// τ_turn = turn_torque * rudder_health_ratio
/// ```
pub fn ship_physics_system(
    input_buffer: Res<ShipInputBuffer>,
    config: Res<ShipPhysicsConfig>,
    mut ship_query: Query<
        (
            &Health,
            &Transform,
            &mut ExternalForce,
            &mut ExternalTorque,
            &mut LinearVelocity,
            &mut AngularVelocity,
        ),
        (With<Ship>, With<Player>),
    >,
) {
    for (health, transform, mut force, mut torque, mut lin_vel, mut ang_vel) in &mut ship_query {
        // Calculate effectiveness based on component damage
        let sail_effectiveness = health.sails_ratio();
        let rudder_effectiveness = health.rudder_ratio();
        
        // Get ship's forward direction (Y-up in local space)
        let forward = transform.rotation * Vec3::Y;
        let forward_2d = Vec2::new(forward.x, forward.y);
        
        // === Anchor: Stop all motion ===
        if input_buffer.anchor {
            lin_vel.0 = Vec2::ZERO;
            force.clear();
            
            // Still allow turning while anchored
            if input_buffer.turn_left {
                ang_vel.0 = config.max_angular_speed * rudder_effectiveness;
            } else if input_buffer.turn_right {
                ang_vel.0 = -config.max_angular_speed * rudder_effectiveness;
            } else {
                ang_vel.0 = 0.0;
            }
            torque.clear();
            continue;
        }
        
        // === Calculate net thrust force ===
        let mut thrust_magnitude = 0.0;
        
        if input_buffer.thrust {
            thrust_magnitude += config.max_thrust * sail_effectiveness;
        }
        if input_buffer.reverse {
            thrust_magnitude -= config.max_reverse_thrust * sail_effectiveness;
        }
        
        // Apply thrust force in forward direction
        let thrust_force = forward_2d * thrust_magnitude;
        *force = ExternalForce::new(thrust_force);
        
        // === Calculate turning torque ===
        let mut turn_direction = 0.0;
        
        if input_buffer.turn_left {
            turn_direction = 1.0;
        } else if input_buffer.turn_right {
            turn_direction = -1.0;
        }
        
        let turn_torque_value = config.turn_torque * turn_direction * rudder_effectiveness;
        *torque = ExternalTorque::new(turn_torque_value);
        
        // === Angular speed limit ===
        let max_ang = config.max_angular_speed * rudder_effectiveness;
        if ang_vel.0.abs() > max_ang {
            ang_vel.0 = ang_vel.0.signum() * max_ang;
        }
    }
}

/// Debug system to log ship physics state periodically.
pub fn debug_ship_physics(
    time: Res<Time>,
    mut timer: Local<f32>,
    ship_query: Query<
        (&Transform, &LinearVelocity, &AngularVelocity, &ExternalForce),
        (With<Ship>, With<Player>),
    >,
) {
    *timer += time.delta_secs();
    if *timer < 1.0 {
        return;
    }
    *timer = 0.0;
    
    for (transform, lin_vel, ang_vel, force) in &ship_query {
        let speed = lin_vel.0.length();
        println!(
            "[PHYSICS] Pos: ({:.1}, {:.1}) | Speed: {:.1} | AngVel: {:.2} | Force: ({:.1}, {:.1})",
            transform.translation.x,
            transform.translation.y,
            speed,
            ang_vel.0,
            force.force().x,
            force.force().y,
        );
    }
}



