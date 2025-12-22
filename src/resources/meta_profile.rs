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

impl MetaProfile {
    /// Recalculates player stats based on lifetime milestones.
    ///
    /// Stat gains are milestone-based:
    /// - Charisma: Increases with completed runs (1 per 2 runs, max +4)
    /// - Navigation: Increases with lifetime gold (1 per 5000 gold, max +4)
    /// - Logistics: Increases with lifetime captures (1 per 3 ships, max +4)
    ///
    /// All stats cap at level 5 (base 1 + 4 from milestones).
    pub fn recalculate_stats(&mut self) {
        // Charisma: social skill from completing runs
        let charisma_bonus = (self.runs_completed / 2).min(4) as u8;
        self.stats.charisma = 1 + charisma_bonus;

        // Navigation: sailing skill from earning gold (exploration)
        let navigation_bonus = ((self.lifetime_gold / 5000) as u32).min(4) as u8;
        self.stats.navigation = 1 + navigation_bonus;

        // Logistics: fleet management from capturing ships
        let logistics_bonus = (self.lifetime_captures / 3).min(4) as u8;
        self.stats.logistics = 1 + logistics_bonus;
    }

    /// Adds gold to lifetime total and triggers stat recalculation.
    pub fn add_lifetime_gold(&mut self, amount: u64) {
        self.lifetime_gold += amount;
        self.recalculate_stats();
    }

    /// Increments capture count and triggers stat recalculation.
    pub fn add_capture(&mut self) {
        self.lifetime_captures += 1;
        self.recalculate_stats();
    }

    /// Increments completed runs and triggers stat recalculation.
    pub fn complete_run(&mut self) {
        self.runs_completed += 1;
        self.recalculate_stats();
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

impl PlayerStats {
    /// Returns the companion recruitment cost multiplier (lower is better).
    /// At level 1: 1.0, at level 10: 0.55 (45% discount).
    pub fn companion_cost_multiplier(&self) -> f32 {
        1.0 - (self.charisma as f32 - 1.0) * 0.05
    }

    /// Returns the faction reputation gain multiplier.
    /// At level 1: 1.0, at level 10: 1.45 (45% bonus).
    pub fn reputation_gain_multiplier(&self) -> f32 {
        1.0 + (self.charisma as f32 - 1.0) * 0.05
    }

    /// Returns the sailing speed multiplier.
    /// At level 1: 1.0, at level 10: 1.27 (27% bonus).
    pub fn sailing_speed_multiplier(&self) -> f32 {
        1.0 + (self.navigation as f32 - 1.0) * 0.03
    }

    /// Returns the cargo capacity bonus (flat addition).
    /// At level 1: 0, at level 10: 45 extra cargo slots.
    pub fn cargo_capacity_bonus(&self) -> u32 {
        (self.logistics as u32 - 1) * 5
    }

    /// Returns the maximum fleet size allowed.
    /// At level 1: 1, at level 10: 10 ships.
    pub fn max_fleet_size(&self) -> usize {
        self.logistics as usize
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

impl ArchetypeId {
    /// Returns all archetype variants for iteration.
    pub fn all() -> &'static [ArchetypeId] {
        &[
            ArchetypeId::Default,
            ArchetypeId::RoyalNavyCaptain,
            ArchetypeId::Smuggler,
            ArchetypeId::Castaway,
        ]
    }
}

use crate::components::ship::{FactionId, ShipType};
use std::collections::HashMap;

/// Configuration defining starting bonuses for an archetype.
#[derive(Debug, Clone)]
pub struct ArchetypeConfig {
    /// Display name shown in UI.
    pub name: &'static str,
    /// Short description of the archetype's playstyle.
    pub description: &'static str,
    /// Starting gold amount.
    pub starting_gold: u32,
    /// Starting ship type.
    pub ship_type: ShipType,
    /// Faction reputation modifiers (added to base 0).
    pub faction_reputation: HashMap<FactionId, i32>,
    /// Condition required to unlock this archetype.
    pub unlock_condition: UnlockCondition,
}

/// Conditions that unlock archetypes based on lifetime stats.
#[derive(Debug, Clone)]
pub enum UnlockCondition {
    /// Always available from the start.
    AlwaysUnlocked,
    /// Requires completing N runs.
    RunsCompleted(u32),
    /// Requires earning N lifetime gold.
    LifetimeGold(u64),
    /// Requires dying within N in-game hours of starting.
    QuickDeath(u32),
}

/// Global registry mapping archetype IDs to their configurations.
#[derive(Resource, Debug)]
pub struct ArchetypeRegistry {
    configs: HashMap<ArchetypeId, ArchetypeConfig>,
}

impl Default for ArchetypeRegistry {
    fn default() -> Self {
        let mut configs = HashMap::new();

        // Default: Balanced starting point
        configs.insert(
            ArchetypeId::Default,
            ArchetypeConfig {
                name: "Freebooter",
                description: "A balanced start for any aspiring captain.",
                starting_gold: 500,
                ship_type: ShipType::Sloop,
                faction_reputation: HashMap::new(),
                unlock_condition: UnlockCondition::AlwaysUnlocked,
            },
        );

        // Royal Navy Captain: Military background
        let mut navy_rep = HashMap::new();
        navy_rep.insert(FactionId::NationA, 50); // Crown favor
        navy_rep.insert(FactionId::Pirates, -50); // Pirate enmity
        configs.insert(
            ArchetypeId::RoyalNavyCaptain,
            ArchetypeConfig {
                name: "Royal Navy Captain",
                description: "A disgraced officer seeking fortune on the high seas.",
                starting_gold: 1000,
                ship_type: ShipType::Frigate,
                faction_reputation: navy_rep,
                unlock_condition: UnlockCondition::RunsCompleted(5),
            },
        );

        // Smuggler: Criminal connections
        let mut smuggler_rep = HashMap::new();
        smuggler_rep.insert(FactionId::NationB, 25); // Merchant contacts
        configs.insert(
            ArchetypeId::Smuggler,
            ArchetypeConfig {
                name: "Smuggler",
                description: "Fast ship, light pockets, and underworld connections.",
                starting_gold: 300,
                ship_type: ShipType::Schooner,
                faction_reputation: smuggler_rep,
                unlock_condition: UnlockCondition::LifetimeGold(10_000),
            },
        );

        // Castaway: Hard mode survivor
        let mut castaway_rep = HashMap::new();
        castaway_rep.insert(FactionId::NationA, -25);
        castaway_rep.insert(FactionId::NationB, -25);
        castaway_rep.insert(FactionId::NationC, -25);
        configs.insert(
            ArchetypeId::Castaway,
            ArchetypeConfig {
                name: "Castaway",
                description: "Washed ashore with nothing. Prove your worth.",
                starting_gold: 0,
                ship_type: ShipType::Raft,
                faction_reputation: castaway_rep,
                unlock_condition: UnlockCondition::QuickDeath(24), // Die within 1 in-game day
            },
        );

        Self { configs }
    }
}

impl ArchetypeRegistry {
    /// Retrieves the configuration for an archetype.
    pub fn get(&self, id: ArchetypeId) -> Option<&ArchetypeConfig> {
        self.configs.get(&id)
    }

    /// Checks if an archetype is unlocked based on player profile stats.
    pub fn is_unlocked(&self, id: ArchetypeId, profile: &MetaProfile) -> bool {
        let Some(config) = self.get(id) else {
            return false;
        };

        match &config.unlock_condition {
            UnlockCondition::AlwaysUnlocked => true,
            UnlockCondition::RunsCompleted(n) => profile.runs_completed >= *n,
            UnlockCondition::LifetimeGold(n) => profile.lifetime_gold >= *n,
            UnlockCondition::QuickDeath(_) => {
                // Tracked separately via death events; check unlocked_archetypes
                profile.unlocked_archetypes.contains(&id)
            }
        }
    }
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

/// Transient resource capturing player state at death for legacy wreck creation.
/// Populated by `ship_destruction_system`, consumed by `save_profile_on_death`.
#[derive(Resource, Default, Debug)]
pub struct PlayerDeathData {
    /// Position where the player died (world coordinates).
    pub position: Option<bevy::math::Vec2>,
    /// Gold the player had at death.
    pub gold: u32,
    /// Cargo the player had at death.
    pub cargo: Vec<(String, u32)>,
    /// Name of the player's ship.
    pub ship_name: String,
}

impl PlayerDeathData {
    /// Converts death data into a legacy wreck for the given run number.
    pub fn to_legacy_wreck(&self, run_number: u32, tile_size: f32) -> Option<LegacyWreck> {
        let pos = self.position?;
        // Convert world position to tile coordinates
        let tile_pos = bevy::math::IVec2::new(
            (pos.x / tile_size).round() as i32,
            (pos.y / tile_size).round() as i32,
        );
        Some(LegacyWreck {
            position: tile_pos,
            gold: self.gold,
            cargo: self.cargo.clone(),
            ship_name: self.ship_name.clone(),
            run_number,
        })
    }

    /// Clears the death data after it's been consumed.
    pub fn clear(&mut self) {
        self.position = None;
        self.gold = 0;
        self.cargo.clear();
        self.ship_name.clear();
    }
}

/// Default file name for profile storage.
const PROFILE_FILE_NAME: &str = "profile.json";

impl MetaProfile {
    /// Loads the profile from the default save location, or returns a fresh default profile.
    ///
    /// Save location is platform-specific:
    /// - macOS: ~/Library/Application Support/pirates/
    /// - Linux: ~/.local/share/pirates/
    /// - Windows: %APPDATA%/pirates/
    pub fn load_from_file() -> Self {
        let Some(path) = Self::get_save_path() else {
            warn!("Could not determine save directory, using default profile");
            return Self::default();
        };

        if !path.exists() {
            info!("No existing profile found, creating fresh profile");
            return Self::default();
        }

        match std::fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(profile) => {
                    info!("Loaded profile from {:?}", path);
                    profile
                }
                Err(e) => {
                    error!("Failed to parse profile file: {}", e);
                    Self::default()
                }
            },
            Err(e) => {
                error!("Failed to read profile file: {}", e);
                Self::default()
            }
        }
    }

    /// Returns the platform-specific path for the profile save file.
    pub fn get_save_path() -> Option<std::path::PathBuf> {
        dirs::data_dir().map(|mut path| {
            path.push("pirates");
            path.push(PROFILE_FILE_NAME);
            path
        })
    }

    /// Returns the platform-specific directory for profile storage.
    pub fn get_save_dir() -> Option<std::path::PathBuf> {
        dirs::data_dir().map(|mut path| {
            path.push("pirates");
            path
        })
    }

    /// Saves the profile to the default save location.
    ///
    /// Creates the save directory if it doesn't exist.
    pub fn save_to_file(&self) -> Result<(), String> {
        let Some(path) = Self::get_save_path() else {
            return Err("Could not determine save directory".to_string());
        };

        // Ensure save directory exists
        if let Some(dir) = Self::get_save_dir() {
            if !dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&dir) {
                    return Err(format!("Failed to create save directory: {}", e));
                }
                info!("Created save directory: {:?}", dir);
            }
        }

        // Serialize and write
        match serde_json::to_string_pretty(self) {
            Ok(json) => match std::fs::write(&path, json) {
                Ok(()) => {
                    info!("Saved profile to {:?}", path);
                    Ok(())
                }
                Err(e) => Err(format!("Failed to write profile file: {}", e)),
            },
            Err(e) => Err(format!("Failed to serialize profile: {}", e)),
        }
    }
}
