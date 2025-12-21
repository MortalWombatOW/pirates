//! Order execution system for AI ship behavior.
//!
//! Translates orders from OrderQueue into navigation destinations and actions.

use bevy::prelude::*;

use crate::components::{AI, Ship, Order, OrderQueue, Destination, NavigationPath, Port};
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
                );
            }
            Order::Escort { target, follow_distance } => {
                execute_escort(
                    &mut commands,
                    entity,
                    transform,
                    *target,
                    *follow_distance,
                );
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
fn execute_patrol(
    commands: &mut Commands,
    entity: Entity,
    ship_transform: &Transform,
    center: Vec2,
    radius: f32,
    waypoint_index: u32,
    order_queue: &mut OrderQueue,
) {
    // Generate waypoint based on index (deterministic-ish based on index)
    let angle = (waypoint_index as f32 * 2.4) % std::f32::consts::TAU;
    let dist = radius * 0.3 + (waypoint_index as f32 * 0.7) % (radius * 0.7);
    let target = center + Vec2::new(angle.cos() * dist, angle.sin() * dist);
    
    // Check if arrived at waypoint
    let ship_pos = ship_transform.translation.truncate();
    if ship_pos.distance(target) < 50.0 {
        // Move to next waypoint
        order_queue.pop();
        order_queue.push(Order::Patrol {
            center,
            radius,
            waypoint_index: waypoint_index.wrapping_add(1),
        });
    } else {
        // Navigate to waypoint
        commands.entity(entity).insert(Destination { target });
    }
}

/// Executes Escort order logic.
/// 
/// Ship follows the target entity at the specified distance.
fn execute_escort(
    _commands: &mut Commands,
    _entity: Entity,
    _ship_transform: &Transform,
    target: Entity,
    _follow_distance: f32,
) {
    // TODO(5.3.6): Query target entity position and navigate to follow
    // For now, this is a stub that does nothing
    debug!("Escort order not yet implemented for entity {:?}", target);
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
