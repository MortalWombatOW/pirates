//! Ink Reveal System
//!
//! Handles animated "ink spreading" effects when fog of war tiles are revealed.

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::components::ink_reveal::InkReveal;
use crate::components::HighSeasEntity;
use crate::plugins::worldmap::FogMap;
use crate::resources::FogOfWar;

/// System that spawns InkReveal entities for newly explored tiles.
/// Takes tiles from FogOfWar.take_newly_explored() and creates animation components.
pub fn spawn_ink_reveals(
    mut commands: Commands,
    mut fog: ResMut<FogOfWar>,
    time: Res<Time>,
) {
    let newly_explored = fog.take_newly_explored();
    
    if newly_explored.is_empty() {
        return;
    }
    
    let current_time = time.elapsed_secs();
    
    for tile_pos in newly_explored {
        // Spawn an entity to track this tile's reveal animation
        commands.spawn((
            Name::new(format!("InkReveal_{},{}", tile_pos.x, tile_pos.y)),
            InkReveal::new(tile_pos, current_time),
            HighSeasEntity,
        ));
    }
}

/// System that updates fog tilemap tiles based on InkReveal animation progress.
/// Animates tile alpha from fully opaque fog to transparent (revealed).
pub fn animate_ink_reveals(
    mut commands: Commands,
    time: Res<Time>,
    reveals: Query<(Entity, &InkReveal)>,
    fog_tilemap_query: Query<(&TilemapSize, &TileStorage), With<FogMap>>,
    mut tile_query: Query<&mut TileColor>,
) {
    let current_time = time.elapsed_secs();
    
    // Get the fog tilemap
    let Ok((tilemap_size, tile_storage)) = fog_tilemap_query.get_single() else {
        return;
    };
    
    for (entity, reveal) in reveals.iter() {
        let progress = reveal.eased_progress(current_time);
        
        // Calculate tile position in tilemap coordinates
        let tile_pos = TilePos {
            x: reveal.tile_pos.x as u32,
            y: reveal.tile_pos.y as u32,
        };
        
        // Bounds check
        if tile_pos.x >= tilemap_size.x || tile_pos.y >= tilemap_size.y {
            // Clean up out-of-bounds reveals
            if reveal.is_complete(current_time) {
                commands.entity(entity).despawn();
            }
            continue;
        }
        
        // Get the tile entity
        if let Some(tile_entity) = tile_storage.get(&tile_pos) {
            if let Ok(mut tile_color) = tile_query.get_mut(tile_entity) {
                // Animate alpha from 1.0 (fog) to 0.0 (revealed)
                let alpha = 1.0 - progress;
                tile_color.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
            }
        }
        
        // Clean up completed animations
        if reveal.is_complete(current_time) {
            commands.entity(entity).despawn();
        }
    }
}
