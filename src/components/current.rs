//! Current zone components for water currents that affect physics objects.

use bevy::prelude::*;

/// A zone that applies a constant force to all physics objects within its bounds.
/// 
/// The zone is defined by an AABB centered on the entity's Transform with the given half-extents.
/// Any RigidBody entity inside the zone will have an external force applied equal to `velocity`.
#[derive(Component, Debug, Clone)]
pub struct CurrentZone {
    /// The velocity/force direction and magnitude of the current.
    /// This is applied as an ExternalForce to entities in the zone.
    pub velocity: Vec2,
    /// Half-extents of the zone's bounding box.
    pub half_extents: Vec2,
}

impl CurrentZone {
    /// Creates a new current zone with the given velocity and half-extents.
    pub fn new(velocity: Vec2, half_extents: Vec2) -> Self {
        Self { velocity, half_extents }
    }
    
    /// Creates a gentle rightward current zone.
    pub fn gentle_right(half_extents: Vec2) -> Self {
        Self::new(Vec2::new(50.0, 0.0), half_extents)
    }
    
    /// Checks if a point is inside this zone, given the zone's center position.
    pub fn contains(&self, zone_center: Vec2, point: Vec2) -> bool {
        let min = zone_center - self.half_extents;
        let max = zone_center + self.half_extents;
        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }
}
