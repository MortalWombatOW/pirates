//! Landmass-based movement systems for velocity steering navigation.
//!
//! These systems replace the old waypoint-following navigation with
//! velocity-based steering from bevy_landmass. Ships move forward in their
//! facing direction and rotate toward the desired direction at a rate
//! determined by ship size.

use bevy::prelude::*;
use bevy_landmass::prelude::*;

use crate::components::{Player, Ship, Destination};
use crate::components::ship::ShipType;
use crate::components::companion::CompanionRole;
use crate::plugins::worldmap::HighSeasAI;
use crate::resources::{Wind, MapData};
use crate::utils::pathfinding::{world_to_tile, tile_to_world};

/// Extracts the facing direction (forward vector) from a 2D rotation.
/// Ships face "up" in local space, so we extract the Y axis of the rotation.
fn facing_direction(rotation: Quat) -> Vec2 {
    let forward_3d = rotation * Vec3::Y;
    Vec2::new(forward_3d.x, forward_3d.y).normalize_or_zero()
}

/// Calculates the signed angle between two 2D vectors.
fn signed_angle(from: Vec2, to: Vec2) -> f32 {
    let cross = from.x * to.y - from.y * to.x;
    let dot = from.dot(to);
    cross.atan2(dot)
}

/// System that moves ships using landmass velocity steering.
///
/// Ships rotate toward the desired velocity direction at a rate limited by
/// their ship type, then move forward in their facing direction.
pub fn landmass_player_movement_system(
    mut query: Query<
        (&mut Transform, &AgentDesiredVelocity2d, Option<&Destination>, &ShipType),
        (With<Player>, With<Ship>),
    >,
    companion_query: Query<&CompanionRole>,
    meta_profile: Option<Res<crate::resources::MetaProfile>>,
    wind: Res<Wind>,
    time: Res<Time>,
) {
    // Check if player has a Navigator companion (provides +25% speed bonus)
    let has_navigator = companion_query.iter().any(|role| *role == CompanionRole::Navigator);
    let navigator_bonus = if has_navigator { 1.25 } else { 1.0 };

    // Apply Navigation stat bonus
    let stat_bonus = meta_profile
        .as_ref()
        .map(|p| p.stats.sailing_speed_multiplier())
        .unwrap_or(1.0);

    for (mut transform, desired_velocity, destination, ship_type) in &mut query {
        // Skip if no destination set
        if destination.is_none() {
            continue;
        }

        let velocity = desired_velocity.velocity();

        // Skip if velocity is essentially zero (arrived at destination)
        if velocity.length_squared() < 0.01 {
            continue;
        }

        let desired_direction = velocity.normalize_or_zero();
        let current_facing = facing_direction(transform.rotation);

        // Calculate how much we need to turn
        let angle_diff = signed_angle(current_facing, desired_direction);

        // Limit turn rate based on ship type
        let max_turn = ship_type.turn_rate() * time.delta_secs();
        let actual_turn = angle_diff.clamp(-max_turn, max_turn);

        // Apply rotation
        transform.rotation *= Quat::from_rotation_z(actual_turn);

        // Get the new facing direction after rotation
        let new_facing = facing_direction(transform.rotation);

        // Calculate speed - reduce when facing differs from desired direction
        // Quadratic falloff: facing 90° off = 0% speed, 45° off ≈ 50%
        let alignment = new_facing.dot(desired_direction).max(0.0);
        let turn_penalty = alignment.powi(2);
        let base_speed = ship_type.base_speed() * navigator_bonus * stat_bonus * turn_penalty;

        // Wind effect (±50% based on alignment with facing direction)
        let wind_alignment = new_facing.dot(wind.direction_vec());
        let wind_effect = wind_alignment * wind.strength * 0.5;
        let speed = base_speed * (1.0 + wind_effect);

        // Move forward in facing direction
        let movement = new_facing * speed * time.delta_secs();
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;
    }
}

/// System that moves AI ships using landmass velocity steering.
///
/// AI ships also use ship-type-based turning, moving forward in their
/// facing direction with rotation limited by ship type.
pub fn landmass_ai_movement_system(
    mut query: Query<
        (&mut Transform, &AgentDesiredVelocity2d, Option<&Destination>, &ShipType),
        (With<HighSeasAI>, With<Ship>),
    >,
    time: Res<Time>,
) {
    for (mut transform, desired_velocity, destination, ship_type) in &mut query {
        // Skip if no destination set
        if destination.is_none() {
            continue;
        }

        let velocity = desired_velocity.velocity();

        // Skip if velocity is essentially zero (arrived at destination)
        if velocity.length_squared() < 0.01 {
            continue;
        }

        let desired_direction = velocity.normalize_or_zero();
        let current_facing = facing_direction(transform.rotation);

        // Calculate how much we need to turn
        let angle_diff = signed_angle(current_facing, desired_direction);

        // Limit turn rate based on ship type
        let max_turn = ship_type.turn_rate() * time.delta_secs();
        let actual_turn = angle_diff.clamp(-max_turn, max_turn);

        // Apply rotation
        transform.rotation *= Quat::from_rotation_z(actual_turn);

        // Get the new facing direction after rotation
        let new_facing = facing_direction(transform.rotation);

        // Speed reduction when facing differs from desired (same as player)
        let alignment = new_facing.dot(desired_direction).max(0.0);
        let turn_penalty = alignment.powi(2);
        // AI ships move at reduced speed (set in agent settings)
        let speed = ship_type.base_speed() * 0.5 * turn_penalty;

        // Move forward in facing direction
        let movement = new_facing * speed * time.delta_secs();
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;
    }
}

/// System that detects arrival at destination and cleans up navigation components.
///
/// Uses proximity to destination rather than waypoint completion.
pub fn arrival_detection_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Destination)>,
) {
    const ARRIVAL_THRESHOLD: f32 = 32.0;

    for (entity, transform, destination) in &query {
        let distance = transform.translation.truncate().distance(destination.target);

        if distance < ARRIVAL_THRESHOLD {
            // Arrived at destination - remove navigation components
            commands.entity(entity).remove::<Destination>();
            // Also remove AgentTarget2d to stop landmass from steering
            commands.entity(entity).remove::<AgentTarget2d>();
        }
    }
}

/// System that syncs Destination component to AgentTarget2d.
///
/// When Destination changes, updates the landmass target.
pub fn sync_destination_to_agent_target(
    mut commands: Commands,
    query: Query<(Entity, &Destination), Changed<Destination>>,
) {
    for (entity, destination) in &query {
        commands.entity(entity).insert(AgentTarget2d::Point(destination.target));
    }
}

/// Emergency coastline avoidance system.
///
/// Queries nearby tiles and applies avoidance push if too close to land.
/// Runs AFTER movement systems to correct any remaining collision risk.
pub fn coastline_avoidance_system(
    mut query: Query<(&mut Transform, &ShipType), With<Ship>>,
    map_data: Res<MapData>,
) {
    /// Distance in world units at which emergency avoidance activates (~0.75 tiles).
    const CRITICAL_DISTANCE: f32 = 48.0;
    /// Push strength in units per frame at maximum urgency.
    const PUSH_STRENGTH: f32 = 2.0;

    for (mut transform, ship_type) in &mut query {
        let pos = transform.translation.truncate();
        let tile = world_to_tile(pos, map_data.width, map_data.height);

        // Sample 8 surrounding tiles + center
        let mut avoidance_vec = Vec2::ZERO;
        let mut land_count = 0;

        for dx in -1..=1_i32 {
            for dy in -1..=1_i32 {
                let check_x = tile.x + dx;
                let check_y = tile.y + dy;

                // Map edge treated as land
                if check_x < 0 || check_y < 0 {
                    avoidance_vec += Vec2::new(-dx as f32, -dy as f32);
                    land_count += 1;
                    continue;
                }

                if !map_data.is_navigable(check_x as u32, check_y as u32) {
                    // This tile is land — push away from it
                    let land_center = tile_to_world(
                        bevy::math::IVec2::new(check_x, check_y),
                        map_data.width,
                        map_data.height,
                    );
                    let to_ship = pos - land_center;
                    let dist = to_ship.length();

                    if dist < CRITICAL_DISTANCE {
                        // Urgency increases as ship gets closer
                        let urgency = 1.0 - (dist / CRITICAL_DISTANCE);
                        avoidance_vec += to_ship.normalize_or_zero() * urgency;
                        land_count += 1;
                    }
                }
            }
        }

        if land_count > 0 {
            // Apply push scaled by ship agility (nimble ships escape faster)
            let push = avoidance_vec.normalize_or_zero() * PUSH_STRENGTH * ship_type.turn_rate();
            transform.translation.x += push.x;
            transform.translation.y += push.y;
        }
    }
}
