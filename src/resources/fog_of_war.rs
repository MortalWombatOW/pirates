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
    /// Tiles that were newly explored this frame (for efficient tilemap updates).
    newly_explored: Vec<IVec2>,
}

impl FogOfWar {
    /// Adds a tile coordinate to the set of explored tiles.
    /// Returns true if the tile was newly explored, false if already known.
    pub fn explore(&mut self, pos: IVec2) -> bool {
        // Only mutate if this is actually a new tile
        // This avoids triggering change detection on already-explored tiles
        if self.explored_tiles.contains(&pos) {
            return false;
        }
        self.explored_tiles.insert(pos);
        self.newly_explored.push(pos);
        true
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
        self.newly_explored.clear();
    }

    /// Returns and clears the list of newly explored tiles.
    /// Call this after updating the tilemap to reset for next frame.
    pub fn take_newly_explored(&mut self) -> Vec<IVec2> {
        std::mem::take(&mut self.newly_explored)
    }

    /// Returns true if there are newly explored tiles to process.
    pub fn has_newly_explored(&self) -> bool {
        !self.newly_explored.is_empty()
    }
}
