use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::{
    cargo::{GoodType, GoodsTrait},
    port::{Inventory, InventoryItem, Port},
};

/// Constants for price calculation.
pub mod price_config {
    /// Base prices for each good type (used for equilibrium reference).
    pub fn base_price(good: &super::GoodType) -> f32 {
        match good {
            super::GoodType::Rum => 15.0,
            super::GoodType::Sugar => 8.0,
            super::GoodType::Spices => 25.0,
            super::GoodType::Timber => 5.0,
            super::GoodType::Cloth => 12.0,
            super::GoodType::Weapons => 40.0,
        }
    }

    /// Base (equilibrium) quantity expected for each good type.
    pub fn base_quantity(good: &super::GoodType) -> u32 {
        match good {
            super::GoodType::Rum => 100,
            super::GoodType::Sugar => 140,
            super::GoodType::Spices => 50,
            super::GoodType::Timber => 200,
            super::GoodType::Cloth => 80,
            super::GoodType::Weapons => 30,
        }
    }

    /// Minimum price multiplier (floor).
    pub const MIN_PRICE_MULTIPLIER: f32 = 0.5;
    /// Maximum price multiplier (ceiling).
    pub const MAX_PRICE_MULTIPLIER: f32 = 3.0;
    /// How strongly supply affects price (higher = more volatile).
    pub const SUPPLY_SENSITIVITY: f32 = 0.8;
    /// How strongly global demand affects price.
    pub const DEMAND_SENSITIVITY: f32 = 0.5;
    /// Decay rate for perishable goods (fraction lost per world tick).
    /// At 60Hz, 0.0001 means ~0.6% lost per hour (~14% per day).
    pub const PERISHABLE_DECAY_RATE: f32 = 0.0001;
}

/// Resource tracking global demand levels for each good type.
/// 
/// Demand is a multiplier (1.0 = normal, >1.0 = high demand, <1.0 = low demand).
/// High demand goods have higher prices everywhere.
#[derive(Resource, Debug, Clone)]
pub struct GlobalDemand {
    /// Map of good type to demand multiplier.
    pub demand: HashMap<GoodType, f32>,
}

impl Default for GlobalDemand {
    fn default() -> Self {
        let mut demand = HashMap::new();
        // Initialize all goods with normal demand
        demand.insert(GoodType::Rum, 1.0);
        demand.insert(GoodType::Sugar, 1.0);
        demand.insert(GoodType::Spices, 1.0);
        demand.insert(GoodType::Timber, 1.0);
        demand.insert(GoodType::Cloth, 1.0);
        demand.insert(GoodType::Weapons, 1.0);
        Self { demand }
    }
}

impl GlobalDemand {
    /// Gets the demand multiplier for a good type (defaults to 1.0).
    pub fn get(&self, good: &GoodType) -> f32 {
        *self.demand.get(good).unwrap_or(&1.0)
    }

    /// Sets the demand multiplier for a good type.
    pub fn set(&mut self, good: GoodType, multiplier: f32) {
        self.demand.insert(good, multiplier.clamp(0.5, 2.0));
    }

    /// Increases demand for a good (e.g., after trade or event).
    pub fn increase(&mut self, good: GoodType, amount: f32) {
        let current = self.get(&good);
        self.set(good, current + amount);
    }

    /// Decreases demand for a good.
    pub fn decrease(&mut self, good: GoodType, amount: f32) {
        let current = self.get(&good);
        self.set(good, current - amount);
    }
}

/// System that recalculates prices for all ports based on supply and demand.
/// 
/// Runs every world tick (via FixedUpdate).
/// 
/// **Price Formula:**
/// ```text
/// price = base_price * supply_multiplier * demand_multiplier
/// 
/// supply_ratio = current_quantity / base_quantity
/// supply_multiplier = clamp(1.0 / supply_ratio^sensitivity, min, max)
/// demand_multiplier = global_demand ^ demand_sensitivity
/// ```
/// 
/// Low stock → higher prices, high stock → lower prices.
/// High demand → higher prices everywhere, low demand → lower prices.
pub fn price_calculation_system(
    mut port_query: Query<&mut Inventory, With<Port>>,
    global_demand: Res<GlobalDemand>,
) {
    for mut inventory in port_query.iter_mut() {
        for (good_type, item) in inventory.goods.iter_mut() {
            let demand_mult = global_demand.get(good_type);
            let new_price = calculate_price(good_type, item, demand_mult);
            item.price = new_price;
        }
    }
}

/// Calculates price based on supply and demand.
fn calculate_price(good_type: &GoodType, item: &InventoryItem, demand_multiplier: f32) -> f32 {
    let base_price = price_config::base_price(good_type);
    let base_quantity = price_config::base_quantity(good_type) as f32;
    
    // Handle edge case of zero quantity
    if item.quantity == 0 {
        return base_price * price_config::MAX_PRICE_MULTIPLIER * demand_multiplier.powf(price_config::DEMAND_SENSITIVITY);
    }
    
    let supply_ratio = item.quantity as f32 / base_quantity;
    
    // Supply effect: low supply = high multiplier
    let supply_mult = supply_ratio.powf(-price_config::SUPPLY_SENSITIVITY);
    
    // Demand effect: high demand = higher multiplier
    let demand_effect = demand_multiplier.powf(price_config::DEMAND_SENSITIVITY);
    
    // Combined multiplier with clamping
    let raw_multiplier = supply_mult * demand_effect;
    let clamped_multiplier = raw_multiplier.clamp(
        price_config::MIN_PRICE_MULTIPLIER,
        price_config::MAX_PRICE_MULTIPLIER,
    );
    
    base_price * clamped_multiplier
}

/// Helper for tests - calculate supply-only price (backwards compatibility).
#[cfg(test)]
fn calculate_supply_price(good_type: &GoodType, item: &InventoryItem) -> f32 {
    calculate_price(good_type, item, 1.0)
}

/// System that decays perishable goods in port inventories over time.
/// 
/// Runs every world tick (via FixedUpdate).
/// Perishable goods (Rum, Sugar) gradually lose quantity, simulating spoilage.
pub fn goods_decay_system(
    mut port_query: Query<&mut Inventory, With<Port>>,
) {
    for mut inventory in port_query.iter_mut() {
        let decay_rate = price_config::PERISHABLE_DECAY_RATE;
        
        for (good_type, item) in inventory.goods.iter_mut() {
            if good_type.traits().contains(&GoodsTrait::Perishable) {
                // Decay perishable goods
                let decay_amount = (item.quantity as f32 * decay_rate).max(0.0);
                let lost = decay_amount.floor() as u32;
                
                // Use fractional accumulation for small decay amounts
                // For now, just apply whole unit decay when enough accumulates
                if lost > 0 {
                    item.quantity = item.quantity.saturating_sub(lost);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_supply_increases_price() {
        let item = InventoryItem::new(10, 15.0); // Very low rum (base 100)
        let price = calculate_supply_price(&GoodType::Rum, &item);
        let base = price_config::base_price(&GoodType::Rum);
        assert!(price > base, "Low supply should increase price");
    }

    #[test]
    fn test_high_supply_decreases_price() {
        let item = InventoryItem::new(300, 15.0); // Very high rum (base 100)
        let price = calculate_supply_price(&GoodType::Rum, &item);
        let base = price_config::base_price(&GoodType::Rum);
        assert!(price < base, "High supply should decrease price");
    }

    #[test]
    fn test_equilibrium_price() {
        let base_qty = price_config::base_quantity(&GoodType::Rum);
        let item = InventoryItem::new(base_qty, 15.0);
        let price = calculate_supply_price(&GoodType::Rum, &item);
        let base = price_config::base_price(&GoodType::Rum);
        // At equilibrium, price should be close to base
        assert!((price - base).abs() < 1.0, "Equilibrium should yield base price");
    }

    #[test]
    fn test_zero_quantity_caps_at_max() {
        let item = InventoryItem::new(0, 15.0);
        let price = calculate_supply_price(&GoodType::Rum, &item);
        let max_price = price_config::base_price(&GoodType::Rum) * price_config::MAX_PRICE_MULTIPLIER;
        assert!((price - max_price).abs() < 0.1, "Zero stock should hit max price");
    }

    #[test]
    fn test_price_clamping() {
        // Very low supply
        let item_low = InventoryItem::new(1, 15.0);
        let price_low = calculate_supply_price(&GoodType::Rum, &item_low);
        let max = price_config::base_price(&GoodType::Rum) * price_config::MAX_PRICE_MULTIPLIER;
        assert!(price_low <= max + 0.1, "Price should be capped at max");

        // Very high supply
        let item_high = InventoryItem::new(10000, 15.0);
        let price_high = calculate_supply_price(&GoodType::Rum, &item_high);
        let min = price_config::base_price(&GoodType::Rum) * price_config::MIN_PRICE_MULTIPLIER;
        assert!(price_high >= min - 0.1, "Price should be floored at min");
    }

    #[test]
    fn test_high_demand_increases_price() {
        let base_qty = price_config::base_quantity(&GoodType::Rum);
        let item = InventoryItem::new(base_qty, 15.0);
        let high_demand = 1.5;
        let price = calculate_price(&GoodType::Rum, &item, high_demand);
        let base = price_config::base_price(&GoodType::Rum);
        assert!(price > base, "High demand should increase price");
    }

    #[test]
    fn test_low_demand_decreases_price() {
        let base_qty = price_config::base_quantity(&GoodType::Rum);
        let item = InventoryItem::new(base_qty, 15.0);
        let low_demand = 0.6;
        let price = calculate_price(&GoodType::Rum, &item, low_demand);
        let base = price_config::base_price(&GoodType::Rum);
        assert!(price < base, "Low demand should decrease price");
    }

    #[test]
    fn test_global_demand_methods() {
        let mut gd = GlobalDemand::default();
        assert!((gd.get(&GoodType::Rum) - 1.0).abs() < 0.01);
        
        gd.increase(GoodType::Rum, 0.3);
        assert!((gd.get(&GoodType::Rum) - 1.3).abs() < 0.01);
        
        gd.decrease(GoodType::Rum, 0.5);
        assert!((gd.get(&GoodType::Rum) - 0.8).abs() < 0.01);
        
        // Test clamping
        gd.set(GoodType::Rum, 5.0);
        assert!((gd.get(&GoodType::Rum) - 2.0).abs() < 0.01, "Should clamp to max 2.0");
    }
}
