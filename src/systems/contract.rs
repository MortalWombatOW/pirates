use bevy::prelude::*;

use crate::components::contract::{AcceptedContract, Contract, ContractDetails};
use crate::events::ContractExpiredEvent;
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
