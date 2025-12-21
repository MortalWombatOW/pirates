//! Faction simulation resources for the world economy.
//!
//! Contains `FactionRegistry` which tracks the state of all factions in the game world.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::FactionId;

/// State for a single faction in the world simulation.
/// Tracks economic and military capabilities.
#[derive(Debug, Clone, Reflect)]
pub struct FactionState {
    /// Faction's treasury.
    pub gold: u32,
    /// Number of ships owned by this faction.
    pub ships: u32,
    /// Reputation with the player (-100 to 100).
    pub player_reputation: i32,
    /// Trade routes managed by this faction (origin port Entity, destination port Entity).
    pub trade_routes: Vec<(Entity, Entity)>,
}

impl Default for FactionState {
    fn default() -> Self {
        Self {
            gold: 10_000,
            ships: 10,
            player_reputation: 0,
            trade_routes: Vec::new(),
        }
    }
}

/// Global registry of all faction states.
/// Keyed by `FactionId`.
#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct FactionRegistry {
    pub factions: HashMap<FactionId, FactionState>,
}

impl FactionRegistry {
    /// Creates a new registry with default states for all known factions.
    pub fn new() -> Self {
        let mut factions = HashMap::new();
        factions.insert(FactionId::Pirates, FactionState {
            gold: 5_000,
            ships: 20,
            player_reputation: -100, // Always hostile
            ..Default::default()
        });
        factions.insert(FactionId::NationA, FactionState::default());
        factions.insert(FactionId::NationB, FactionState::default());
        factions.insert(FactionId::NationC, FactionState::default());
        Self { factions }
    }

    /// Gets an immutable reference to a faction's state.
    pub fn get(&self, faction: FactionId) -> Option<&FactionState> {
        self.factions.get(&faction)
    }

    /// Gets a mutable reference to a faction's state.
    pub fn get_mut(&mut self, faction: FactionId) -> Option<&mut FactionState> {
        self.factions.get_mut(&faction)
    }

    /// Returns true if the faction is hostile to the player.
    pub fn is_hostile(&self, faction: FactionId) -> bool {
        self.factions
            .get(&faction)
            .map_or(false, |state| state.player_reputation < -50)
    }
}
