//! A* pathfinding utility for world map navigation.
//!
//! Operates on the `MapData` resource to find paths around land tiles.

use bevy::prelude::*;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

use crate::resources::MapData;

/// A node in the A* search.
#[derive(Clone, Copy, Eq, PartialEq)]
struct Node {
    pos: IVec2,
    cost: u32,      // g: cost from start
    priority: u32,  // f: g + h (heuristic)
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (BinaryHeap is max-heap by default)
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Finds a path from `start` to `goal` using A* algorithm.
/// Returns a list of tile positions (not world positions).
/// 
/// # Arguments
/// * `start` - Starting tile position
/// * `goal` - Target tile position
/// * `map_data` - The world map data for checking navigability
/// 
/// # Returns
/// `Some(Vec<IVec2>)` with path from start to goal (inclusive), or `None` if no path exists.
pub fn find_path(start: IVec2, goal: IVec2, map_data: &MapData) -> Option<Vec<IVec2>> {
    // Early exit if goal is not navigable
    if !map_data.is_navigable(goal.x as u32, goal.y as u32) {
        return None;
    }
    
    // Early exit if start equals goal
    if start == goal {
        return Some(vec![goal]);
    }
    
    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<IVec2, IVec2> = HashMap::new();
    let mut g_score: HashMap<IVec2, u32> = HashMap::new();
    
    g_score.insert(start, 0);
    open_set.push(Node {
        pos: start,
        cost: 0,
        priority: heuristic(start, goal),
    });
    
    while let Some(current) = open_set.pop() {
        if current.pos == goal {
            // Reconstruct path
            return Some(reconstruct_path(&came_from, current.pos));
        }
        
        // Explore neighbors (4-directional)
        for neighbor in neighbors_4(current.pos, map_data) {
            let tentative_g = g_score.get(&current.pos).unwrap_or(&u32::MAX).saturating_add(1);
            
            if tentative_g < *g_score.get(&neighbor).unwrap_or(&u32::MAX) {
                came_from.insert(neighbor, current.pos);
                g_score.insert(neighbor, tentative_g);
                
                let f_score = tentative_g + heuristic(neighbor, goal);
                open_set.push(Node {
                    pos: neighbor,
                    cost: tentative_g,
                    priority: f_score,
                });
            }
        }
    }
    
    None // No path found
}

/// Manhattan distance heuristic for A*.
fn heuristic(a: IVec2, b: IVec2) -> u32 {
    ((a.x - b.x).abs() + (a.y - b.y).abs()) as u32
}

/// Returns navigable 4-directional neighbors.
fn neighbors_4(pos: IVec2, map_data: &MapData) -> Vec<IVec2> {
    let directions = [
        IVec2::new(1, 0),
        IVec2::new(-1, 0),
        IVec2::new(0, 1),
        IVec2::new(0, -1),
    ];
    
    directions
        .iter()
        .map(|d| pos + *d)
        .filter(|n| {
            n.x >= 0 
                && n.y >= 0 
                && (n.x as u32) < map_data.width 
                && (n.y as u32) < map_data.height
                && map_data.is_navigable(n.x as u32, n.y as u32)
        })
        .collect()
}

/// Reconstructs the path from the came_from map.
fn reconstruct_path(came_from: &HashMap<IVec2, IVec2>, mut current: IVec2) -> Vec<IVec2> {
    let mut path = vec![current];
    while let Some(&prev) = came_from.get(&current) {
        path.push(prev);
        current = prev;
    }
    path.reverse();
    path
}

/// Converts a tile position to world coordinates.
/// Assumes 64x64 tile size and map centered at origin.
pub fn tile_to_world(tile_pos: IVec2, map_width: u32, map_height: u32) -> Vec2 {
    let tile_size = 64.0;
    let offset_x = (map_width as f32 * tile_size) / 2.0;
    let offset_y = (map_height as f32 * tile_size) / 2.0;
    
    Vec2::new(
        tile_pos.x as f32 * tile_size - offset_x + tile_size / 2.0,
        tile_pos.y as f32 * tile_size - offset_y + tile_size / 2.0,
    )
}

/// Converts world coordinates to a tile position.
pub fn world_to_tile(world_pos: Vec2, map_width: u32, map_height: u32) -> IVec2 {
    let tile_size = 64.0;
    let offset_x = (map_width as f32 * tile_size) / 2.0;
    let offset_y = (map_height as f32 * tile_size) / 2.0;
    
    IVec2::new(
        ((world_pos.x + offset_x) / tile_size).floor() as i32,
        ((world_pos.y + offset_y) / tile_size).floor() as i32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::TileType;

    fn create_test_map() -> MapData {
        let mut map = MapData::new(10, 10);
        // Add some land in the middle
        map.set(5, 5, TileType::Land);
        map.set(5, 4, TileType::Land);
        map.set(5, 6, TileType::Land);
        map
    }

    #[test]
    fn test_direct_path() {
        let map = create_test_map();
        let path = find_path(IVec2::new(0, 0), IVec2::new(3, 0), &map);
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.len() >= 4); // At least 4 tiles to traverse
    }

    #[test]
    fn test_path_around_obstacle() {
        let map = create_test_map();
        let path = find_path(IVec2::new(4, 5), IVec2::new(6, 5), &map);
        assert!(path.is_some());
        // Should go around the land, not through it
        let path = path.unwrap();
        for pos in &path {
            assert!(map.is_navigable(pos.x as u32, pos.y as u32));
        }
    }

    #[test]
    fn test_no_path_to_land() {
        let map = create_test_map();
        let path = find_path(IVec2::new(0, 0), IVec2::new(5, 5), &map);
        assert!(path.is_none());
    }
}
