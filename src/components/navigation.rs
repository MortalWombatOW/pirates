use bevy::prelude::*;

/// Component indicating the player's desired destination on the world map.
#[derive(Component, Debug, Clone)]
pub struct Destination {
    /// Target position in world coordinates.
    pub target: Vec2,
}

/// Component holding the calculated path waypoints for navigation.
#[derive(Component, Debug, Clone, Default)]
pub struct NavigationPath {
    /// List of waypoints from current position to destination.
    /// First element is the next waypoint to move toward.
    pub waypoints: Vec<Vec2>,
}

impl NavigationPath {
    /// Returns the next waypoint to navigate toward, if any.
    pub fn next_waypoint(&self) -> Option<Vec2> {
        self.waypoints.first().copied()
    }
    
    /// Removes the first waypoint (called when reached).
    pub fn pop_waypoint(&mut self) {
        if !self.waypoints.is_empty() {
            self.waypoints.remove(0);
        }
    }
    
    /// Returns true if there are no more waypoints.
    pub fn is_empty(&self) -> bool {
        self.waypoints.is_empty()
    }
}
