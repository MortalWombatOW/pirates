use bevy::prelude::*;

use crate::components::{
    cargo::GoodType,
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
}

/// System that recalculates prices for all ports based on supply.
/// 
/// Runs every world tick (via FixedUpdate).
/// 
/// **Price Formula:**
/// ```text
/// price = base_price * supply_multiplier
/// 
/// supply_ratio = current_quantity / base_quantity
/// supply_multiplier = clamp(1.0 / supply_ratio^sensitivity, min, max)
/// ```
/// 
/// Low stock → higher prices, high stock → lower prices.
pub fn price_calculation_system(
    mut port_query: Query<&mut Inventory, With<Port>>,
) {
    for mut inventory in port_query.iter_mut() {
        for (good_type, item) in inventory.goods.iter_mut() {
            let new_price = calculate_supply_price(good_type, item);
            item.price = new_price;
        }
    }
}

/// Calculates price based on supply (low stock = higher price).
fn calculate_supply_price(good_type: &GoodType, item: &InventoryItem) -> f32 {
    let base_price = price_config::base_price(good_type);
    let base_quantity = price_config::base_quantity(good_type) as f32;
    
    // Handle edge case of zero quantity
    if item.quantity == 0 {
        return base_price * price_config::MAX_PRICE_MULTIPLIER;
    }
    
    let supply_ratio = item.quantity as f32 / base_quantity;
    
    // Inverse relationship: low supply = high multiplier
    let raw_multiplier = supply_ratio.powf(-price_config::SUPPLY_SENSITIVITY);
    
    let clamped_multiplier = raw_multiplier.clamp(
        price_config::MIN_PRICE_MULTIPLIER,
        price_config::MAX_PRICE_MULTIPLIER,
    );
    
    base_price * clamped_multiplier
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
}
