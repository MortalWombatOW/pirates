use bevy::prelude::*;
use std::collections::HashMap;

/// Types of goods that can be traded in the game.
/// Each good has different economic properties (see `GoodsTrait`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum GoodType {
    #[default]
    Rum,
    Sugar,
    Spices,
    Timber,
    Cloth,
    Weapons,
    // Future goods can be added here
}

/// Traits that modify how goods behave in the economy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoodsTrait {
    /// Loses value over time.
    Perishable,
    /// Takes more cargo space.
    Heavy,
    /// Illegal in some ports, higher risk/reward.
    Illegal,
}

impl GoodType {
    /// Returns the traits associated with this good type.
    pub fn traits(&self) -> Vec<GoodsTrait> {
        match self {
            GoodType::Rum => vec![GoodsTrait::Perishable],
            GoodType::Sugar => vec![GoodsTrait::Perishable],
            GoodType::Spices => vec![],
            GoodType::Timber => vec![GoodsTrait::Heavy],
            GoodType::Cloth => vec![],
            GoodType::Weapons => vec![GoodsTrait::Illegal, GoodsTrait::Heavy],
        }
    }
}

/// Represents the cargo hold of a ship.
/// Contains goods and tracks capacity limits.
#[derive(Component, Debug, Clone)]
pub struct Cargo {
    /// Map of goods to quantities currently held.
    pub goods: HashMap<GoodType, u32>,
    /// Maximum cargo capacity (total units all goods combined).
    pub capacity: u32,
}

impl Cargo {
    /// Creates a new empty cargo hold with the specified capacity.
    pub fn new(capacity: u32) -> Self {
        Self {
            goods: HashMap::new(),
            capacity,
        }
    }

    /// Returns the total number of units currently in the cargo hold.
    pub fn total_units(&self) -> u32 {
        self.goods.values().sum()
    }

    /// Returns the remaining available capacity.
    pub fn available_capacity(&self) -> u32 {
        self.capacity.saturating_sub(self.total_units())
    }

    /// Returns true if the cargo is at full capacity.
    pub fn is_full(&self) -> bool {
        self.available_capacity() == 0
    }

    /// Attempts to add goods to the cargo. Returns how many were actually added.
    pub fn add(&mut self, good: GoodType, amount: u32) -> u32 {
        let available = self.available_capacity();
        let to_add = amount.min(available);
        
        if to_add > 0 {
            *self.goods.entry(good).or_insert(0) += to_add;
        }
        
        to_add
    }

    /// Attempts to remove goods from the cargo. Returns how many were actually removed.
    pub fn remove(&mut self, good: GoodType, amount: u32) -> u32 {
        if let Some(current) = self.goods.get_mut(&good) {
            let to_remove = amount.min(*current);
            *current -= to_remove;
            
            if *current == 0 {
                self.goods.remove(&good);
            }
            
            to_remove
        } else {
            0
        }
    }

    /// Returns the quantity of a specific good type.
    pub fn get(&self, good: GoodType) -> u32 {
        *self.goods.get(&good).unwrap_or(&0)
    }
}

impl Default for Cargo {
    fn default() -> Self {
        Self::new(100)
    }
}

/// Represents the gold (currency) held by an entity.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Gold(pub u32);

impl Gold {
    /// Attempts to spend the specified amount. Returns true if successful.
    pub fn spend(&mut self, amount: u32) -> bool {
        if self.0 >= amount {
            self.0 -= amount;
            true
        } else {
            false
        }
    }

    /// Adds the specified amount to the gold total.
    pub fn add(&mut self, amount: u32) {
        self.0 = self.0.saturating_add(amount);
    }
}
