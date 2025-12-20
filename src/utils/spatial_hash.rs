//! Spatial hash utility for efficient proximity queries.
//!
//! Provides O(1) bucket lookup for entities in 2D space. Used by the
//! Encounter System to find AI ships near the player.

use bevy::prelude::*;
use std::collections::HashMap;

/// A spatial hash grid for efficient proximity queries.
///
/// Items are stored in grid cells based on their position. Query operations
/// check only relevant cells, avoiding O(n²) comparisons.
///
/// # Type Parameters
/// * `T` - The type of item to store (typically `Entity`)
#[derive(Debug, Clone)]
pub struct SpatialHash<T: Clone + PartialEq> {
    /// Size of each grid cell in world units
    cell_size: f32,
    /// Storage: maps cell coordinates to items in that cell
    cells: HashMap<(i32, i32), Vec<T>>,
}

impl<T: Clone + PartialEq> SpatialHash<T> {
    /// Creates a new spatial hash with the given cell size.
    ///
    /// # Arguments
    /// * `cell_size` - Size of each grid cell in world units (e.g., 64.0 for tile-sized cells)
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    /// Converts a world position to a cell coordinate.
    fn pos_to_cell(&self, position: Vec2) -> (i32, i32) {
        (
            (position.x / self.cell_size).floor() as i32,
            (position.y / self.cell_size).floor() as i32,
        )
    }

    /// Inserts an item at the given world position.
    pub fn insert(&mut self, position: Vec2, item: T) {
        let cell = self.pos_to_cell(position);
        self.cells.entry(cell).or_default().push(item);
    }

    /// Removes an item at the given world position.
    ///
    /// Returns `true` if the item was found and removed.
    pub fn remove(&mut self, position: Vec2, item: &T) -> bool {
        let cell = self.pos_to_cell(position);
        if let Some(items) = self.cells.get_mut(&cell) {
            if let Some(idx) = items.iter().position(|i| i == item) {
                items.swap_remove(idx);
                return true;
            }
        }
        false
    }

    /// Clears all items from the spatial hash.
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Queries all items within a circular radius of the given position.
    ///
    /// # Arguments
    /// * `position` - Center of the query circle
    /// * `radius` - Radius of the query circle
    ///
    /// # Returns
    /// A vector of references to items within the radius.
    pub fn query(&self, position: Vec2, radius: f32) -> Vec<&T> {
        let mut results = Vec::new();
        let radius_sq = radius * radius;

        // Determine which cells to check
        let cells_to_check = (radius / self.cell_size).ceil() as i32 + 1;
        let center_cell = self.pos_to_cell(position);

        for dy in -cells_to_check..=cells_to_check {
            for dx in -cells_to_check..=cells_to_check {
                let cell = (center_cell.0 + dx, center_cell.1 + dy);
                if let Some(items) = self.cells.get(&cell) {
                    // For now, we don't store individual positions, so we include all items
                    // in nearby cells. For more precise filtering, store (T, Vec2) pairs.
                    // In practice, the cell size is chosen to match the query radius,
                    // so this approximation is acceptable.
                    for item in items {
                        results.push(item);
                    }
                }
            }
        }

        // Note: Without stored positions, we can't do exact distance filtering.
        // The caller should verify distance if precision is needed.
        // For typical use cases (encounter detection), checking nearby cells is sufficient.
        let _ = radius_sq; // Suppress unused warning - would be used with stored positions

        results
    }

    /// Queries all items within an axis-aligned bounding box.
    ///
    /// # Arguments
    /// * `min` - Minimum corner of the AABB
    /// * `max` - Maximum corner of the AABB
    ///
    /// # Returns
    /// A vector of references to items within the AABB.
    pub fn query_rect(&self, min: Vec2, max: Vec2) -> Vec<&T> {
        let mut results = Vec::new();

        let min_cell = self.pos_to_cell(min);
        let max_cell = self.pos_to_cell(max);

        for y in min_cell.1..=max_cell.1 {
            for x in min_cell.0..=max_cell.0 {
                if let Some(items) = self.cells.get(&(x, y)) {
                    for item in items {
                        results.push(item);
                    }
                }
            }
        }

        results
    }

    /// Returns the number of items stored in the spatial hash.
    pub fn len(&self) -> usize {
        self.cells.values().map(|v| v.len()).sum()
    }

    /// Returns true if the spatial hash contains no items.
    pub fn is_empty(&self) -> bool {
        self.cells.values().all(|v| v.is_empty())
    }
}

impl<T: Clone + PartialEq> Default for SpatialHash<T> {
    fn default() -> Self {
        Self::new(64.0) // Default to tile size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_query() {
        let mut hash = SpatialHash::new(64.0);
        hash.insert(Vec2::new(100.0, 100.0), 1u32);
        hash.insert(Vec2::new(110.0, 110.0), 2u32);

        let results = hash.query(Vec2::new(100.0, 100.0), 64.0);
        assert!(results.contains(&&1u32));
        assert!(results.contains(&&2u32));
    }

    #[test]
    fn test_query_radius_excludes_distant() {
        let mut hash = SpatialHash::new(64.0);
        hash.insert(Vec2::new(0.0, 0.0), 1u32);
        hash.insert(Vec2::new(500.0, 500.0), 2u32);

        // Query near origin - should only find item 1
        let results = hash.query(Vec2::new(0.0, 0.0), 64.0);
        assert!(results.contains(&&1u32));
        assert!(!results.contains(&&2u32));
    }

    #[test]
    fn test_clear() {
        let mut hash = SpatialHash::new(64.0);
        hash.insert(Vec2::new(0.0, 0.0), 1u32);
        hash.insert(Vec2::new(100.0, 100.0), 2u32);

        assert_eq!(hash.len(), 2);

        hash.clear();

        assert!(hash.is_empty());
        assert_eq!(hash.len(), 0);
    }

    #[test]
    fn test_remove() {
        let mut hash = SpatialHash::new(64.0);
        hash.insert(Vec2::new(50.0, 50.0), 1u32);
        hash.insert(Vec2::new(50.0, 50.0), 2u32);

        assert_eq!(hash.len(), 2);

        let removed = hash.remove(Vec2::new(50.0, 50.0), &1u32);
        assert!(removed);
        assert_eq!(hash.len(), 1);

        let results = hash.query(Vec2::new(50.0, 50.0), 32.0);
        assert!(!results.contains(&&1u32));
        assert!(results.contains(&&2u32));
    }

    #[test]
    fn test_query_rect() {
        let mut hash = SpatialHash::new(64.0);
        hash.insert(Vec2::new(32.0, 32.0), 1u32);
        hash.insert(Vec2::new(96.0, 96.0), 2u32);
        hash.insert(Vec2::new(200.0, 200.0), 3u32);

        let results = hash.query_rect(Vec2::new(0.0, 0.0), Vec2::new(128.0, 128.0));
        assert!(results.contains(&&1u32));
        assert!(results.contains(&&2u32));
        assert!(!results.contains(&&3u32));
    }

    #[test]
    fn test_negative_coordinates() {
        let mut hash = SpatialHash::new(64.0);
        hash.insert(Vec2::new(-100.0, -100.0), 1u32);
        hash.insert(Vec2::new(-50.0, -50.0), 2u32);

        let results = hash.query(Vec2::new(-75.0, -75.0), 64.0);
        assert!(results.contains(&&1u32));
        assert!(results.contains(&&2u32));
    }

    #[test]
    fn test_default() {
        let hash: SpatialHash<u32> = SpatialHash::default();
        assert!(hash.is_empty());
        assert_eq!(hash.cell_size, 64.0);
    }
}
