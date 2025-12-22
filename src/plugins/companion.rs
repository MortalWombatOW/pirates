use bevy::prelude::*;
use crate::components::companion::{Companion, CompanionName, CompanionRole};
use crate::plugins::core::GameState;

use crate::components::ship::{Player, Ship};
use crate::components::cargo::Gold;
use crate::components::Cargo;
use crate::events::TradeExecutedEvent;
use rand::Rng;

pub struct CompanionPlugin;

impl Plugin for CompanionPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TavernCompanions>()
            .add_event::<CompanionRecruitedEvent>()
            .add_event::<AutoTradeEvent>()
            .add_systems(OnEnter(GameState::Port), generate_tavern_companions)
            .add_systems(OnExit(GameState::Port), clear_tavern_companions)
            .add_systems(Update, (
                companion_recruitment_system,
                auto_trade_system,
            ).run_if(in_state(GameState::Port)));
    }
}

/// Resource storing companions available for recruitment in the current port.
#[derive(Resource, Default)]
pub struct TavernCompanions {
    pub available: Vec<RecruitableCompanion>,
}

/// Data struct for a companion available in the tavern (not yet an entity).
#[derive(Clone, Debug)]
pub struct RecruitableCompanion {
    pub name: String,
    pub role: CompanionRole,
    pub cost: u32,
    /// Unique ID for UI tracking
    pub id: u64,
}

/// Event triggered when a companion is recruited.
#[derive(Event)]
pub struct CompanionRecruitedEvent {
    pub companion_id: u64,
}

/// Event triggered to execute an auto-trade.
#[derive(Event)]
pub struct AutoTradeEvent {
    pub port_entity: Entity,
}

/// Helper function to spawn a companion entity.
/// Returns the Entity ID.
pub fn spawn_companion(
    commands: &mut Commands,
    name: String,
    role: CompanionRole,
) -> Entity {
    commands.spawn((
        Companion,
        CompanionName(name),
        role,
    )).id()
}

/// System to generate random companions when entering a port.
fn generate_tavern_companions(
    mut tavern_comps: ResMut<TavernCompanions>,
) {
    let mut rng = rand::thread_rng();
    let num_companions = rng.gen_range(1..=3);
    
    let mut companions = Vec::new();
    
    for _i in 0..num_companions {
        let role = match rng.gen_range(0..5) {
            0 => CompanionRole::Quartermaster,
            1 => CompanionRole::Navigator,
            2 => CompanionRole::Lookout,
            3 => CompanionRole::Gunner,
            _ => CompanionRole::Mystic,
        };
        
        let name = generate_companion_name(&mut rng);
        let cost = calculate_recruitment_cost(role, &mut rng);
        
        companions.push(RecruitableCompanion {
            name,
            role,
            cost,
            id: rng.gen::<u64>(), // Simple random ID
        });
    }
    
    tavern_comps.available = companions;
    info!("Generated {} companions at tavern", tavern_comps.available.len());
}

/// Clears tavern companions when leaving port.
fn clear_tavern_companions(mut tavern_comps: ResMut<TavernCompanions>) {
    tavern_comps.available.clear();
}

/// System to handle recruitment events.
fn companion_recruitment_system(
    mut commands: Commands,
    mut events: EventReader<CompanionRecruitedEvent>,
    mut tavern_comps: ResMut<TavernCompanions>,
    mut player_query: Query<&mut Gold, (With<Player>, With<Ship>)>,
) {
    for event in events.read() {
        // Find the companion in the available list
        if let Some(index) = tavern_comps.available.iter().position(|c| c.id == event.companion_id) {
            let companion_data = &tavern_comps.available[index];
            
            // Check gold
            if let Ok(mut gold) = player_query.get_single_mut() {
                if gold.spend(companion_data.cost) {
                    // Spawn the entity
                    spawn_companion(
                        &mut commands,
                        companion_data.name.clone(),
                        companion_data.role,
                    );
                    
                    info!("Recruited companion: {} ({:?})", companion_data.name, companion_data.role);
                    
                    // Remove from tavern list
                    tavern_comps.available.remove(index);
                } else {
                    warn!("Failed to recruit companion: Insufficient gold");
                }
            } else {
                warn!("Failed to recruit companion: Player not found");
            }
        }
    }
}

fn calculate_recruitment_cost(role: CompanionRole, rng: &mut rand::rngs::ThreadRng) -> u32 {
    let base_cost = match role {
        CompanionRole::Quartermaster => 600,
        CompanionRole::Navigator => 500,
        CompanionRole::Lookout => 400,
        CompanionRole::Gunner => 550,
        CompanionRole::Mystic => 1000,
    };
    
    // Variance +/- 10%
    let variance = rng.gen_range(0.9..=1.1);
    (base_cost as f32 * variance) as u32
}

fn generate_companion_name(rng: &mut rand::rngs::ThreadRng) -> String {
    let first_names = [
        "Jack", "Anne", "Edward", "Mary", "William", "Grace", "Henry", "Elizabeth",
        "Bartholomew", "Sadie", "Charles", "Abigail", "Thomas", "Jane"
    ];
    let last_names = [
        "Teach", "Bonny", "Low", "Read", "Kidd", "O'Malley", "Morgan", "Swann",
        "Roberts", "Farrell", "Vane", "Sharpe", "Tew", "Drake"
    ];
    
    let first = first_names[rng.gen_range(0..first_names.len())];
    let last = last_names[rng.gen_range(0..last_names.len())];
    
    format!("{} {}", first, last)
}

/// System to execute auto-trades when requested by Quartermaster.
fn auto_trade_system(
    _commands: Commands,
    mut events: EventReader<AutoTradeEvent>,
    mut player_query: Query<(Entity, &mut Gold, &mut Cargo), (With<Player>, With<Ship>)>,
    port_query: Query<&crate::components::port::Inventory, With<crate::components::port::Port>>,
    mut trade_events: EventWriter<TradeExecutedEvent>,
) {
    for event in events.read() {
        if let Ok((player_entity, mut gold, mut cargo)) = player_query.get_single_mut() {
            if let Ok(inventory) = port_query.get(event.port_entity) {
                // Find a good to buy.
                // Pick a random good sold by the market.
                if !inventory.goods.is_empty() {
                     // Just pick the first one for now as a placeholder for "best deal"
                     if let Some(good_type) = inventory.goods.keys().next() {
                         if let Some(item) = inventory.get_good(good_type) {
                             let price = item.price;
                             // Buy as much as we can afford and carry
                             let space = cargo.capacity.saturating_sub(cargo.total_units());
                             let affordable = (gold.0 as f32 / price).floor() as u32;
                             let amount = space.min(affordable).min(item.quantity);
                             
                             if amount > 0 {
                                 // Execute trade
                                 // We send the event, which handles the actual deduction/addition in trade_execution_system
                                 // But wait, the Quartermaster logic here was trying to do it manually?
                                 // "gold.spend... cargo.add..."
                                 // If I send TradeExecutedEvent, the SYSTEM handles it.
                                 // So I should just SEND THE EVENT and not mutate gold/cargo here directly.
                                 // That avoids double transactions.
                                 
                                 trade_events.send(TradeExecutedEvent {
                                     port_entity: event.port_entity,
                                     good_type: *good_type,
                                     quantity: amount,
                                     is_buy: true,
                                 });
                                 
                                 info!("Quartermaster auto-traded: Requested buy of {} x {:?}", amount, good_type);
                             } else {
                                 info!("Quartermaster could not trade: No space, gold, or stock.");
                             }
                         }
                     }
                }
            }
        }
    }
}
