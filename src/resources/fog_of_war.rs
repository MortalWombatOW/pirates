use bevy::prelude::*;
use bevy::utils::HashSet;

/// Resource that tracks which tiles on the world map have been explored by the player.
/// 
/// This is used by:
/// - `FogOfWarSystem` to update visibility
/// - Rendering systems to apply the fog overlay
#[derive(Resource, Default, Debug)]
pub struct FogOfWar {
    /// Set of tile coordinates (x, y) that have been revealed.
    explored_tiles: HashSet<IVec2>,
}

impl FogOfWar {
    /// Adds a tile coordinate to the set of explored tiles.
    pub fn explore(&mut self, pos: IVec2) {
        self.explored_tiles.insert(pos);
    }

    /// Checks if a tile coordinate has been explored.
    pub fn is_explored(&self, pos: IVec2) -> bool {
        self.explored_tiles.contains(&pos)
    }

    /// Returns the number of explored tiles.
    pub fn explored_count(&self) -> usize {
        self.explored_tiles.len()
    }

    /// Clears all explored tiles (e.g., for a new game).
    pub fn clear(&mut self) {
        self.explored_tiles.clear();
    }
}
