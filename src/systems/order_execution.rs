//! Order execution system for AI ship behavior.
//!
//! Translates orders from OrderQueue into navigation destinations and actions.

use bevy::prelude::*;

use crate::components::{AI, Ship, Order, OrderQueue, Destination, NavigationPath, Port, Player};
use crate::plugins::worldmap::HighSeasAI;

/// System that reads orders from AI ships and sets navigation destinations.
///
/// This system:
/// - Queries AI ships with OrderQueue
/// - Reads the current (front) order
/// - Sets Destination based on order type
/// - Handles order completion transitions
pub fn order_execution_system(
    mut commands: Commands,
    mut ai_query: Query<
        (Entity, &Transform, &mut OrderQueue, Option<&NavigationPath>),
        (With<AI>, With<Ship>, With<HighSeasAI>),
    >,
    port_query: Query<&Transform, With<Port>>,
    // General query for looking up random targets (Player, other ships)
    // Note: This conflicts with ai_query if we aren't careful. 
    // Since ai_query is mut, we can't have another query overlapping it easily in the same system without specific filters.
    // BUT: We only need read access to targets.
    // If we use `Query<&Transform>` it will conflict with `&mut Transform` in ai_query.
    // CORRECT APPROACH: Use `transmute_lens` or `ParamSet`? Too complex for this snippet.
    // SIMPLE APPROACH: We already have `ai_query`. If target is an AI ship, we can't easily look it up in a separate query.
    // If target is Player, we need a separate query for Player.
    // If target is Port, we have port_query.
    // Check INVARIANTS: "Order execution system...".
    // Let's add a specific Player query for now, as Escort usually targets Player or specific VIPs.
    player_query: Query<&Transform, With<Player>>,
    map_data: Res<MapData>,
) {
    for (entity, transform, mut order_queue, nav_path) in &mut ai_query {
        // Skip if ship is currently navigating (has remaining waypoints)
        if let Some(path) = nav_path {
            if !path.is_empty() {
                continue;
            }
        }

        // Get current order
        let Some(order) = order_queue.current().cloned() else {
            // No orders - ship is idle
            continue;
        };

        // Execute order based on type
        match &order {
            Order::TradeRoute { origin, destination, outbound } => {
                execute_trade_route(
                    &mut commands,
                    entity,
                    transform,
                    &port_query,
                    *origin,
                    *destination,
                    *outbound,
                    &mut order_queue,
                );
            }
            Order::Patrol { center, radius, waypoint_index } => {
                execute_patrol(
                    &mut commands,
                    entity,
                    transform,
                    *center,
                    *radius,
                    *waypoint_index,
                    &mut order_queue,
                    &map_data,
                );
            }
            Order::Escort { target, follow_distance } => {
                // Try to find target in player query first
                if let Ok(target_transform) = player_query.get(*target) {
                     execute_escort(
                        &mut commands,
                        entity,
                        transform,
                        target_transform, // Pass transform directly
                        *follow_distance,
                    );
                } else {
                    // Could also be another AI ship... 
                    // For now, only supporting Player or Port as robust targets in this simple system
                    // If we strictly implement "Escort Player", this is fine.
                    debug!("Escort: Target {:?} not found in Player query", target);
                }
            }
            Order::Scout { area_center, area_radius, progress } => {
                execute_scout(
                    &mut commands,
                    entity,
                    transform,
                    *area_center,
                    *area_radius,
                    *progress,
                    &mut order_queue,
                );
            }
            Order::Idle => {
                // No action needed for idle
            }
        }
    }
}

/// Executes TradeRoute order logic.
/// 
/// Ship navigates to destination port. When arrived, toggles outbound flag
/// and continues to the other port.
fn execute_trade_route(
    commands: &mut Commands,
    entity: Entity,
    ship_transform: &Transform,
    port_query: &Query<&Transform, With<Port>>,
    origin: Entity,
    destination: Entity,
    outbound: bool,
    order_queue: &mut OrderQueue,
) {
    // Determine target port based on outbound flag
    let target_port = if outbound { destination } else { origin };
    
    // Get port position
    let Ok(port_transform) = port_query.get(target_port) else {
        // Port doesn't exist - remove order
        debug!("TradeRoute: Port entity no longer exists, removing order");
        order_queue.pop();
        return;
    };
    let target_pos = port_transform.translation.truncate();
    
    // Check if we've arrived at the port
    let ship_pos = ship_transform.translation.truncate();
    let distance = ship_pos.distance(target_pos);
    
    if distance < 100.0 {
        // Arrived at port - toggle direction and continue
        order_queue.pop();
        order_queue.push(Order::TradeRoute {
            origin,
            destination,
            outbound: !outbound,
        });
        debug!("TradeRoute: Arrived at port, reversing direction");
    } else {
        // Navigate to port
        commands.entity(entity).insert(Destination { target: target_pos });
    }
}

/// Executes Patrol order logic.
/// 
/// Ship moves between random waypoints within the patrol radius.
/// Verifies that generated waypoints are navigable water tiles.
fn execute_patrol(
    commands: &mut Commands,
    entity: Entity,
    ship_transform: &Transform,
    center: Vec2,
    radius: f32,
    waypoint_index: u32,
    order_queue: &mut OrderQueue,
    map_data: &MapData,
) {
    // Generate waypoint based on index (deterministic-ish based on index)
    // Try multiple offsets to find a navigable point
    let mut target = None;
    let current_idx = waypoint_index;
    
    // Try up to 5 times to find a valid spot
    for i in 0..5 {
        let try_idx = current_idx + i as u32;
        let angle = (try_idx as f32 * 2.4) % std::f32::consts::TAU;
        let dist = radius * 0.3 + (try_idx as f32 * 0.7) % (radius * 0.7);
        let candidate = center + Vec2::new(angle.cos() * dist, angle.sin() * dist);
        
        // Check navigability
        let tile_pos = world_to_tile(candidate, map_data.width, map_data.height);
        if map_data.is_navigable(tile_pos.x as u32, tile_pos.y as u32) {
            target = Some((candidate, try_idx));
            break;
        }
    }
    
    let Some((target_pos, valid_idx)) = target else {
        // No valid target found in attempts - just increment and wait next frame
        order_queue.pop();
        order_queue.push(Order::Patrol {
            center,
            radius,
            waypoint_index: waypoint_index + 1,
        });
        return;
    };

    // Check if arrived at waypoint
    let ship_pos = ship_transform.translation.truncate();
    if ship_pos.distance(target_pos) < 50.0 {
        // Move to next waypoint
        order_queue.pop();
        order_queue.push(Order::Patrol {
            center,
            radius,
            waypoint_index: valid_idx.wrapping_add(1),
        });
    } else {
        // Navigate to waypoint
        commands.entity(entity).insert(Destination { target: target_pos });
    }
}

/// Executes Escort order logic.
/// 
/// Ship follows the target entity at the specified distance.
fn execute_escort(
    commands: &mut Commands,
    entity: Entity,
    ship_transform: &Transform,
    target_transform: &Transform,
    follow_distance: f32,
) {
    let target_pos = target_transform.translation.truncate();
    let ship_pos = ship_transform.translation.truncate();
    let distance = ship_pos.distance(target_pos);
    
    // Simple hysteresis: Move if too far, stop if close enough
    if distance > follow_distance * 1.2 {
        // Move towards target
        // Calculate a point 'follow_distance' away from target in direction of ship
        let direction = (ship_pos - target_pos).normalize_or_zero();
        let destination = target_pos + direction * follow_distance;
        
        commands.entity(entity).insert(Destination { target: destination });
    } else if distance < follow_distance * 0.8 {
        // Too close? Back off or just stop. 
        // For now, satisfied.
    }
}

/// Executes Scout order logic.
/// 
/// Ship explores the area systematically, updating progress.
fn execute_scout(
    commands: &mut Commands,
    entity: Entity,
    ship_transform: &Transform,
    area_center: Vec2,
    area_radius: f32,
    progress: f32,
    order_queue: &mut OrderQueue,
) {
    // Simple spiral pattern based on progress
    let angle = progress * std::f32::consts::TAU * 5.0;
    let dist = area_radius * progress.min(1.0);
    let target = area_center + Vec2::new(angle.cos() * dist, angle.sin() * dist);
    
    // Check if arrived at target
    let ship_pos = ship_transform.translation.truncate();
    if ship_pos.distance(target) < 50.0 {
         if progress >= 1.0 {
            // Scouting complete
            order_queue.pop();
            debug!("Scout order complete");
        } else {
            // Continue scouting
            order_queue.pop();
            order_queue.push(Order::Scout {
                area_center,
                area_radius,
                progress: progress + 0.1,
            });
        }
    } else {
        // Navigate to target
        commands.entity(entity).insert(Destination { target });
    }
}

use crate::resources::{RouteCache, MapData};
use crate::utils::pathfinding::{find_path, tile_to_world, world_to_tile};

/// System that calculates simple paths for AI ships.
/// 
/// Uses cached Theta* pathfinding to navigate around land.
pub fn ai_pathfinding_system(
    mut commands: Commands,
    mut query: Query<
        (Entity, &Transform, &Destination),
        (With<AI>, With<Ship>, With<HighSeasAI>, Changed<Destination>),
    >,
    mut route_cache: ResMut<RouteCache>,
    map_data: Res<MapData>,
) {
    for (entity, transform, destination) in &mut query {
        let start_pos = transform.translation.truncate();
        let target_pos = destination.target;
        
        // Convert to tile coordinates
        let start_tile = world_to_tile(start_pos, map_data.width, map_data.height);
        let goal_tile = world_to_tile(target_pos, map_data.width, map_data.height);
        
        // Check cache first
        let tile_path = if let Some(cached) = route_cache.get(start_tile, goal_tile) {
            Some(cached.clone())
        } else {
            // Cache miss - compute path
            if let Some(path) = find_path(start_tile, goal_tile, &map_data) {
                // Store in cache
                route_cache.insert(start_tile, goal_tile, path.clone());
                Some(path)
            } else {
                warn!("AI Pathfinding: No path found from {:?} to {:?}", start_tile, goal_tile);
                None
            }
        };
        
        if let Some(path) = tile_path {
            // Convert tile path to world waypoints
            // Skip the first point (start) as we are already there
            let waypoints: Vec<Vec2> = path.iter()
                .skip(1) 
                .map(|&p| tile_to_world(p, map_data.width, map_data.height))
                .collect();
            
            // If path is empty (start == goal), rely on movement system to handle proximity
            let final_waypoints = if waypoints.is_empty() {
                vec![target_pos]
            } else {
                waypoints
            };
            
            commands.entity(entity).insert(NavigationPath { waypoints: final_waypoints });
        } else {
            // Fallback: Direct line if pathfinding failed (shouldn't happen on valid map)
            commands.entity(entity).insert(NavigationPath { waypoints: vec![target_pos] });
        }
    }
}

/// System that moves AI ships along their navigation paths.
/// 
/// AI ships move at a fixed speed toward their waypoints.
pub fn ai_movement_system(
    mut commands: Commands,
    mut query: Query<
        (Entity, &mut Transform, &mut NavigationPath),
        (With<AI>, With<Ship>, With<HighSeasAI>),
    >,
    time: Res<Time>,
) {
    const AI_SHIP_SPEED: f32 = 150.0;
    
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
        
        // Move toward waypoint
        let movement = direction_normalized * AI_SHIP_SPEED * time.delta_secs();
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;
        
        // Rotate to face direction of movement
        let target_angle = direction_normalized.y.atan2(direction_normalized.x) - std::f32::consts::FRAC_PI_2;
        let target_rotation = Quat::from_rotation_z(target_angle);
        
        // Smooth rotation
        let rotation_speed = 2.0;
        transform.rotation = transform.rotation.slerp(target_rotation, rotation_speed * time.delta_secs());
    }
}
