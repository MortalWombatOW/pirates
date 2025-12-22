use bevy::prelude::*;

use crate::components::contract::{AcceptedContract, AssignedShip, Contract, ContractDetails, ContractProgress, ContractType};
use crate::components::{Order, OrderQueue, Port, NavigationPath, PlayerOwned};
use crate::events::{ContractExpiredEvent, ContractCompletedEvent};
use crate::resources::WorldClock;

/// System that checks for and removes expired contracts.
/// 
/// Runs on FixedUpdate after world_tick_system.
/// Emits `ContractExpiredEvent` for each expired contract before despawning.
pub fn contract_expiry_system(
    mut commands: Commands,
    mut expire_events: EventWriter<ContractExpiredEvent>,
    world_clock: Res<WorldClock>,
    contract_query: Query<(Entity, &ContractDetails, Option<&AcceptedContract>), With<Contract>>,
) {
    let current_tick = world_clock.total_ticks();
    
    for (entity, details, accepted) in contract_query.iter() {
        if details.is_expired(current_tick) {
            let was_accepted = accepted.is_some();
            
            // Emit event before despawning
            expire_events.send(ContractExpiredEvent {
                contract_entity: entity,
                was_accepted,
            });
            
            if was_accepted {
                info!("Contract expired (was accepted): {:?}", entity);
            } else {
                debug!("Contract expired (unaccepted): {:?}", entity);
            }
            
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// System that processes delegated contracts and sets ship orders.
///
/// For Transport contracts:
/// 1. If ship has no order, set order to go to destination port
/// 2. Check if ship arrived (no navigation path and close to destination)
/// 3. On arrival, complete the contract
pub fn contract_delegation_system(
    mut commands: Commands,
    mut completion_events: EventWriter<ContractCompletedEvent>,
    mut contract_query: Query<
        (Entity, &ContractDetails, &AssignedShip, &mut ContractProgress),
        (With<Contract>, With<AcceptedContract>),
    >,
    mut ship_query: Query<
        (Entity, &Transform, Option<&OrderQueue>, Option<&NavigationPath>),
        With<PlayerOwned>,
    >,
    port_query: Query<&Transform, With<Port>>,
    mut player_gold: Query<&mut crate::components::Gold, With<crate::components::Player>>,
) {
    for (contract_entity, details, assigned, mut progress) in contract_query.iter_mut() {
        // Only handle Transport contracts for MVP
        if details.contract_type != ContractType::Transport {
            continue;
        }

        let Some(destination_entity) = details.destination else {
            continue;
        };

        // Get the assigned ship
        let Ok((ship_entity, ship_transform, order_queue, nav_path)) = ship_query.get(assigned.ship_entity) else {
            // Ship no longer exists - could remove assignment
            continue;
        };

        // Get destination port position
        let Ok(dest_transform) = port_query.get(destination_entity) else {
            continue;
        };
        let dest_pos = dest_transform.translation.truncate();
        let ship_pos = ship_transform.translation.truncate();
        let distance = ship_pos.distance(dest_pos);

        // Check if ship needs orders
        let needs_order = order_queue
            .map(|q| q.is_empty())
            .unwrap_or(true);
        
        let is_navigating = nav_path
            .map(|p| !p.is_empty())
            .unwrap_or(false);

        // If ship arrived at destination (close and not navigating)
        if distance < 100.0 && !is_navigating && !progress.destination_reached {
            progress.destination_reached = true;
            
            // Calculate reward with player cut
            let base_reward = details.reward_gold;
            let player_reward = (base_reward as f32 * assigned.player_cut) as u32;
            
            // Pay the player
            if let Ok(mut gold) = player_gold.get_single_mut() {
                gold.add(player_reward);
                info!(
                    "Contract completed by fleet ship! Reward: {} gold ({}% cut)",
                    player_reward,
                    (assigned.player_cut * 100.0) as u32
                );
            }

            // Emit completion event
            completion_events.send(ContractCompletedEvent {
                contract_entity,
                reward_gold: player_reward,
            });

            // Remove the contract
            commands.entity(contract_entity).despawn_recursive();
            
            // Set ship back to idle
            if let Ok(ship) = ship_query.get_mut(assigned.ship_entity) {
                commands.entity(ship.0).insert(OrderQueue::with_order(Order::Idle));
            }
        } else if needs_order && !progress.destination_reached {
            // Set order to go to destination
            commands.entity(ship_entity).insert(
                OrderQueue::with_order(Order::TradeRoute {
                    origin: details.origin_port,
                    destination: destination_entity,
                    outbound: true,
                })
            );
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::cargo::GoodType;

    fn create_test_entity() -> Entity {
        Entity::from_raw(1)
    }

    #[test]
    fn test_contract_expiry_detection() {
        let origin = create_test_entity();
        let dest = create_test_entity();
        
        // Contract expiring at tick 100
        let mut details = ContractDetails::transport(origin, dest, GoodType::Rum, 10, 100);
        details.expiry_tick = Some(100);
        
        assert!(!details.is_expired(50));
        assert!(!details.is_expired(99));
        assert!(details.is_expired(100));
        assert!(details.is_expired(150));
    }

    #[test]
    fn test_contract_no_expiry() {
        let origin = create_test_entity();
        let dest = create_test_entity();
        
        // Contract with no expiry
        let details = ContractDetails::transport(origin, dest, GoodType::Rum, 10, 100);
        
        assert!(!details.is_expired(0));
        assert!(!details.is_expired(1_000_000));
    }

    #[test]
    fn test_transport_with_expiry() {
        let origin = create_test_entity();
        let dest = create_test_entity();
        
        let details = ContractDetails::transport_with_expiry(
            origin, dest, GoodType::Rum, 10, 100, 1000
        );
        
        // Should expire at 1000 + DEFAULT_DURATION_TICKS
        let expected_expiry = 1000 + ContractDetails::DEFAULT_DURATION_TICKS;
        assert_eq!(details.expiry_tick, Some(expected_expiry));
        assert!(details.is_expired(expected_expiry));
        assert!(!details.is_expired(expected_expiry - 1));
    }
}
