//! Faction AI system for world simulation.
//!
//! Runs per world tick to simulate faction economic and military decisions.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::resources::{FactionRegistry, WorldClock};
use crate::components::{FactionId, Faction, Port};

/// Runs faction simulation logic once per in-game hour.
/// 
/// This system triggers when WorldClock.tick == 0 (start of each hour):
/// - Evaluates faction economies
/// - Plans trade routes (task 5.2.4)
/// - Spawns ships to fulfill routes (task 5.2.5)
/// - Responds to threats (task 5.2.6)
pub fn faction_ai_system(
    world_clock: Res<WorldClock>,
    mut faction_registry: ResMut<FactionRegistry>,
) {
    // Run once per hour (when tick resets to 0)
    if world_clock.tick != 0 {
        return;
    }

    debug!(
        "FactionAI: Day {} Hour {} - Running faction simulation",
        world_clock.day, world_clock.hour
    );

    // Iterate all factions and run their AI
    for (faction_id, state) in faction_registry.factions.iter_mut() {
        run_faction_tick(faction_id, state, &world_clock);
    }
}

/// Processes a single faction's hourly tick.
fn run_faction_tick(
    faction_id: &FactionId,
    state: &mut crate::resources::faction::FactionState,
    world_clock: &WorldClock,
) {
    // Passive income: Factions earn gold based on trade route count
    let route_income = state.trade_routes.len() as u32 * 10;
    state.gold = state.gold.saturating_add(route_income);

    // Log faction state for debugging (only once per day at midnight)
    if world_clock.hour == 0 {
        info!(
            "Faction {:?}: gold={}, ships={}, reputation={}, routes={}",
            faction_id,
            state.gold,
            state.ships,
            state.player_reputation,
            state.trade_routes.len()
        );
    }

    // TODO(5.2.5): Spawn ships if gold permits and routes need fulfillment
    // TODO(5.2.6): Check for hostile player in territory, respond with ships
}

/// Maximum number of trade routes a faction can maintain.
const MAX_ROUTES_PER_FACTION: usize = 5;

/// Cost in gold to establish a new trade route.
const ROUTE_ESTABLISHMENT_COST: u32 = 500;

/// Generates trade routes between ports belonging to the same faction.
/// 
/// This system runs once per in-game day (at midnight) and:
/// - Queries all ports and groups them by faction
/// - For each faction with fewer than MAX_ROUTES, attempts to create new routes
/// - Routes are bidirectional pairs of port entities
pub fn trade_route_generation_system(
    world_clock: Res<WorldClock>,
    mut faction_registry: ResMut<FactionRegistry>,
    port_query: Query<(Entity, &Faction), With<Port>>,
) {
    // Run once per day at midnight
    if world_clock.tick != 0 || world_clock.hour != 0 {
        return;
    }

    // Group ports by faction
    let mut ports_by_faction: HashMap<FactionId, Vec<Entity>> = HashMap::new();
    for (entity, faction) in &port_query {
        ports_by_faction
            .entry(faction.0)
            .or_default()
            .push(entity);
    }

    // For each faction, try to generate routes between their ports
    for (faction_id, faction_ports) in &ports_by_faction {
        // Skip Pirates - they don't trade
        if *faction_id == FactionId::Pirates {
            continue;
        }

        let Some(state) = faction_registry.get_mut(*faction_id) else {
            continue;
        };

        // Skip if faction already has max routes
        if state.trade_routes.len() >= MAX_ROUTES_PER_FACTION {
            continue;
        }

        // Skip if faction can't afford a new route
        if state.gold < ROUTE_ESTABLISHMENT_COST {
            continue;
        }

        // Need at least 2 ports to create a route
        if faction_ports.len() < 2 {
            continue;
        }

        // Find a new valid route (not already existing)
        for i in 0..faction_ports.len() {
            for j in (i + 1)..faction_ports.len() {
                let port_a = faction_ports[i];
                let port_b = faction_ports[j];

                // Check if this route already exists (in either direction)
                let route_exists = state.trade_routes.iter().any(|(a, b)| {
                    (*a == port_a && *b == port_b) || (*a == port_b && *b == port_a)
                });

                if !route_exists {
                    // Create the route
                    state.trade_routes.push((port_a, port_b));
                    state.gold = state.gold.saturating_sub(ROUTE_ESTABLISHMENT_COST);

                    info!(
                        "Faction {:?} established new trade route (total: {})",
                        faction_id,
                        state.trade_routes.len()
                    );

                    // Only create one route per day per faction
                    break;
                }
            }
            // Break outer loop if we created a route (check route count change)
            if state.trade_routes.len() >= MAX_ROUTES_PER_FACTION {
                break;
            }
        }
    }
}
