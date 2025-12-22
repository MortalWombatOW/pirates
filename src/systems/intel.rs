use bevy::prelude::*;

use crate::components::intel::{Intel, IntelData, IntelType, IntelExpiry, AcquiredIntel};
use crate::events::IntelAcquiredEvent;
use crate::resources::FogOfWar;

/// System that processes acquired intel and applies its effects.
/// 
/// Handles different intel types:
/// - `MapReveal`: Reveals positions in the fog of war
/// - `ShipRoute`: Stores route for visualization (handled by UI)
/// - Other types: Marks as acquired for player reference
pub fn intel_acquisition_system(
    mut events: EventReader<IntelAcquiredEvent>,
    mut commands: Commands,
    mut fog_of_war: ResMut<FogOfWar>,
    intel_query: Query<&IntelData>,
) {
    for event in events.read() {
        // Get the intel data
        let Ok(intel_data) = intel_query.get(event.intel_entity) else {
            warn!("IntelAcquiredEvent for non-existent intel entity {:?}", event.intel_entity);
            continue;
        };

        // Mark intel as acquired
        commands.entity(event.intel_entity).insert(AcquiredIntel);

        // Process based on intel type
        match intel_data.intel_type {
            IntelType::MapReveal => {
                // Reveal all positions in the fog of war
                let revealed_count = intel_data.revealed_positions
                    .iter()
                    .filter(|pos| fog_of_war.explore(**pos))
                    .count();
                
                info!(
                    "MapReveal intel acquired: {} new tiles revealed",
                    revealed_count
                );
            }
            IntelType::TreasureLocation => {
                // Reveal treasure location on map
                for pos in &intel_data.revealed_positions {
                    fog_of_war.explore(*pos);
                }
                info!(
                    "TreasureLocation intel acquired at {:?}",
                    intel_data.revealed_positions.first()
                );
            }
            IntelType::FleetPosition => {
                // Reveal fleet position on map
                for pos in &intel_data.revealed_positions {
                    fog_of_war.explore(*pos);
                }
                info!(
                    "FleetPosition intel acquired: target {:?}",
                    intel_data.target_entity
                );
            }
            IntelType::ShipRoute => {
                // Route waypoints are stored for UI visualization
                // Optionally reveal waypoint tiles
                for pos in &intel_data.route_waypoints {
                    fog_of_war.explore(*pos);
                }
                info!(
                    "ShipRoute intel acquired: {} waypoints",
                    intel_data.route_waypoints.len()
                );
            }
            IntelType::PortInventory => {
                // No map reveal, just stored for player reference
                info!(
                    "PortInventory intel acquired for port {:?}",
                    intel_data.target_entity
                );
            }
            IntelType::Rumor => {
                // Rumors may hint at other intel, no direct map effect
                info!("Rumor acquired: {}", intel_data.description);
            }
        }
    }
}

/// System that checks for and removes expired intel.
/// 
/// Runs on FixedUpdate after world_tick_system.
/// Transient intel with IntelExpiry component is despawned when TTL expires.
pub fn intel_expiry_system(
    mut commands: Commands,
    world_clock: Res<crate::resources::WorldClock>,
    intel_query: Query<(Entity, &Intel, &IntelExpiry)>,
) {
    let current_tick = world_clock.total_ticks();
    
    for (entity, _intel, expiry) in intel_query.iter() {
        if expiry.is_expired(current_tick) {
            debug!("Intel {:?} expired at tick {}", entity, current_tick);
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// System that visualizes acquired intel on the High Seas map using Gizmos.
/// 
/// Draws:
/// - Diamond markers for TreasureLocation and FleetPosition intel
/// - Dotted lines for ShipRoute intel waypoints
pub fn intel_visualization_system(
    intel_query: Query<&IntelData, (With<Intel>, With<AcquiredIntel>)>,
    map_data: Res<crate::resources::MapData>,
    mut gizmos: Gizmos,
) {
    // Colors for different intel types
    let treasure_color = Color::srgba(1.0, 0.85, 0.0, 0.9);  // Gold
    let fleet_color = Color::srgba(0.9, 0.2, 0.2, 0.9);       // Red
    let route_color = Color::srgba(0.2, 0.6, 0.9, 0.7);       // Blue
    
    for intel_data in intel_query.iter() {
        match intel_data.intel_type {
            IntelType::TreasureLocation => {
                // Draw gold diamond markers for treasure locations
                for tile_pos in &intel_data.revealed_positions {
                    let world_pos = tile_to_world(*tile_pos, map_data.width, map_data.height);
                    draw_diamond_marker(&mut gizmos, world_pos, 30.0, treasure_color);
                    // Draw a small X inside
                    draw_x_marker(&mut gizmos, world_pos, 10.0, treasure_color);
                }
            }
            IntelType::FleetPosition => {
                // Draw red diamond markers for fleet positions
                for tile_pos in &intel_data.revealed_positions {
                    let world_pos = tile_to_world(*tile_pos, map_data.width, map_data.height);
                    draw_diamond_marker(&mut gizmos, world_pos, 25.0, fleet_color);
                    // Draw anchor symbol (simple cross)
                    gizmos.line_2d(
                        world_pos + Vec2::new(0.0, -15.0),
                        world_pos + Vec2::new(0.0, 15.0),
                        fleet_color,
                    );
                    gizmos.line_2d(
                        world_pos + Vec2::new(-10.0, 5.0),
                        world_pos + Vec2::new(10.0, 5.0),
                        fleet_color,
                    );
                }
            }
            IntelType::ShipRoute => {
                // Draw dotted line for ship routes
                if intel_data.route_waypoints.len() >= 2 {
                    for i in 0..intel_data.route_waypoints.len() - 1 {
                        let start_tile = intel_data.route_waypoints[i];
                        let end_tile = intel_data.route_waypoints[i + 1];
                        let start_pos = tile_to_world(start_tile, map_data.width, map_data.height);
                        let end_pos = tile_to_world(end_tile, map_data.width, map_data.height);
                        
                        // Draw dotted line between waypoints
                        draw_dotted_line(&mut gizmos, start_pos, end_pos, 15.0, route_color);
                    }
                    
                    // Draw small circle at each waypoint
                    for &tile_pos in &intel_data.route_waypoints {
                        let world_pos = tile_to_world(tile_pos, map_data.width, map_data.height);
                        gizmos.circle_2d(Isometry2d::from_translation(world_pos), 8.0, route_color);
                    }
                }
            }
            // MapReveal is handled by fog of war, no additional visualization needed
            // Rumor and PortInventory have no map visualization
            _ => {}
        }
    }
}

/// Draws a diamond marker at the given position.
fn draw_diamond_marker(gizmos: &mut Gizmos, center: Vec2, size: f32, color: Color) {
    let half = size / 2.0;
    let top = center + Vec2::new(0.0, half);
    let bottom = center + Vec2::new(0.0, -half);
    let left = center + Vec2::new(-half, 0.0);
    let right = center + Vec2::new(half, 0.0);
    
    gizmos.line_2d(top, right, color);
    gizmos.line_2d(right, bottom, color);
    gizmos.line_2d(bottom, left, color);
    gizmos.line_2d(left, top, color);
}

/// Draws an X marker at the given position.
fn draw_x_marker(gizmos: &mut Gizmos, center: Vec2, size: f32, color: Color) {
    let half = size / 2.0;
    gizmos.line_2d(
        center + Vec2::new(-half, -half),
        center + Vec2::new(half, half),
        color,
    );
    gizmos.line_2d(
        center + Vec2::new(-half, half),
        center + Vec2::new(half, -half),
        color,
    );
}

/// Draws a dotted line between two points.
fn draw_dotted_line(gizmos: &mut Gizmos, start: Vec2, end: Vec2, dash_length: f32, color: Color) {
    let direction = end - start;
    let distance = direction.length();
    if distance < 0.1 {
        return;
    }
    
    let normalized = direction / distance;
    let gap_length = dash_length * 0.75;
    let segment_length = dash_length + gap_length;
    
    let mut current = 0.0;
    let mut draw = true;
    
    while current < distance {
        let next = (current + if draw { dash_length } else { gap_length }).min(distance);
        
        if draw {
            let start_pos = start + normalized * current;
            let end_pos = start + normalized * next;
            gizmos.line_2d(start_pos, end_pos, color);
        }
        
        current = next;
        draw = !draw;
    }
}

/// Converts tile coordinates to world coordinates.
/// Duplicated from pathfinding utility to avoid circular imports.
fn tile_to_world(tile_pos: IVec2, map_width: u32, map_height: u32) -> Vec2 {
    const TILE_SIZE: f32 = 64.0;
    let x = (tile_pos.x as f32 - map_width as f32 / 2.0) * TILE_SIZE + TILE_SIZE / 2.0;
    let y = (tile_pos.y as f32 - map_height as f32 / 2.0) * TILE_SIZE + TILE_SIZE / 2.0;
    Vec2::new(x, y)
}
