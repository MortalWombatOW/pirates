use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{Player, Ship, Destination, NavigationPath};
use crate::resources::{MapData, Wind};
use crate::plugins::core::GameState;
use crate::utils::pathfinding::{find_path, tile_to_world, world_to_tile};

/// System that handles mouse clicks to set navigation destination.
pub fn click_to_navigate_system(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
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

/// System that calculates A* path when destination changes.
pub fn pathfinding_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Destination), (With<Player>, Changed<Destination>)>,
    map_data: Res<MapData>,
) {
    for (entity, transform, destination) in &query {
        let current_pos = transform.translation.truncate();
        let start_tile = world_to_tile(current_pos, map_data.width, map_data.height);
        let goal_tile = world_to_tile(destination.target, map_data.width, map_data.height);
        
        if let Some(tile_path) = find_path(start_tile, goal_tile, &map_data) {
            // Convert tile path to world waypoints
            let waypoints: Vec<Vec2> = tile_path
                .into_iter()
                .map(|t| tile_to_world(t, map_data.width, map_data.height))
                .collect();
            
            info!("Path found with {} waypoints", waypoints.len());
            commands.entity(entity).insert(NavigationPath { waypoints });
        } else {
            warn!("No path found from ({}, {}) to ({}, {})", 
                  start_tile.x, start_tile.y, goal_tile.x, goal_tile.y);
            // Remove destination if no path
            commands.entity(entity).remove::<Destination>();
        }
    }
}

/// System that moves the player along the navigation path.
pub fn navigation_movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut NavigationPath), With<Player>>,
    wind: Res<Wind>,
    time: Res<Time>,
) {
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
        
        // Base speed with wind effect
        let base_speed = 300.0;
        let wind_alignment = direction_normalized.dot(wind.direction_vec());
        let wind_effect = wind_alignment * wind.strength * 0.5;
        let speed = base_speed * (1.0 + wind_effect);
        
        // Move toward waypoint
        let movement = direction_normalized * speed * time.delta_secs();
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;
        
        // Face direction of movement
        let angle = direction_normalized.y.atan2(direction_normalized.x) - std::f32::consts::FRAC_PI_2;
        transform.rotation = Quat::from_rotation_z(angle);
    }
}

/// System that visualizes the navigation path using gizmos.
pub fn path_visualization_system(
    query: Query<(&Transform, &NavigationPath), With<Player>>,
    mut gizmos: Gizmos,
) {
    for (transform, path) in &query {
        if path.waypoints.is_empty() {
            continue;
        }
        
        let path_color = Color::srgba(1.0, 0.8, 0.2, 0.8); // Golden yellow
        let waypoint_color = Color::srgba(1.0, 0.5, 0.0, 0.6); // Orange
        
        let current_pos = transform.translation.truncate();
        
        // Draw line from current position to first waypoint
        if let Some(&first) = path.waypoints.first() {
            gizmos.line_2d(current_pos, first, path_color);
        }
        
        // Draw lines between waypoints
        for window in path.waypoints.windows(2) {
            gizmos.line_2d(window[0], window[1], path_color);
        }
        
        // Draw circles at waypoints
        for waypoint in &path.waypoints {
            gizmos.circle_2d(Isometry2d::from_translation(*waypoint), 8.0, waypoint_color);
        }
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
