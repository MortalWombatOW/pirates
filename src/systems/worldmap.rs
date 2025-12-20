use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::resources::{FogOfWar, MapData};
use crate::components::{Player, Vision};

/// Marker component for tiles in the fog layer.
#[derive(Component)]
pub struct FogTile;

/// System that updates the `FogOfWar` resource based on entities with `Vision`.
pub fn fog_of_war_update_system(
    mut fog_of_war: ResMut<FogOfWar>,
    query: Query<(&Transform, &Vision), With<Player>>,
    map_data: Res<MapData>,
) {
    let tile_size = 64.0;
    let map_width = map_data.width as f32;
    let map_height = map_data.height as f32;

    for (transform, vision) in &query {
        let pos = transform.translation.truncate();
        
        // Convert world position to tile coordinates
        // World (0,0) is map center.
        let tile_x = (pos.x / tile_size + map_width / 2.0).floor() as i32;
        let tile_y = (pos.y / tile_size + map_height / 2.0).floor() as i32;
        
        let radius = vision.radius as i32;
        
        // Reveal tiles within radius
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                // Circular radius check
                if dx * dx + dy * dy <= radius * radius {
                    let tx = tile_x + dx;
                    let ty = tile_y + dy;
                    
                    if tx >= 0 && tx < map_data.width as i32 && ty >= 0 && ty < map_data.height as i32 {
                        fog_of_war.explore(IVec2::new(tx, ty));
                    }
                }
            }
        }
    }
}

/// System that updates the visual representation of fog tiles.
/// Only updates tiles that were newly explored (not all 262k tiles).
pub fn update_fog_tilemap_system(
    mut fog_of_war: ResMut<FogOfWar>,
    mut tile_query: Query<(&TilePos, &mut TileColor), With<FogTile>>,
    _player_query: Query<&Transform, With<Player>>,
    _map_data: Res<MapData>,
) {
    // Only process if there are newly explored tiles
    if !fog_of_war.has_newly_explored() {
        return;
    }

    // Take the list of newly explored tiles (clears the list)
    let newly_explored = fog_of_war.take_newly_explored();
    
    // Build a set of newly explored positions for O(1) lookup
    let newly_explored_set: bevy::utils::HashSet<IVec2> = newly_explored.into_iter().collect();
    
    // Only iterate tiles and update those that were newly explored
    // This is still O(n) for tiles but the early continue makes it much faster
    // A better approach would be to store Entity references, but this is simpler
    for (pos, mut color) in &mut tile_query {
        let tile_pos = IVec2::new(pos.x as i32, pos.y as i32);
        
        if newly_explored_set.contains(&tile_pos) {
            // Set alpha to 0 for explored tiles (make transparent)
            color.0.set_alpha(0.0);
        }
    }
}

