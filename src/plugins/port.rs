use bevy::prelude::*;
use rand::Rng;

use crate::components::{
    cargo::GoodType,
    port::{Inventory, Port, PortName},
    ship::Faction,
};

/// Plugin for managing port entities and their interactions.
pub struct PortPlugin;

impl Plugin for PortPlugin {
    fn build(&self, _app: &mut App) {
        // Systems will be added when Port UI is implemented in Epic 4.2
        info!("PortPlugin initialized");
    }
}

/// Spawns a port entity at the specified world position.
/// 
/// # Arguments
/// * `commands` - Bevy Commands for entity spawning
/// * `world_position` - Position in world coordinates
/// * `name` - Display name of the port
/// * `faction` - The faction controlling this port
/// 
/// Returns the spawned port entity.
pub fn spawn_port(
    commands: &mut Commands,
    world_position: Vec2,
    name: String,
    faction: Faction,
) -> Entity {
    let inventory = generate_random_inventory();
    
    let entity = commands.spawn((
        Port,
        PortName(name.clone()),
        faction,
        inventory,
        Transform::from_xyz(world_position.x, world_position.y, 0.0),
    )).id();
    
    info!("Spawned port '{}' at ({}, {})", name, world_position.x, world_position.y);
    
    entity
}

/// Generates a random starting inventory for a port.
/// Each port has a randomized selection of goods with varied quantities and prices.
pub fn generate_random_inventory() -> Inventory {
    let mut rng = rand::thread_rng();
    let mut inventory = Inventory::new();
    
    // Base prices for each good type
    let goods_config = [
        (GoodType::Rum, 15.0, 50, 150),      // (type, base_price, min_qty, max_qty)
        (GoodType::Sugar, 8.0, 80, 200),
        (GoodType::Spices, 25.0, 20, 80),
        (GoodType::Timber, 5.0, 100, 300),
        (GoodType::Cloth, 12.0, 40, 120),
        (GoodType::Weapons, 40.0, 10, 50),
    ];
    
    // Each port has 3-5 goods initially available
    let num_goods = rng.gen_range(3..=5);
    let mut available_goods: Vec<_> = goods_config.iter().collect();
    
    for _ in 0..num_goods {
        if available_goods.is_empty() {
            break;
        }
        
        let idx = rng.gen_range(0..available_goods.len());
        let (good, base_price, min_qty, max_qty) = available_goods.remove(idx);
        
        // Randomize quantity within range
        let quantity = rng.gen_range(*min_qty..=*max_qty);
        
        // Randomize price within ±30% of base
        let price_variance = rng.gen_range(0.7..1.3);
        let price = base_price * price_variance;
        
        inventory.set_good(*good, quantity, price);
    }
    
    inventory
}

/// Port name generator - creates thematic pirate-era port names.
pub fn generate_port_name() -> String {
    let mut rng = rand::thread_rng();
    
    let prefixes = [
        "Port", "Nueva", "San", "Fort", "Cape", "Old", "Black",
    ];
    
    let names = [
        "Royal", "Isabella", "Diego", "Nassau", "Tortuga", "Havana",
        "Kingston", "Maracaibo", "Trinidad", "Barbados", "Santiago",
    ];
    
    let suffixes = [
        "", " Bay", " Harbor", " Cove", " Point",
    ];
    
    format!(
        "{} {}{}",
        prefixes[rng.gen_range(0..prefixes.len())],
        names[rng.gen_range(0..names.len())],
        suffixes[rng.gen_range(0..suffixes.len())]
    )
}
