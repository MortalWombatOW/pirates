//! AI systems for controlling enemy ships in combat.
//! 
//! The AI uses a broadside circling strategy:
//! - Circle around the player to maintain optimal firing range
//! - Keep the player perpendicular (at broadside angle) for cannon fire
//! - Flee when health is critical

use bevy::prelude::*;
use avian2d::prelude::*;

use crate::components::{Ship, Player, Health, AI, Projectile, TargetComponent};

/// AI behavior state.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AIState {
    /// Circling to maintain broadside angle
    #[default]
    Circling,
    /// Fleeing due to low health
    Fleeing,
}

/// Per-enemy cannon cooldown tracking.
#[derive(Component, Debug)]
pub struct AICannonCooldown {
    pub timer: Timer,
}

impl Default for AICannonCooldown {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Once), // Slightly slower than player
        }
    }
}

/// AI physics configuration.
#[derive(Resource)]
pub struct AIPhysicsConfig {
    /// Thrust force (Newtons) - same as player
    pub thrust: f32,
    /// Torque (Newton-meters) - same as player
    pub torque: f32,
    /// Optimal range to maintain from player
    pub optimal_range: f32,
    /// Range at which AI can fire
    pub firing_range: f32,
    /// Firing arc (radians from perpendicular, ±this value)
    pub firing_arc: f32,
    /// Health percentage threshold to trigger flee
    pub flee_threshold: f32,
    /// Forward drag coefficient
    pub drag_forward: f32,
    /// Lateral drag coefficient (keel effect)
    pub drag_lateral: f32,
}

impl Default for AIPhysicsConfig {
    fn default() -> Self {
        Self {
            thrust: 150000.0,        // Same as player
            torque: 8000000.0,       // Same as player
            optimal_range: 150.0,    // Ideal circling distance
            firing_range: 250.0,     // Max range to fire
            firing_arc: 0.5,         // ~30 degrees from broadside
            flee_threshold: 0.2,     // 20% HP
            drag_forward: 0.5,
            drag_lateral: 5.0,
        }
    }
}

/// Main AI behavior system that controls enemy ship movement.
/// Runs in FixedUpdate for physics consistency.
pub fn combat_ai_system(
    mut commands: Commands,
    config: Res<AIPhysicsConfig>,
    player_query: Query<&Transform, (With<Player>, With<Ship>, Without<AI>)>,
    mut ai_query: Query<
        (
            Entity,
            &Transform,
            &Health,
            &LinearVelocity,
            &AngularVelocity,
            &Mass,
            &mut ExternalForce,
            &mut ExternalTorque,
            &mut AIState,
        ),
        (With<Ship>, With<AI>),
    >,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return; // No player to chase
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform, health, velocity, ang_velocity, mass, mut force, mut torque, mut ai_state) in &mut ai_query {
        // Check for surrender condition
        if health.hull < 20.0 {
            // Surrender!
            // Only do this once
            if commands.get_entity(entity).is_some() {
                 // We need to check if we already surrendered to avoid spamming components/logs
                 // But since we can't easily query "Without<Surrendered>" in this specific loop structure without changing signature,
                 // we rely on the fact that we break out logic. 
                 // Actually, best to add "Without<Surrendered>" to the query filter if possible, 
                 // OR check here. Since we can't see components on entity easily without another query...
                 // Let's just assume we re-apply (idempotent-ish) or check if we can add a cooldown?
                 // No, let's just add the component. Bevy handles duplicate component insertion gracefully (replaces).
                 // Ideally we'd filter the query, but let's just do it here.
                 
                 commands.entity(entity)
                    .insert(crate::components::Surrendered)
                    .insert(Name::new("Surrendered Ship"));
            }
            continue; // Stop AI logic
        }

        let ai_pos = transform.translation.truncate();
        let to_player = player_pos - ai_pos;
        let distance = to_player.length();

        // Get ship's forward direction (Y+ in local space after flip_y)
        let forward = (transform.rotation * Vec3::Y).truncate();
        let right = (transform.rotation * Vec3::X).truncate();

        // Calculate desired behavior based on state
        let (desired_direction, should_thrust) = match *ai_state {
            AIState::Circling => {
                // Broadside circling: maintain perpendicular angle to player
                // Dynamically choose which side to present based on player position
                
                let to_player_normalized = if distance > 0.01 { 
                    to_player / distance 
                } else { 
                    Vec2::Y 
                };
                
                // Determine which side the player is on relative to our heading
                // Positive = starboard (right), Negative = port (left)
                let player_side = right.dot(to_player_normalized);
                
                // Circle direction: if player is to starboard, circle counter-clockwise
                // to keep them on starboard; if port, circle clockwise to keep them on port
                let circle_direction = player_side.signum();
                
                // The tangent direction (perpendicular to line-of-sight)
                // Direction depends on which side we want the player
                let tangent = Vec2::new(
                    -to_player_normalized.y * circle_direction,
                    to_player_normalized.x * circle_direction,
                );
                
                // Blend between closing in and circling based on range
                let range_factor = (distance / config.optimal_range).clamp(0.5, 2.0);
                
                let desired = if distance > config.optimal_range * 1.2 {
                    // Too far: move toward player while circling
                    (to_player_normalized * 0.6 + tangent * 0.4).normalize_or_zero()
                } else if distance < config.optimal_range * 0.8 {
                    // Too close: move away while circling
                    (-to_player_normalized * 0.6 + tangent * 0.4).normalize_or_zero()
                } else {
                    // Good range: pure circling
                    tangent
                };
                
                (desired, range_factor > 0.6)
            }
            AIState::Fleeing => {
                // Run away from player
                let away = if distance > 0.01 { -to_player / distance } else { -Vec2::Y };
                (away, true)
            }
        };

        // Calculate steering torque using a PD controller to prevent oscillation
        let desired_angle = desired_direction.y.atan2(desired_direction.x) - std::f32::consts::FRAC_PI_2;
        let current_angle = transform.rotation.to_euler(EulerRot::ZYX).0;
        
        // Normalize angle difference to [-PI, PI]
        let mut angle_diff = desired_angle - current_angle;
        while angle_diff > std::f32::consts::PI { angle_diff -= 2.0 * std::f32::consts::PI; }
        while angle_diff < -std::f32::consts::PI { angle_diff += 2.0 * std::f32::consts::PI; }

        // PD Controller for steering:
        // - Proportional term: torque proportional to angle error
        // - Derivative term: damping based on current angular velocity
        let kp = 1.5;  // Proportional gain - higher = snappier turning
        let kd = 0.25; // Derivative gain - lower = less damping, more responsive
        
        // Scale angle_diff to [-1, 1] range (full torque at PI radians error)
        let proportional = (angle_diff / std::f32::consts::PI).clamp(-1.0, 1.0);
        
        // Damping term reduces torque when already spinning in the right direction
        let derivative = -ang_velocity.0 * kd;
        
        // Combine P and D terms, clamp to [-1, 1], then scale by max torque
        let torque_factor = (proportional * kp + derivative).clamp(-1.0, 1.0);
        let torque_amount = torque_factor * config.torque;
        torque.set_torque(torque_amount);

        // Apply thrust when roughly facing correct direction
        // Lower threshold (0.3 ≈ 72°) allows thrusting while still turning
        let facing_threshold = 0.3;
        let facing_right = forward.dot(desired_direction) > facing_threshold;
        
        let thrust_force = if should_thrust && facing_right {
            forward * config.thrust
        } else {
            Vec2::ZERO
        };

        // Apply anisotropic drag (keel effect) - same as player
        let vel = velocity.0;
        let forward_speed = vel.dot(forward);
        let lateral_speed = vel.dot(right);
        
        let drag_force = -forward * forward_speed * config.drag_forward * mass.0
                        - right * lateral_speed * config.drag_lateral * mass.0;

        force.set_force(thrust_force + drag_force);
    }
}

/// AI firing system - fires cannons when player is in broadside arc.
pub fn ai_firing_system(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<AIPhysicsConfig>,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, (With<Player>, With<Ship>, Without<AI>)>,
    mut ai_query: Query<
        (
            Entity,
            &Transform,
            &LinearVelocity,
            &AIState,
            &mut AICannonCooldown,
        ),
        (With<Ship>, With<AI>),
    >,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform, velocity, ai_state, mut cooldown) in &mut ai_query {
        // Tick cooldown
        cooldown.timer.tick(time.delta());

        // Don't fire while fleeing
        if *ai_state == AIState::Fleeing {
            continue;
        }

        // Check if ready to fire
        if !cooldown.timer.finished() {
            continue;
        }

        let ai_pos = transform.translation.truncate();
        let to_player = player_pos - ai_pos;
        let distance = to_player.length();

        // Check range
        if distance > config.firing_range {
            continue;
        }

        // Check if player is in broadside arc
        let right = (transform.rotation * Vec3::X).truncate();
        let to_player_normalized = to_player / distance;
        
        // Dot product with right vector: 1.0 = perfect starboard, -1.0 = perfect port
        let broadside_dot = right.dot(to_player_normalized).abs();
        
        // We want the dot to be high (player is to our side)
        let in_arc = broadside_dot > (1.0 - config.firing_arc);

        if in_arc {
            // Fire! Determine which side (port or starboard)
            let side = if right.dot(to_player_normalized) > 0.0 { 1.0 } else { -1.0 };
            
            let spawn_direction = right * side;
            let spawn_pos_center = transform.translation + (Vec3::from((right * side * 40.0, 0.0))) + Vec3::Z * 5.0;
            let projectile_speed = 400.0;

            // Fire 3 cannonballs in a spread (same as player)
            for i in -1..=1 {
                let forward = (transform.rotation * Vec3::Y).truncate();
                let offset = Vec3::from((forward * (i as f32 * 15.0), 0.0));
                let spawn_pos = spawn_pos_center + offset;

                commands.spawn((
                    Sprite {
                        image: asset_server.load("sprites/projectile.png"),
                        custom_size: Some(Vec2::new(16.0, 16.0)),
                        ..default()
                    },
                    Transform::from_translation(spawn_pos),
                    RigidBody::Dynamic,
                    Collider::circle(8.0),
                    Sensor,
                    LinearVelocity(velocity.0 + spawn_direction * projectile_speed),
                    Projectile {
                        damage: 10.0,
                        target: TargetComponent::Hull,
                        source: entity,
                    },
                    crate::systems::combat::ProjectileTimer::default(),
                ));
            }

            // Reset cooldown
            cooldown.timer.reset();
            
            info!(
                "Enemy fired broadside to {}!",
                if side > 0.0 { "Starboard" } else { "Port" }
            );
        }
    }
}

/// System to spawn enemies when entering combat state.
/// Uses the EncounteredEnemy resource to spawn the correct faction.
pub fn spawn_combat_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut encountered_enemy: ResMut<crate::plugins::worldmap::EncounteredEnemy>,
) {
    use crate::components::FactionId;
    use crate::systems::ship::spawn_enemy_ship;
    
    // Get faction from encounter data, default to Pirates
    let faction = encountered_enemy.faction.take().unwrap_or(FactionId::Pirates);
    
    // Spawn one enemy ship to the north
    let enemy_id = spawn_enemy_ship(
        &mut commands,
        &asset_server,
        Vec2::new(0.0, 200.0),
        faction,
    );
    
    // Add AI-specific components
    commands.entity(enemy_id).insert((
        AIState::default(),
        AICannonCooldown::default(),
    ));
    
    info!("Combat enemy spawned with faction {:?}!", faction);
}
