use bevy::prelude::*;
use std::collections::HashMap;

use super::cargo::GoodType;

/// Marker component that identifies an entity as a port.
/// Ports are docking locations where players can trade, repair, and recruit.
#[derive(Component, Debug, Default)]
pub struct Port;

/// The display name of a port.
#[derive(Component, Debug, Clone)]
pub struct PortName(pub String);

impl Default for PortName {
    fn default() -> Self {
        Self("Unknown Port".to_string())
    }
}

/// Represents a single item in a port's inventory.
#[derive(Debug, Clone)]
pub struct InventoryItem {
    /// Current quantity in stock.
    pub quantity: u32,
    /// Current price per unit.
    pub price: f32,
}

impl InventoryItem {
    pub fn new(quantity: u32, price: f32) -> Self {
        Self { quantity, price }
    }
}

/// Port inventory containing goods available for trade.
/// Each good has a quantity and price that can fluctuate.
#[derive(Component, Debug, Clone)]
pub struct Inventory {
    /// Map of goods to their quantity and price.
    pub goods: HashMap<GoodType, InventoryItem>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            goods: HashMap::new(),
        }
    }
}

impl Inventory {
    /// Creates a new empty inventory.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds or updates a good in the inventory.
    pub fn set_good(&mut self, good: GoodType, quantity: u32, price: f32) {
        self.goods.insert(good, InventoryItem::new(quantity, price));
    }

    /// Gets the quantity and price of a good, if present.
    pub fn get_good(&self, good: &GoodType) -> Option<&InventoryItem> {
        self.goods.get(good)
    }

    /// Attempts to buy goods from this inventory.
    /// Returns the actual quantity bought (may be less if stock is low).
    pub fn buy(&mut self, good: &GoodType, amount: u32) -> Option<(u32, f32)> {
        if let Some(item) = self.goods.get_mut(good) {
            let bought = amount.min(item.quantity);
            let cost = bought as f32 * item.price;
            item.quantity -= bought;
            Some((bought, cost))
        } else {
            None
        }
    }

    /// Sells goods to this inventory, adding to stock.
    /// Returns the revenue from the sale.
    pub fn sell(&mut self, good: GoodType, amount: u32, price_modifier: f32) -> f32 {
        let item = self.goods.entry(good).or_insert(InventoryItem::new(0, 10.0));
        item.quantity += amount;
        // Sell price is typically lower than buy price
        let revenue = amount as f32 * item.price * price_modifier;
        revenue
    }
}
