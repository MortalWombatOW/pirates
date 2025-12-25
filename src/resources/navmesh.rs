//! Legacy navmesh module - DEPRECATED.
//!
//! This module is kept for backward compatibility during migration.
//! Use `crate::resources::landmass` instead.
//!
//! The old `navmesh` crate has been replaced with `bevy_landmass`.

// Re-export types from landmass for backward compatibility
pub use super::landmass::{
    ShoreBufferTier,
    SHORE_BUFFER_SMALL,
    SHORE_BUFFER_MEDIUM,
    SHORE_BUFFER_LARGE,
};

use bevy::prelude::*;

/// Legacy NavMeshResource - DEPRECATED.
///
/// This stub is kept for compilation during migration.
/// Use `LandmassArchipelagos` instead.
#[derive(Resource, Default)]
pub struct NavMeshResource {
    _placeholder: (),
}

impl NavMeshResource {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }

    /// Returns true if any meshes are loaded.
    /// Always returns false - use LandmassArchipelagos instead.
    pub fn is_ready(&self) -> bool {
        false
    }

    /// Legacy find_path - always returns None.
    /// Use landmass velocity-based navigation instead.
    pub fn find_path(
        &self,
        _start: Vec2,
        _goal: Vec2,
        _tier: ShoreBufferTier,
    ) -> Option<Vec<Vec2>> {
        warn!("NavMeshResource.find_path() is deprecated - use landmass navigation");
        None
    }

    /// Legacy find_path_for_ship - always returns None.
    /// Use landmass velocity-based navigation instead.
    pub fn find_path_for_ship(
        &self,
        _start: Vec2,
        _goal: Vec2,
        _ship_type: crate::components::ship::ShipType,
    ) -> Option<Vec<Vec2>> {
        warn!("NavMeshResource.find_path_for_ship() is deprecated - use landmass navigation");
        None
    }
}
