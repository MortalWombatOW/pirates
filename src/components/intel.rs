use bevy::prelude::*;

/// Marker component identifying an entity as intel.
#[derive(Component, Debug, Default)]
pub struct Intel;

/// Types of intel available in the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
pub enum IntelType {
    /// Reveals a ship's trade route between ports.
    #[default]
    ShipRoute,
    /// Reveals a port's current inventory and prices.
    PortInventory,
    /// Reveals location of hidden treasure or loot.
    TreasureLocation,
    /// General rumor - may lead to other intel.
    Rumor,
    /// Reveals the location of a hostile fleet.
    FleetPosition,
    /// Reveals an area of the map (removes fog of war).
    MapReveal,
}

impl IntelType {
    /// Returns a human-readable description of the intel type.
    pub fn description(&self) -> &'static str {
        match self {
            IntelType::ShipRoute => "Ship Route",
            IntelType::PortInventory => "Port Inventory",
            IntelType::TreasureLocation => "Treasure Location",
            IntelType::Rumor => "Rumor",
            IntelType::FleetPosition => "Fleet Position",
            IntelType::MapReveal => "Map Information",
        }
    }
}

/// Data payload for intel, varies by type.
#[derive(Component, Debug, Clone)]
pub struct IntelData {
    /// Type of intel this data represents.
    pub intel_type: IntelType,
    /// Source port where this intel was acquired (if any).
    pub source_port: Option<Entity>,
    /// Target entity this intel relates to (ship, port, etc).
    pub target_entity: Option<Entity>,
    /// Map positions revealed by this intel (for MapReveal, TreasureLocation).
    pub revealed_positions: Vec<IVec2>,
    /// Route waypoints (for ShipRoute intel).
    pub route_waypoints: Vec<IVec2>,
    /// Human-readable description.
    pub description: String,
    /// Gold cost to purchase this intel from a tavern.
    pub purchase_cost: u32,
}

impl Default for IntelData {
    fn default() -> Self {
        Self {
            intel_type: IntelType::default(),
            source_port: None,
            target_entity: None,
            revealed_positions: Vec::new(),
            route_waypoints: Vec::new(),
            description: String::new(),
            purchase_cost: 0,
        }
    }
}

impl IntelData {
    /// Creates a new ShipRoute intel.
    pub fn ship_route(
        source: Entity,
        target_ship: Entity,
        waypoints: Vec<IVec2>,
        description: String,
        cost: u32,
    ) -> Self {
        Self {
            intel_type: IntelType::ShipRoute,
            source_port: Some(source),
            target_entity: Some(target_ship),
            revealed_positions: Vec::new(),
            route_waypoints: waypoints,
            description,
            purchase_cost: cost,
        }
    }

    /// Creates a new PortInventory intel.
    pub fn port_inventory(source: Entity, target_port: Entity, description: String, cost: u32) -> Self {
        Self {
            intel_type: IntelType::PortInventory,
            source_port: Some(source),
            target_entity: Some(target_port),
            revealed_positions: Vec::new(),
            route_waypoints: Vec::new(),
            description,
            purchase_cost: cost,
        }
    }

    /// Creates a new TreasureLocation intel.
    pub fn treasure_location(source: Entity, position: IVec2, description: String, cost: u32) -> Self {
        Self {
            intel_type: IntelType::TreasureLocation,
            source_port: Some(source),
            target_entity: None,
            revealed_positions: vec![position],
            route_waypoints: Vec::new(),
            description,
            purchase_cost: cost,
        }
    }

    /// Creates a new Rumor intel.
    pub fn rumor(source: Entity, description: String, cost: u32) -> Self {
        Self {
            intel_type: IntelType::Rumor,
            source_port: Some(source),
            target_entity: None,
            revealed_positions: Vec::new(),
            route_waypoints: Vec::new(),
            description,
            purchase_cost: cost,
        }
    }

    /// Creates a new MapReveal intel.
    pub fn map_reveal(source: Entity, positions: Vec<IVec2>, description: String, cost: u32) -> Self {
        Self {
            intel_type: IntelType::MapReveal,
            source_port: Some(source),
            target_entity: None,
            revealed_positions: positions,
            route_waypoints: Vec::new(),
            description,
            purchase_cost: cost,
        }
    }

    /// Creates a new FleetPosition intel.
    pub fn fleet_position(
        source: Entity,
        target_fleet: Entity,
        position: IVec2,
        description: String,
        cost: u32,
    ) -> Self {
        Self {
            intel_type: IntelType::FleetPosition,
            source_port: Some(source),
            target_entity: Some(target_fleet),
            revealed_positions: vec![position],
            route_waypoints: Vec::new(),
            description,
            purchase_cost: cost,
        }
    }
}

/// Component tracking intel expiry based on world time.
/// Intel with this component is transient and will be removed after TTL.
#[derive(Component, Debug, Clone)]
pub struct IntelExpiry {
    /// World tick at which this intel expires.
    pub expiry_tick: u32,
}

impl IntelExpiry {
    /// Default intel duration in ticks (1 in-game day = 24 * 60 ticks).
    pub const DEFAULT_DURATION_TICKS: u32 = 24 * 60;

    /// Creates a new expiry with the default duration from current tick.
    pub fn new(current_tick: u32) -> Self {
        Self {
            expiry_tick: current_tick + Self::DEFAULT_DURATION_TICKS,
        }
    }

    /// Creates a new expiry with a custom duration in ticks.
    pub fn with_duration(current_tick: u32, duration_ticks: u32) -> Self {
        Self {
            expiry_tick: current_tick + duration_ticks,
        }
    }

    /// Returns true if this intel has expired.
    pub fn is_expired(&self, current_tick: u32) -> bool {
        current_tick >= self.expiry_tick
    }
}

/// Marker component for intel that has been acquired by the player.
#[derive(Component, Debug, Default)]
pub struct AcquiredIntel;

/// Marker component for intel available for purchase at a tavern.
#[derive(Component, Debug, Default)]
pub struct TavernIntel;
