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
use crate::resources::Wind;

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
        let pos = transform.translation.truncate();
        let velocity = desired_velocity.velocity();
        
        // Skip if no destination set
        if destination.is_none() {
            continue;
        }
        
        let dest = destination.unwrap();

        // If velocity is zero but we have a destination, use direct movement as fallback
        // This allows testing coastline avoidance even when navmesh pathing fails
        let (desired_direction, using_fallback) = if velocity.length_squared() < 0.01 {
            let direct = (dest.target - pos).normalize_or_zero();
            if direct.length_squared() < 0.01 {
                continue; // Already at destination
            }
            info!("[MOVE] Using direct fallback movement toward {:?}", dest.target);
            (direct, true)
        } else {
            (velocity.normalize_or_zero(), false)
        };
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

/// Coastline avoidance system using polygon-based velocity clamping.
///
/// Instead of pushing ships away, this system neutralizes the component of
/// movement velocity that would take the ship closer to the coastline.
/// Uses actual coastline polygon geometry for accurate normal calculation.
pub fn coastline_avoidance_system(
    mut query: Query<(&mut Transform, Option<&crate::components::Player>), With<Ship>>,
    coastline_data: Res<crate::plugins::worldmap::CoastlineData>,
) {
    const CRITICAL_DISTANCE: f32 = 64.0;

    for (mut transform, player) in &mut query {
        let pos = transform.translation.truncate();
        let is_player = player.is_some();

        // Find the nearest coastline edge segment
        if let Some((nearest_point, edge_normal)) = find_nearest_coastline_edge(pos, &coastline_data.polygons) {
            let dist_to_coast = pos.distance(nearest_point);

            // Only apply clamping when close to coast
            if dist_to_coast >= CRITICAL_DISTANCE {
                continue;
            }

            // The edge_normal points outward (away from land, into water).
            // If the ship's position relative to the coast has a negative dot with
            // edge_normal, it's on the land side — push it out.
            let to_ship = pos - nearest_point;
            let side = to_ship.dot(edge_normal);

            if side < 0.0 {
                // Ship is on wrong side or very close - push along normal
                let push = edge_normal * (-side + 1.0);
                if is_player {
                    info!("[COAST] PUSHING player by {:?}", push);
                }
                transform.translation.x += push.x;
                transform.translation.y += push.y;
            } else if is_player {
                 // No push needed
            }
        }
    }
}

/// Finds the nearest point on any coastline edge to the given position,
/// and returns the outward-facing normal of that edge.
///
/// Returns `None` if no coastline polygons exist.
fn find_nearest_coastline_edge(
    pos: Vec2,
    polygons: &[crate::utils::geometry::CoastlinePolygon],
) -> Option<(Vec2, Vec2)> {
    let mut best_dist_sq = f32::MAX;
    let mut best_point = Vec2::ZERO;
    let mut best_normal = Vec2::ZERO;

    for polygon in polygons {
        let points = &polygon.points;
        if points.len() < 3 {
            continue;
        }

        // No filtering by point count - check all polygons
        // Border polygons can be distinguished by their spatial extent instead

        for i in 0..points.len() {
            let a = points[i];
            let b = points[(i + 1) % points.len()];

            // Find closest point on segment [a, b] to pos
            let (closest, normal) = closest_point_on_segment_with_normal(pos, a, b);
            let dist_sq = pos.distance_squared(closest);

            if dist_sq < best_dist_sq {
                best_dist_sq = dist_sq;
                best_point = closest;
                best_normal = normal;
            }
        }
    }

    if best_dist_sq < f32::MAX {
        Some((best_point, best_normal))
    } else {
        None
    }
}

/// Returns the closest point on segment [a, b] to point p, plus the outward normal.
///
/// For CCW polygons where land is on the left, the outward normal (into water)
/// is perpendicular to the edge, pointing right (CW rotation of edge direction).
fn closest_point_on_segment_with_normal(p: Vec2, a: Vec2, b: Vec2) -> (Vec2, Vec2) {
    let ab = b - a;
    let ab_len_sq = ab.length_squared();

    if ab_len_sq < 1e-10 {
        // Degenerate segment - use perpendicular to arbitrary direction
        return (a, Vec2::Y);
    }

    // Project p onto line ab, clamped to segment
    let t = ((p - a).dot(ab) / ab_len_sq).clamp(0.0, 1.0);
    let closest = a + ab * t;

    // Outward normal: perpendicular to edge, pointing right (into water)
    // For CCW winding with land on left, RIGHT is the water side
    // 90° CW rotation of (x, y) is (y, -x)
    let edge_dir = ab.normalize();
    let normal = Vec2::new(edge_dir.y, -edge_dir.x);

    (closest, normal)
}

