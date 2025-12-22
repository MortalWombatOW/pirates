//! Meta-progression profile that persists across game runs.
//!
//! Stores player stats, unlocked archetypes, and legacy wrecks from previous deaths.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Persistent profile that tracks meta-progression across runs.
///
/// This resource is loaded from file on app start and saved on death/quit.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct MetaProfile {
    /// Player stats that affect gameplay systems.
    pub stats: PlayerStats,
    /// Unlocked starting archetypes.
    pub unlocked_archetypes: Vec<ArchetypeId>,
    /// Wrecks from previous runs, placed on new maps.
    pub legacy_wrecks: Vec<LegacyWreck>,
    /// Total gold earned across all runs.
    pub lifetime_gold: u64,
    /// Total ships captured across all runs.
    pub lifetime_captures: u32,
    /// Number of completed runs (reached GameOver with victory).
    pub runs_completed: u32,
    /// Number of deaths.
    pub deaths: u32,
}

impl Default for MetaProfile {
    fn default() -> Self {
        Self {
            stats: PlayerStats::default(),
            unlocked_archetypes: vec![ArchetypeId::Default],
            legacy_wrecks: Vec::new(),
            lifetime_gold: 0,
            lifetime_captures: 0,
            runs_completed: 0,
            deaths: 0,
        }
    }
}

/// Player stats that persist and grow across runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    /// Affects companion recruitment costs and faction reputation gains.
    pub charisma: u8,
    /// Affects sailing speed and pathfinding efficiency.
    pub navigation: u8,
    /// Affects cargo capacity and fleet management.
    pub logistics: u8,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            charisma: 1,
            navigation: 1,
            logistics: 1,
        }
    }
}

/// Starting archetype identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArchetypeId {
    /// Default starting character with balanced stats.
    Default,
    /// Former naval officer with military connections.
    RoyalNavyCaptain,
    /// Underworld connections and contraband expertise.
    Smuggler,
    /// Survivor starting with nothing but determination.
    Castaway,
}

/// A wreck from a previous run that spawns on the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyWreck {
    /// Tile position where the player died.
    pub position: IVec2,
    /// Gold the player had at death.
    pub gold: u32,
    /// Cargo the player had at death (serialized as type and quantity).
    pub cargo: Vec<(String, u32)>,
    /// Name of the ship that sank.
    pub ship_name: String,
    /// Run number when this wreck was created.
    pub run_number: u32,
}
