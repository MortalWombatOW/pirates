//! Faction AI system for world simulation.
//!
//! Runs per world tick to simulate faction economic and military decisions.

use bevy::prelude::*;

use crate::resources::{FactionRegistry, WorldClock};
use crate::components::FactionId;

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

    // TODO(5.2.4): Generate new trade routes based on port availability
    // TODO(5.2.5): Spawn ships if gold permits and routes need fulfillment
    // TODO(5.2.6): Check for hostile player in territory, respond with ships
}
