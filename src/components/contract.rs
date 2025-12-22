use bevy::prelude::*;

use super::cargo::GoodType;

/// Marker component identifying an entity as a contract.
#[derive(Component, Debug, Default)]
pub struct Contract;

/// Types of contracts available in the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ContractType {
    /// Deliver specific goods from origin port to destination port.
    #[default]
    Transport,
    /// Visit a specific area or tile on the map.
    Explore,
    /// Protect a ship along a route (future implementation).
    Escort,
    /// Hunt down and destroy a specific enemy ship.
    Hunt,
}

impl ContractType {
    /// Returns a human-readable description of the contract type.
    pub fn description(&self) -> &'static str {
        match self {
            ContractType::Transport => "Deliver cargo",
            ContractType::Explore => "Explore area",
            ContractType::Escort => "Escort ship",
            ContractType::Hunt => "Hunt target",
        }
    }
}

/// Details of a specific contract.
#[derive(Component, Debug, Clone)]
pub struct ContractDetails {
    /// Type of contract.
    pub contract_type: ContractType,
    /// Port where the contract was offered.
    pub origin_port: Entity,
    /// Target destination (port entity for Transport, or area center for Explore).
    pub destination: Option<Entity>,
    /// Gold reward upon completion.
    pub reward_gold: u32,
    /// Cargo requirement for Transport contracts: (good type, quantity).
    pub cargo_required: Option<(GoodType, u32)>,
    /// Human-readable description of the contract.
    pub description: String,
    /// World tick at which this contract expires (None = never expires).
    pub expiry_tick: Option<u32>,
}

impl ContractDetails {
    /// Default contract duration in ticks (2 in-game days = 2 * 24 * 60 ticks).
    pub const DEFAULT_DURATION_TICKS: u32 = 2 * 24 * 60;

    /// Creates a new Transport contract.
    pub fn transport(
        origin: Entity,
        destination: Entity,
        good: GoodType,
        quantity: u32,
        reward: u32,
    ) -> Self {
        Self {
            contract_type: ContractType::Transport,
            origin_port: origin,
            destination: Some(destination),
            reward_gold: reward,
            cargo_required: Some((good, quantity)),
            description: format!("Deliver {} {:?} to destination", quantity, good),
            expiry_tick: None, // Set by system when created with WorldClock
        }
    }

    /// Creates a new Transport contract with expiry.
    pub fn transport_with_expiry(
        origin: Entity,
        destination: Entity,
        good: GoodType,
        quantity: u32,
        reward: u32,
        current_tick: u32,
    ) -> Self {
        let mut contract = Self::transport(origin, destination, good, quantity, reward);
        contract.expiry_tick = Some(current_tick + Self::DEFAULT_DURATION_TICKS);
        contract
    }

    /// Creates a new Explore contract (simplified - just visit a port).
    pub fn explore(origin: Entity, target: Entity, reward: u32) -> Self {
        Self {
            contract_type: ContractType::Explore,
            origin_port: origin,
            destination: Some(target),
            reward_gold: reward,
            cargo_required: None,
            description: "Visit the marked location".to_string(),
            expiry_tick: None, // Set by system when created with WorldClock
        }
    }

    /// Creates a new Explore contract with expiry.
    pub fn explore_with_expiry(
        origin: Entity,
        target: Entity,
        reward: u32,
        current_tick: u32,
    ) -> Self {
        let mut contract = Self::explore(origin, target, reward);
        contract.expiry_tick = Some(current_tick + Self::DEFAULT_DURATION_TICKS);
        contract
    }

    /// Returns true if this contract has expired.
    pub fn is_expired(&self, current_tick: u32) -> bool {
        if let Some(expiry) = self.expiry_tick {
            current_tick >= expiry
        } else {
            false // No expiry = never expires
        }
    }
}

/// Component marking a contract as accepted by the player.
#[derive(Component, Debug, Default)]
pub struct AcceptedContract;

/// Component for tracking contract progress.
#[derive(Component, Debug, Clone)]
pub struct ContractProgress {
    /// For Transport: cargo delivered so far.
    pub cargo_delivered: u32,
    /// Whether the destination has been reached.
    pub destination_reached: bool,
}

impl Default for ContractProgress {
    fn default() -> Self {
        Self {
            cargo_delivered: 0,
            destination_reached: false,
        }
    }
}

/// Component marking a contract as delegated to a fleet ship.
/// The assigned ship will autonomously fulfill the contract.
#[derive(Component, Debug)]
pub struct AssignedShip {
    /// The fleet ship entity assigned to this contract.
    pub ship_entity: Entity,
    /// Player receives this percentage of the reward (0.0 to 1.0).
    /// Remaining percentage represents "fleet overhead".
    pub player_cut: f32,
}

impl AssignedShip {
    /// Default player cut for delegated contracts (70%).
    pub const DEFAULT_CUT: f32 = 0.7;

    /// Creates a new assignment with the default player cut.
    pub fn new(ship_entity: Entity) -> Self {
        Self {
            ship_entity,
            player_cut: Self::DEFAULT_CUT,
        }
    }
}
