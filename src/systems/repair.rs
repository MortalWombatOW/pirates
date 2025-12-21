use bevy::prelude::*;

use crate::components::{
    cargo::Gold,
    health::{Health, WaterIntake},
    ship::{Player, Ship},
};
use crate::events::{RepairRequestEvent, RepairType};

/// Repair cost configuration.
pub mod repair_config {
    /// Base cost per HP to repair sails.
    pub const SAILS_COST_PER_HP: f32 = 1.0;
    /// Base cost per HP to repair rudder.
    pub const RUDDER_COST_PER_HP: f32 = 1.5;
    /// Base cost per HP to repair hull.
    pub const HULL_COST_PER_HP: f32 = 2.0;
}

/// Calculates repair cost for the given component and damage amount.
pub fn calculate_repair_cost(repair_type: RepairType, damage: f32) -> u32 {
    let cost_per_hp = match repair_type {
        RepairType::Sails => repair_config::SAILS_COST_PER_HP,
        RepairType::Rudder => repair_config::RUDDER_COST_PER_HP,
        RepairType::Hull => repair_config::HULL_COST_PER_HP,
    };
    (damage * cost_per_hp).ceil() as u32
}

/// System that handles repair requests by restoring component HP and deducting gold.
/// 
/// For hull repairs, also removes the WaterIntake component if present.
pub fn repair_execution_system(
    mut commands: Commands,
    mut repair_events: EventReader<RepairRequestEvent>,
    mut player_query: Query<(Entity, &mut Health, &mut Gold), (With<Player>, With<Ship>)>,
) {
    for event in repair_events.read() {
        let Ok((entity, mut health, mut gold)) = player_query.get_single_mut() else {
            warn!("Repair failed: Player ship not found");
            continue;
        };

        match event.repair_type {
            RepairType::Sails => {
                let damage = health.sails_max - health.sails;
                if damage <= 0.0 {
                    info!("Sails already at full health");
                    continue;
                }
                let cost = calculate_repair_cost(RepairType::Sails, damage);
                if !gold.spend(cost) {
                    info!("Cannot afford sails repair ({} gold needed)", cost);
                    continue;
                }
                health.sails = health.sails_max;
                info!("Repaired sails for {} gold", cost);
            }
            RepairType::Rudder => {
                let damage = health.rudder_max - health.rudder;
                if damage <= 0.0 {
                    info!("Rudder already at full health");
                    continue;
                }
                let cost = calculate_repair_cost(RepairType::Rudder, damage);
                if !gold.spend(cost) {
                    info!("Cannot afford rudder repair ({} gold needed)", cost);
                    continue;
                }
                health.rudder = health.rudder_max;
                info!("Repaired rudder for {} gold", cost);
            }
            RepairType::Hull => {
                let damage = health.hull_max - health.hull;
                if damage <= 0.0 {
                    info!("Hull already at full health");
                    continue;
                }
                let cost = calculate_repair_cost(RepairType::Hull, damage);
                if !gold.spend(cost) {
                    info!("Cannot afford hull repair ({} gold needed)", cost);
                    continue;
                }
                health.hull = health.hull_max;
                
                // Remove WaterIntake when hull is fully repaired
                commands.entity(entity).remove::<WaterIntake>();
                
                info!("Repaired hull for {} gold (WaterIntake removed)", cost);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_repair_cost_sails() {
        let cost = calculate_repair_cost(RepairType::Sails, 50.0);
        assert_eq!(cost, 50); // 50 * 1.0 = 50
    }

    #[test]
    fn test_calculate_repair_cost_rudder() {
        let cost = calculate_repair_cost(RepairType::Rudder, 50.0);
        assert_eq!(cost, 75); // 50 * 1.5 = 75
    }

    #[test]
    fn test_calculate_repair_cost_hull() {
        let cost = calculate_repair_cost(RepairType::Hull, 50.0);
        assert_eq!(cost, 100); // 50 * 2.0 = 100
    }

    #[test]
    fn test_cost_rounds_up() {
        let cost = calculate_repair_cost(RepairType::Sails, 1.1);
        assert_eq!(cost, 2); // 1.1 * 1.0 = 1.1, ceil = 2
    }
}
