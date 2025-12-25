//! Nav Mesh resource for polygon-based pathfinding.
//!
//! Provides efficient pathfinding on triangulated water regions.
//! Supports multiple mesh tiers based on ship size (shore buffer distance).

use bevy::prelude::*;
use navmesh::{NavMesh, NavVec3, NavTriangle, NavQuery, NavPathMode};

use crate::components::ship::ShipType;

/// Shore buffer distances for different ship sizes (in world units).
/// Larger ships need more clearance from coastlines.
pub const SHORE_BUFFER_SMALL: f32 = 32.0;   // Sloop, Raft
pub const SHORE_BUFFER_MEDIUM: f32 = 64.0;  // Schooner
pub const SHORE_BUFFER_LARGE: f32 = 96.0;   // Frigate

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

/// A single nav mesh for a specific shore buffer tier.
#[derive(Clone)]
pub struct TieredNavMesh {
    /// The underlying navmesh crate mesh.
    mesh: NavMesh,
    /// Shore buffer tier this mesh represents.
    pub tier: ShoreBufferTier,
    /// Debug: vertex count for visualization.
    pub vertex_count: usize,
    /// Debug: triangle count for visualization.
    pub triangle_count: usize,
}

impl TieredNavMesh {
    /// Creates a new tiered nav mesh from vertices and triangles.
    pub fn new(
        vertices: Vec<Vec2>,
        triangles: Vec<[usize; 3]>,
        tier: ShoreBufferTier,
    ) -> Option<Self> {
        if vertices.is_empty() || triangles.is_empty() {
            return None;
        }

        let vertex_count = vertices.len();
        let triangle_count = triangles.len();

        // Convert to navmesh format (2D -> 3D with z=0)
        let nav_vertices: Vec<NavVec3> = vertices
            .iter()
            .map(|v| NavVec3::new(v.x, v.y, 0.0))
            .collect();

        let nav_triangles: Vec<NavTriangle> = triangles
            .iter()
            .map(|t| (t[0] as u32, t[1] as u32, t[2] as u32).into())
            .collect();

        match NavMesh::new(nav_vertices, nav_triangles) {
            Ok(mesh) => Some(Self {
                mesh,
                tier,
                vertex_count,
                triangle_count,
            }),
            Err(e) => {
                warn!("Failed to create NavMesh for tier {:?}: {:?}", tier, e);
                None
            }
        }
    }

    /// Finds a path from start to goal.
    pub fn find_path(&self, start: Vec2, goal: Vec2) -> Option<Vec<Vec2>> {
        let start_3d = NavVec3::new(start.x, start.y, 0.0);
        let goal_3d = NavVec3::new(goal.x, goal.y, 0.0);

        // Use midpoints mode for smoother paths through triangle centers
        let query = NavQuery::Closest;
        let mode = NavPathMode::MidPoints;

        match self.mesh.find_path(start_3d, goal_3d, query, mode) {
            Some(path) => {
                let result: Vec<Vec2> = path
                    .into_iter()
                    .map(|p| Vec2::new(p.x as f32, p.y as f32))
                    .collect();
                Some(result)
            }
            None => None,
        }
    }

    /// Returns the raw vertices for debug visualization.
    pub fn debug_vertices(&self) -> Vec<Vec2> {
        self.mesh
            .vertices()
            .iter()
            .map(|v| Vec2::new(v.x as f32, v.y as f32))
            .collect()
    }

    /// Returns the triangle indices for debug visualization.
    pub fn debug_triangles(&self) -> Vec<[usize; 3]> {
        self.mesh
            .triangles()
            .iter()
            .map(|t| [
                t.first as usize,
                t.second as usize,
                t.third as usize,
            ])
            .collect()
    }
}

/// Bevy Resource containing nav meshes for all ship size tiers.
#[derive(Resource, Default)]
pub struct NavMeshResource {
    /// Nav meshes indexed by tier. All tiers are optional.
    meshes: [Option<TieredNavMesh>; 3],
}

impl NavMeshResource {
    /// Creates a new empty resource.
    pub fn new() -> Self {
        Self {
            meshes: [None, None, None],
        }
    }

    /// Sets the nav mesh for a specific tier.
    pub fn set_mesh(&mut self, tier: ShoreBufferTier, mesh: TieredNavMesh) {
        let index = tier as usize;
        self.meshes[index] = Some(mesh);
    }

    /// Gets the nav mesh for a specific tier.
    pub fn get_mesh(&self, tier: ShoreBufferTier) -> Option<&TieredNavMesh> {
        self.meshes[tier as usize].as_ref()
    }

    /// Finds a path using the appropriate tier for the given ship type.
    pub fn find_path_for_ship(
        &self,
        start: Vec2,
        goal: Vec2,
        ship_type: ShipType,
    ) -> Option<Vec<Vec2>> {
        let tier = ShoreBufferTier::from_ship_type(ship_type);
        self.find_path(start, goal, tier)
    }

    /// Finds a path using the specified tier.
    pub fn find_path(
        &self,
        start: Vec2,
        goal: Vec2,
        tier: ShoreBufferTier,
    ) -> Option<Vec<Vec2>> {
        // Try the requested tier first
        if let Some(mesh) = self.get_mesh(tier) {
            if let Some(path) = mesh.find_path(start, goal) {
                return Some(path);
            }
        }

        // Fall back to smaller tiers if requested tier fails
        let tier_index = tier as usize;
        for i in (0..tier_index).rev() {
            let fallback_tier = match i {
                0 => ShoreBufferTier::Small,
                1 => ShoreBufferTier::Medium,
                _ => continue,
            };
            if let Some(mesh) = self.get_mesh(fallback_tier) {
                if let Some(path) = mesh.find_path(start, goal) {
                    debug!(
                        "NavMesh: Falling back from {:?} to {:?} tier",
                        tier, fallback_tier
                    );
                    return Some(path);
                }
            }
        }

        None
    }

    /// Returns true if any meshes are loaded.
    pub fn is_ready(&self) -> bool {
        self.meshes.iter().any(|m| m.is_some())
    }

    /// Returns statistics for debug display.
    pub fn stats(&self) -> Vec<(ShoreBufferTier, usize, usize)> {
        ShoreBufferTier::all()
            .iter()
            .filter_map(|&tier| {
                self.get_mesh(tier)
                    .map(|m| (tier, m.vertex_count, m.triangle_count))
            })
            .collect()
    }
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
    fn test_simple_navmesh_creation() {
        // A simple triangle covering a water area
        let vertices = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(50.0, 100.0),
        ];
        let triangles = vec![[0, 1, 2]];

        let mesh = TieredNavMesh::new(vertices, triangles, ShoreBufferTier::Small);
        assert!(mesh.is_some());

        let mesh = mesh.unwrap();
        assert_eq!(mesh.vertex_count, 3);
        assert_eq!(mesh.triangle_count, 1);
    }

    #[test]
    fn test_navmesh_resource_fallback() {
        let mut resource = NavMeshResource::new();

        // Only create small tier mesh
        let vertices = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(50.0, 100.0),
        ];
        let triangles = vec![[0, 1, 2]];

        let mesh = TieredNavMesh::new(
            vertices,
            triangles,
            ShoreBufferTier::Small,
        ).unwrap();
        resource.set_mesh(ShoreBufferTier::Small, mesh);

        // Large tier should fall back to small
        let path = resource.find_path(
            Vec2::new(30.0, 30.0),
            Vec2::new(60.0, 30.0),
            ShoreBufferTier::Large,
        );
        assert!(path.is_some());
    }
}
