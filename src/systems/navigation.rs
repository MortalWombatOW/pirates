use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{Player, Ship, Destination, NavigationPath};
use crate::components::companion::CompanionRole;
use crate::resources::{MapData, Wind};
use crate::plugins::core::{GameState, MainCamera};
use crate::utils::pathfinding::{find_path, tile_to_world, world_to_tile};

/// System that handles mouse clicks to set navigation destination.
pub fn click_to_navigate_system(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    player_query: Query<Entity, (With<Player>, With<Ship>)>,
    map_data: Res<MapData>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    
    let Ok(window) = window_query.get_single() else { return };
    let Ok((camera, camera_transform)) = camera_query.get_single() else { return };
    let Ok(player_entity) = player_query.get_single() else { return };
    
    // Get cursor position in world coordinates
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };
    
    // Convert to tile coordinates
    let tile_pos = world_to_tile(world_pos, map_data.width, map_data.height);
    
    // Check if destination is navigable
    if !map_data.is_navigable(tile_pos.x as u32, tile_pos.y as u32) {
        info!("Clicked on non-navigable tile at ({}, {})", tile_pos.x, tile_pos.y);
        return;
    }
    
    // Set destination on player
    let world_target = tile_to_world(tile_pos, map_data.width, map_data.height);
    commands.entity(player_entity).insert(Destination { target: world_target });
    
    info!("Navigation destination set to tile ({}, {}) = world ({:.0}, {:.0})", 
          tile_pos.x, tile_pos.y, world_target.x, world_target.y);
}

/// System that calculates paths when destination changes.
/// Uses NavMesh pathfinding when available, with fallback to grid-based Theta*.
/// Applies Catmull-Rom spline smoothing for natural, flowing paths.
pub fn pathfinding_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Destination), (With<Player>, Changed<Destination>)>,
    map_data: Res<MapData>,
    navmesh: Option<Res<crate::resources::NavMeshResource>>,
) {
    use crate::components::ship::ShipType;
    
    for (entity, transform, destination) in &query {
        let current_pos = transform.translation.truncate();
        let goal_pos = destination.target;
        
        // Try NavMesh pathfinding first (uses Small tier for player - Sloop equivalent)
        let navmesh_path = navmesh.as_ref().and_then(|nm| {
            if nm.is_ready() {
                nm.find_path_for_ship(current_pos, goal_pos, ShipType::Sloop)
            } else {
                None
            }
        });
        
        let waypoints = if let Some(path) = navmesh_path {
            info!("NavMesh path found with {} waypoints", path.len());
            // Apply smoothing if enough points
            if path.len() >= 3 {
                smooth_path_catmull_rom(&path, 8)
            } else {
                path
            }
        } else {
            // Fallback to grid-based Theta*
            let start_tile = world_to_tile(current_pos, map_data.width, map_data.height);
            let goal_tile = world_to_tile(goal_pos, map_data.width, map_data.height);
            
            if let Some(tile_path) = find_path(start_tile, goal_tile, &map_data) {
                // Convert tile path to world waypoints
                let control_points: Vec<Vec2> = tile_path
                    .into_iter()
                    .map(|t| tile_to_world(t, map_data.width, map_data.height))
                    .collect();
                
                let num_control_points = control_points.len();
                
                // Apply curve smoothing if we have enough points
                let smoothed = if control_points.len() >= 3 {
                    smooth_path_catmull_rom(&control_points, 8)
                } else {
                    control_points
                };
                
                info!("Grid path found with {} waypoints (smoothed from {} control points)", 
                      smoothed.len(), num_control_points);
                smoothed
            } else {
                warn!("No path found to ({:.0}, {:.0})", goal_pos.x, goal_pos.y);
                // Remove destination if no path
                commands.entity(entity).remove::<Destination>();
                continue;
            }
        };
        
        commands.entity(entity).insert(NavigationPath { waypoints });
    }
}

/// Smooths a path using Catmull-Rom spline interpolation.
/// 
/// Uses reflected phantom points at endpoints to avoid overshoot, and a reduced
/// number of samples for smoother visuals.
/// 
/// # Arguments
/// * `points` - Control points (must have at least 2 points)
/// * `samples_per_segment` - Number of interpolated points per segment
fn smooth_path_catmull_rom(points: &[Vec2], samples_per_segment: usize) -> Vec<Vec2> {
    if points.len() < 2 {
        return points.to_vec();
    }
    
    if points.len() == 2 {
        // Just interpolate linearly between 2 points
        let mut result = Vec::new();
        for i in 0..=samples_per_segment {
            let t = i as f32 / samples_per_segment as f32;
            result.push(points[0].lerp(points[1], t));
        }
        return result;
    }
    
    let mut result = Vec::with_capacity(points.len() * samples_per_segment);
    let n = points.len();
    
    for i in 0..n - 1 {
        // Get the 4 control points for this segment
        // Use REFLECTION for phantom points at endpoints to avoid overshoot
        let p0 = if i == 0 { 
            // Reflect p1 around p0: phantom = p0 - (p1 - p0) = 2*p0 - p1
            points[0] * 2.0 - points[1]
        } else { 
            points[i - 1] 
        };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 >= n { 
            // Reflect p(n-2) around p(n-1): phantom = 2*p(n-1) - p(n-2)
            points[n - 1] * 2.0 - points[n - 2]
        } else { 
            points[i + 2] 
        };
        
        // Sample points along this segment
        for j in 0..samples_per_segment {
            let t = j as f32 / samples_per_segment as f32;
            let point = catmull_rom_interpolate(p0, p1, p2, p3, t, 0.5);
            result.push(point);
        }
    }
    
    // Add the final point
    result.push(*points.last().unwrap());
    
    result
}

/// Catmull-Rom spline interpolation between p1 and p2.
/// p0 and p3 are used to calculate tangents.
/// 
/// # Arguments
/// * `tension` - Controls curve tightness (0.5 = standard, lower = tighter/less swervy)
fn catmull_rom_interpolate(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32, tension: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    
    // Catmull-Rom basis functions with configurable tension
    // Standard Catmull-Rom uses tension = 0.5
    let b0 = -tension * t3 + 2.0 * tension * t2 - tension * t;
    let b1 = (2.0 - tension) * t3 + (tension - 3.0) * t2 + 1.0;
    let b2 = (tension - 2.0) * t3 + (3.0 - 2.0 * tension) * t2 + tension * t;
    let b3 = tension * t3 - tension * t2;
    
    p0 * b0 + p1 * b1 + p2 * b2 + p3 * b3
}

/// System that moves the player along the navigation path.
/// Uses smooth rotation for natural ship turning.
/// Navigator companion provides +25% speed bonus.
/// Navigation stat provides additional speed scaling.
pub fn navigation_movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut NavigationPath), With<Player>>,
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

    for (entity, mut transform, mut path) in &mut query {
        let Some(next_waypoint) = path.next_waypoint() else {
            // Path complete - remove navigation components
            commands.entity(entity).remove::<NavigationPath>();
            commands.entity(entity).remove::<Destination>();
            continue;
        };
        
        let current_pos = transform.translation.truncate();
        let direction = next_waypoint - current_pos;
        let distance = direction.length();
        
        // Waypoint reached threshold
        if distance < 32.0 {
            path.pop_waypoint();
            continue;
        }
        
        let direction_normalized = direction.normalize();
        
        // Base speed with wind effect, Navigator bonus, and Navigation stat
        let base_speed = 300.0 * navigator_bonus * stat_bonus;
        let wind_alignment = direction_normalized.dot(wind.direction_vec());
        let wind_effect = wind_alignment * wind.strength * 0.5;
        let speed = base_speed * (1.0 + wind_effect);
        
        // Move toward waypoint
        let movement = direction_normalized * speed * time.delta_secs();
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;
        
        // Smooth rotation toward direction of movement
        let target_angle = direction_normalized.y.atan2(direction_normalized.x) - std::f32::consts::FRAC_PI_2;
        let target_rotation = Quat::from_rotation_z(target_angle);
        
        // Smoothly interpolate rotation (slerp)
        let rotation_speed = 3.0; // Radians per second
        transform.rotation = transform.rotation.slerp(target_rotation, rotation_speed * time.delta_secs());
    }
}

/// System that visualizes the navigation path using old-timey map style.
/// Draws a dotted line with an X at the destination.
///
/// Works with both waypoint-based (NavigationPath) and velocity-based (landmass) navigation.
/// If NavigationPath is present, draws through waypoints; otherwise draws direct line.
pub fn path_visualization_system(
    query: Query<(&Transform, Option<&NavigationPath>, &Destination), With<Player>>,
    mut gizmos: Gizmos,
) {
    for (transform, maybe_path, destination) in &query {
        // Parchment/sepia colors for old-timey map look
        let line_color = Color::srgba(0.6, 0.4, 0.2, 0.9); // Brown/sepia
        let x_color = Color::srgba(0.8, 0.2, 0.1, 1.0); // Red X

        let current_pos = transform.translation.truncate();

        // Build path for dotted line
        let full_path: Vec<Vec2> = if let Some(path) = maybe_path {
            // Legacy waypoint-based path
            if path.waypoints.is_empty() {
                vec![current_pos, destination.target]
            } else {
                let mut p = vec![current_pos];
                p.extend(&path.waypoints);
                p
            }
        } else {
            // Landmass velocity-based: draw direct line to destination
            vec![current_pos, destination.target]
        };

        // Draw dotted line along path
        let dot_spacing = 24.0;
        let dot_radius = 4.0;

        for window in full_path.windows(2) {
            let start = window[0];
            let end = window[1];
            let segment = end - start;
            let segment_len = segment.length();
            let segment_dir = segment.normalize_or_zero();

            // Draw dots along this segment
            let num_dots = (segment_len / dot_spacing).ceil() as i32;
            for i in 0..num_dots {
                let t = (i as f32 * dot_spacing) / segment_len;
                if t <= 1.0 {
                    let dot_pos = start + segment_dir * (i as f32 * dot_spacing);
                    gizmos.circle_2d(Isometry2d::from_translation(dot_pos), dot_radius, line_color);
                }
            }
        }

        // Draw X at destination
        let x_size = 20.0;
        let dest = destination.target;
        gizmos.line_2d(
            dest + Vec2::new(-x_size, -x_size),
            dest + Vec2::new(x_size, x_size),
            x_color,
        );
        gizmos.line_2d(
            dest + Vec2::new(-x_size, x_size),
            dest + Vec2::new(x_size, -x_size),
            x_color,
        );

        // Draw small circle around X for emphasis
        gizmos.circle_2d(Isometry2d::from_translation(dest), x_size * 1.2, x_color);
    }
}

/// System that detects arrival at port tiles and triggers state transition.
pub fn port_arrival_system(
    query: Query<&Transform, (With<Player>, With<Ship>)>,
    map_data: Res<MapData>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for transform in &query {
        let pos = transform.translation.truncate();
        let tile = world_to_tile(pos, map_data.width, map_data.height);
        
        if tile.x >= 0 && tile.y >= 0 {
            if let Some(tile_type) = map_data.get(tile.x as u32, tile.y as u32) {
                if tile_type.is_port() {
                    info!("Arrived at port at tile ({}, {})", tile.x, tile.y);
                    next_state.set(GameState::Port);
                }
            }
        }
    }
}
