//! Route caching resource for AI navigation.
//!
//! Stores calculated Theta* paths between map tiles to avoid re-running
//! expensive pathfinding for repeated journeys (e.g., trade routes).

use bevy::prelude::*;
use std::collections::HashMap;

/// Cached paths between map tiles.
/// 
/// Key is (start_tile, goal_tile).
/// Value is the list of waypoints (tile coordinates) for the path.
#[derive(Resource, Default, Debug)]
pub struct RouteCache {
    /// Map from (start, goal) to path.
    cache: HashMap<(IVec2, IVec2), Vec<IVec2>>,
}

impl RouteCache {
    /// Creates a new empty route cache.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Retrieves a cached path if it exists.
    pub fn get(&self, start: IVec2, goal: IVec2) -> Option<&Vec<IVec2>> {
        self.cache.get(&(start, goal))
    }

    /// Inserts a path into the cache.
    pub fn insert(&mut self, start: IVec2, goal: IVec2, path: Vec<IVec2>) {
        self.cache.insert((start, goal), path);
    }

    /// Clears the entire cache.
    /// Should be called on map regeneration or significant world changes.
    pub fn clear(&mut self) {
        self.cache.clear();
    }
    
    /// Returns the number of cached routes.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}
