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
