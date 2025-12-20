use bevy::prelude::*;
use avian2d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::components::{Ship, Player, Health};
use crate::plugins::input::PlayerAction;
use crate::resources::Wind;

/// Ship physics configuration.
/// 
/// Forces are modeled as:
/// - **Thrust**: Continuous force applied in ship's forward direction (Newtons)
/// - **Water Drag**: Handled by Avian's LinearDamping component (coefficient)
/// - **Angular Drag**: Handled by Avian's AngularDamping component (coefficient)
/// - **Wind**: Continuous force applied in wind direction, scaled by strength
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
    /// Drag coefficient for forward/backward movement (longitudinal)
    pub longitudinal_drag: f32,
    /// Drag coefficient for sideways movement (lateral) - should be much higher
    pub lateral_drag: f32,
}

impl Default for ShipPhysicsConfig {
    fn default() -> Self {
        Self {
            max_thrust: 150000.0,     // Halved from 300,000 N
            max_reverse_thrust: 50000.0, // Halved from 100,000 N
            turn_torque: 175000.0,    // Halved from 350,000 Nm
            max_angular_speed: 1.75,  // Halved from 3.5
            longitudinal_drag: 0.6,
            lateral_drag: 3.0,
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
    pub fire_port: bool,
    pub fire_starboard: bool,
    pub mouse_world_pos: Vec2,
}

/// System that captures input state for use by physics systems.
/// Runs in Update to catch all input events.
pub fn buffer_ship_input(
    action_query: Query<&ActionState<PlayerAction>>,
    mut input_buffer: ResMut<ShipInputBuffer>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    if let Ok(action_state) = action_query.get_single() {
        input_buffer.thrust = action_state.pressed(&PlayerAction::Thrust);
        input_buffer.reverse = action_state.pressed(&PlayerAction::Reverse);
        input_buffer.turn_left = action_state.pressed(&PlayerAction::TurnLeft);
        input_buffer.turn_right = action_state.pressed(&PlayerAction::TurnRight);
        input_buffer.anchor = action_state.pressed(&PlayerAction::Anchor);
        
        // Sticky firing: capture the intent, don't clear it until consumed by the physics system
        if action_state.just_pressed(&PlayerAction::FirePort) {
            input_buffer.fire_port = true;
        }
        if action_state.just_pressed(&PlayerAction::FireStarboard) {
            input_buffer.fire_starboard = true;
        }
    }

    // Capture mouse world position
    let window = window_query.single();
    let (camera, camera_transform) = camera_query.single();
    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    {
        input_buffer.mouse_world_pos = world_position;
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
    wind: Res<Wind>,
    mut ship_query: Query<
        (
            &Health,
            &Transform,
            &mut ExternalForce,
            &mut ExternalTorque,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &Mass,
        ),
        (With<Ship>, With<Player>),
    >,
) {
    for (health, transform, mut force, mut torque, mut lin_vel, mut ang_vel, mass) in &mut ship_query {
        let ship_mass = mass.0;
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
            // info!("Movement System: Thrust input detected!");
            thrust_magnitude += config.max_thrust * sail_effectiveness;
        }
        if input_buffer.reverse {
            // info!("Movement System: Reverse input detected!");
            thrust_magnitude -= config.max_reverse_thrust * sail_effectiveness;
        }
        
        if thrust_magnitude != 0.0 {
             info!("Movement System: Applying thrust magnitude: {:.1}", thrust_magnitude);
        }
        
        // Apply thrust force in forward direction
        let mut total_force = forward_2d * thrust_magnitude;
        
        // === Apply Anisotropic Drag (Keel Effect) ===
        // 1. Get current velocity
        let world_vel = lin_vel.0;
        
        // 2. Project velocity onto local axes
        // In Bevy 2D (X-right, Y-up), if forward is (0, 1), right is (1, 0)
        // Vec2::perp() is (-y, x), which is 90deg CCW (Left)
        // So Right is -forward.perp()
        let right_2d = -forward_2d.perp(); 
        
        let v_forward = world_vel.dot(forward_2d);
        let v_lateral = world_vel.dot(right_2d);
        
        // 3. Calculate drag forces
        // F_drag = -velocity * coefficient * mass
        // Using mass ensures acceleration remains consistent with thrust
        let drag_longitudinal = -forward_2d * v_forward * config.longitudinal_drag * ship_mass;
        let drag_lateral = -right_2d * v_lateral * config.lateral_drag * ship_mass;
        
        total_force += drag_longitudinal;
        total_force += drag_lateral;
        
        // === Apply Wind Force ===
        // Wind pushes the ship in its direction, scaled by sail effectiveness
        let wind_force_magnitude = 20000.0; // Base wind force at 100% strength
        let wind_force = wind.velocity() * wind_force_magnitude * sail_effectiveness;
        total_force += wind_force;
        
        force.set_force(total_force);
        
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



