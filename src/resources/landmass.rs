//! Landmass-based navigation system for polygon pathfinding.
//!
//! Uses bevy_landmass for velocity-based steering navigation.
//! Maintains 3 archipelagos for different ship size tiers.

use bevy::prelude::*;
use bevy_landmass::prelude::*;

use crate::components::ship::ShipType;

/// Shore buffer distances for different ship sizes (in world units).
/// Larger ships need more clearance from coastlines.
pub const SHORE_BUFFER_SMALL: f32 = 32.0;   // Sloop, Raft
pub const SHORE_BUFFER_MEDIUM: f32 = 64.0;  // Schooner
pub const SHORE_BUFFER_LARGE: f32 = 96.0;   // Frigate

/// Agent radius for landmass navigation (half the shore buffer).
pub const AGENT_RADIUS_SMALL: f32 = 16.0;
pub const AGENT_RADIUS_MEDIUM: f32 = 32.0;
pub const AGENT_RADIUS_LARGE: f32 = 48.0;

/// Tier classification for shore buffer distances.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ShoreBufferTier {
    /// Small ships: minimal clearance (32 units)
    #[default]
    Small,
    /// Medium ships: moderate clearance (64 units)
    Medium,
    /// Large ships: maximum clearance (96 units)
    Large,
}

impl ShoreBufferTier {
    /// Returns the shore buffer distance for this tier.
    pub fn buffer_distance(&self) -> f32 {
        match self {
            ShoreBufferTier::Small => SHORE_BUFFER_SMALL,
            ShoreBufferTier::Medium => SHORE_BUFFER_MEDIUM,
            ShoreBufferTier::Large => SHORE_BUFFER_LARGE,
        }
    }

    /// Returns the agent radius for this tier.
    pub fn agent_radius(&self) -> f32 {
        match self {
            ShoreBufferTier::Small => AGENT_RADIUS_SMALL,
            ShoreBufferTier::Medium => AGENT_RADIUS_MEDIUM,
            ShoreBufferTier::Large => AGENT_RADIUS_LARGE,
        }
    }

    /// Determines the appropriate tier for a ship type.
    pub fn from_ship_type(ship_type: ShipType) -> Self {
        match ship_type {
            ShipType::Sloop => ShoreBufferTier::Small,
            ShipType::Raft => ShoreBufferTier::Small,
            ShipType::Schooner => ShoreBufferTier::Medium,
            ShipType::Frigate => ShoreBufferTier::Large,
        }
    }

    /// Returns all tiers in order of buffer size.
    pub fn all() -> &'static [ShoreBufferTier] {
        &[ShoreBufferTier::Small, ShoreBufferTier::Medium, ShoreBufferTier::Large]
    }
}

/// Resource containing the three archipelago entities for different ship sizes.
#[derive(Resource)]
pub struct LandmassArchipelagos {
    /// Archipelago for small ships (Sloop, Raft)
    pub small: Entity,
    /// Archipelago for medium ships (Schooner)
    pub medium: Entity,
    /// Archipelago for large ships (Frigate)
    pub large: Entity,
}

impl LandmassArchipelagos {
    /// Gets the archipelago entity for a specific tier.
    pub fn get(&self, tier: ShoreBufferTier) -> Entity {
        match tier {
            ShoreBufferTier::Small => self.small,
            ShoreBufferTier::Medium => self.medium,
            ShoreBufferTier::Large => self.large,
        }
    }

    /// Gets the archipelago entity for a ship type.
    pub fn get_for_ship(&self, ship_type: ShipType) -> Entity {
        self.get(ShoreBufferTier::from_ship_type(ship_type))
    }
}

/// Stores nav mesh data before it's inserted into the asset system.
/// Used during initialization to pass mesh data between systems.
#[derive(Resource)]
pub struct PendingNavMeshes {
    pub small: Option<NavigationMesh2d>,
    pub medium: Option<NavigationMesh2d>,
    pub large: Option<NavigationMesh2d>,
}

impl Default for PendingNavMeshes {
    fn default() -> Self {
        Self {
            small: None,
            medium: None,
            large: None,
        }
    }
}

/// Builds a NavigationMesh2d from vertices and triangles.
///
/// Converts the triangle-based output from spade to the polygon format
/// required by landmass. Returns the raw mesh without validation.
/// Validation happens when inserting into the asset system.
pub fn build_navigation_mesh_2d(
    vertices: Vec<Vec2>,
    triangles: Vec<[usize; 3]>,
) -> Result<NavigationMesh2d, String> {
    if vertices.is_empty() || triangles.is_empty() {
        return Err("Empty vertices or triangles".to_string());
    }

    // Convert triangles to polygon format
    let polygons: Vec<Vec<usize>> = triangles
        .iter()
        .map(|t| vec![t[0], t[1], t[2]])
        .collect();

    // All polygons are navigable (type 0)
    let polygon_type_indices = vec![0; polygons.len()];

    Ok(NavigationMesh2d {
        vertices,
        polygons,
        polygon_type_indices,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shore_buffer_tier_from_ship_type() {
        assert_eq!(
            ShoreBufferTier::from_ship_type(ShipType::Sloop),
            ShoreBufferTier::Small
        );
        assert_eq!(
            ShoreBufferTier::from_ship_type(ShipType::Frigate),
            ShoreBufferTier::Large
        );
        assert_eq!(
            ShoreBufferTier::from_ship_type(ShipType::Schooner),
            ShoreBufferTier::Medium
        );
    }

    #[test]
    fn test_build_navigation_mesh_2d() {
        let vertices = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(50.0, 100.0),
        ];
        let triangles = vec![[0, 1, 2]];

        let result = build_navigation_mesh_2d(vertices, triangles);
        assert!(result.is_ok());
    }
}
