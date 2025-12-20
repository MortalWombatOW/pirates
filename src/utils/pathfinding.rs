//! Theta* pathfinding utility for world map navigation.
//!
//! Implements the Basic Theta* algorithm for any-angle pathfinding on grids.
//! Operates on the `MapData` resource to find paths around land tiles.

use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::resources::MapData;

/// Wrapper for f32 that implements Ord for use in BinaryHeap.
/// Uses total ordering where NaN is treated as greater than all other values.
#[derive(Clone, Copy, PartialEq)]
struct OrderedF32(f32);

impl Eq for OrderedF32 {}

impl Ord for OrderedF32 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for OrderedF32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl OrderedF32 {
    fn new(value: f32) -> Self {
        Self(value)
    }
}

impl std::ops::Add for OrderedF32 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

/// A node in the Theta* search.
#[derive(Clone, Copy, PartialEq)]
struct Node {
    pos: IVec2,
    cost: OrderedF32,     // g: cost from start
    priority: OrderedF32, // f: g + h (heuristic)
}

impl Eq for Node {}

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

/// Finds a path from `start` to `goal` using Theta* algorithm.
/// Returns a list of tile positions (not world positions).
///
/// Theta* produces any-angle paths by checking line-of-sight to parent nodes,
/// resulting in shorter, more natural paths than standard A*.
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
    let mut g_score: HashMap<IVec2, OrderedF32> = HashMap::new();
    let mut closed_set: HashMap<IVec2, bool> = HashMap::new();

    g_score.insert(start, OrderedF32::new(0.0));
    open_set.push(Node {
        pos: start,
        cost: OrderedF32::new(0.0),
        priority: OrderedF32::new(heuristic(start, goal)),
    });

    while let Some(current) = open_set.pop() {
        if current.pos == goal {
            // Reconstruct path
            return Some(reconstruct_path(&came_from, current.pos));
        }

        // Skip if already processed with better cost
        if closed_set.contains_key(&current.pos) {
            continue;
        }
        closed_set.insert(current.pos, true);

        // Get parent of current node (if any)
        let parent = came_from.get(&current.pos).copied();

        // Explore neighbors (8-directional)
        for neighbor in neighbors_8(current.pos, goal, map_data) {
            if closed_set.contains_key(&neighbor) {
                continue;
            }

            // Theta* logic: try to connect neighbor directly to parent
            let (new_g, source) = compute_cost(
                current.pos,
                neighbor,
                parent,
                &g_score,
                map_data,
            );

            let current_g = g_score.get(&neighbor).copied().unwrap_or(OrderedF32::new(f32::INFINITY));

            if new_g.0 < current_g.0 {
                came_from.insert(neighbor, source);
                g_score.insert(neighbor, new_g);

                let f_score = new_g + OrderedF32::new(heuristic(neighbor, goal));
                open_set.push(Node {
                    pos: neighbor,
                    cost: new_g,
                    priority: f_score,
                });
            }
        }
    }

    None // No path found
}

/// Computes the cost to reach a neighbor, implementing Theta* path selection.
///
/// Path 2 (Theta* shortcut): If parent has line-of-sight to neighbor, connect directly.
/// Path 1 (Standard A*): Otherwise, go through current node.
fn compute_cost(
    current: IVec2,
    neighbor: IVec2,
    parent: Option<IVec2>,
    g_score: &HashMap<IVec2, OrderedF32>,
    map_data: &MapData,
) -> (OrderedF32, IVec2) {
    // Apply coastal penalty: 5x cost for water tiles adjacent to land
    let coastal_multiplier = if is_coastal(neighbor, map_data) { 5.0 } else { 1.0 };

    // Try Path 2: direct connection from parent to neighbor
    if let Some(parent_pos) = parent {
        if line_of_sight(parent_pos, neighbor, map_data) {
            let parent_g = g_score.get(&parent_pos).copied().unwrap_or(OrderedF32::new(0.0));
            let base_cost = euclidean_distance(parent_pos, neighbor);
            let cost = parent_g + OrderedF32::new(base_cost * coastal_multiplier);
            return (cost, parent_pos);
        }
    }

    // Path 1: standard A* through current node
    let current_g = g_score.get(&current).copied().unwrap_or(OrderedF32::new(0.0));
    let base_cost = euclidean_distance(current, neighbor);
    let cost = current_g + OrderedF32::new(base_cost * coastal_multiplier);
    (cost, current)
}

/// Checks if a water tile is "coastal" (adjacent to any non-navigable tile).
/// Coastal tiles receive a movement cost penalty to encourage open-water routes.
fn is_coastal(pos: IVec2, map_data: &MapData) -> bool {
    let directions = [
        IVec2::new(1, 0),
        IVec2::new(-1, 0),
        IVec2::new(0, 1),
        IVec2::new(0, -1),
        IVec2::new(1, 1),
        IVec2::new(-1, 1),
        IVec2::new(1, -1),
        IVec2::new(-1, -1),
    ];

    for dir in directions {
        let adj = pos + dir;
        if !map_data.in_bounds(adj.x, adj.y) {
            continue;
        }
        if !map_data.is_navigable(adj.x as u32, adj.y as u32) {
            return true;
        }
    }
    false
}

/// Euclidean distance heuristic for Theta*.
fn heuristic(a: IVec2, b: IVec2) -> f32 {
    euclidean_distance(a, b)
}

/// Computes Euclidean distance between two points.
fn euclidean_distance(a: IVec2, b: IVec2) -> f32 {
    let dx = (a.x - b.x) as f32;
    let dy = (a.y - b.y) as f32;
    (dx * dx + dy * dy).sqrt()
}

/// Checks if there is a clear line of sight between two grid positions.
///
/// Uses a supercover line algorithm that checks ALL cells the line passes through,
/// including cells that are just barely touched at corners. This is more conservative
/// than standard Bresenham and prevents any corner cutting.
fn line_of_sight(p1: IVec2, p2: IVec2, map_data: &MapData) -> bool {
    let mut x = p1.x;
    let mut y = p1.y;
    let dx = (p2.x - p1.x).abs();
    let dy = (p2.y - p1.y).abs();
    let sx = if p1.x < p2.x { 1 } else { -1 };
    let sy = if p1.y < p2.y { 1 } else { -1 };

    // Check start cell
    if !map_data.in_bounds(x, y) || !map_data.is_navigable(x as u32, y as u32) {
        return false;
    }

    // Handle degenerate cases
    if dx == 0 && dy == 0 {
        return true;
    }

    let mut err = dx - dy;

    while x != p2.x || y != p2.y {
        let e2 = 2 * err;

        // Determine step direction
        let step_x = e2 > -dy;
        let step_y = e2 < dx;

        if step_x && step_y {
            // Diagonal step: check BOTH intermediate cells (supercover)
            // This is the key difference from Bresenham - we check both cells
            // that the line might pass through when moving diagonally
            let cell_x = IVec2::new(x + sx, y);
            let cell_y = IVec2::new(x, y + sy);

            let x_blocked = !map_data.in_bounds(cell_x.x, cell_x.y)
                || !map_data.is_navigable(cell_x.x as u32, cell_x.y as u32);
            let y_blocked = !map_data.in_bounds(cell_y.x, cell_y.y)
                || !map_data.is_navigable(cell_y.x as u32, cell_y.y as u32);

            // Block if EITHER adjacent cell is not navigable (strict corner prevention)
            if x_blocked || y_blocked {
                return false;
            }

            err -= dy;
            err += dx;
            x += sx;
            y += sy;
        } else if step_x {
            err -= dy;
            x += sx;
        } else {
            err += dx;
            y += sy;
        }

        // Check current cell
        if !map_data.in_bounds(x, y) || !map_data.is_navigable(x as u32, y as u32) {
            return false;
        }
    }

    true
}

/// Returns navigable 8-directional neighbors with corner-cutting prevention.
///
/// Diagonal movement is only allowed if both adjacent cardinal directions
/// are navigable, preventing ships from cutting through land corners.
/// 
/// Enforces 1-tile shore buffer: coastal tiles are only allowed if they are the goal.
fn neighbors_8(pos: IVec2, goal: IVec2, map_data: &MapData) -> Vec<IVec2> {
    let mut neighbors = Vec::with_capacity(8);

    // Check cardinal neighbors (with shore buffer except for goal)
    let e_pos = pos + IVec2::new(1, 0);
    let w_pos = pos + IVec2::new(-1, 0);
    let n_pos = pos + IVec2::new(0, 1);
    let s_pos = pos + IVec2::new(0, -1);
    
    let e_ok = is_valid_neighbor_with_buffer(e_pos, goal, map_data);
    let w_ok = is_valid_neighbor_with_buffer(w_pos, goal, map_data);
    let n_ok = is_valid_neighbor_with_buffer(n_pos, goal, map_data);
    let s_ok = is_valid_neighbor_with_buffer(s_pos, goal, map_data);

    // Add valid cardinal neighbors
    if e_ok { neighbors.push(e_pos); }
    if w_ok { neighbors.push(w_pos); }
    if n_ok { neighbors.push(n_pos); }
    if s_ok { neighbors.push(s_pos); }

    // Diagonal directions - only allow if both adjacent cardinals are passable
    let ne_pos = pos + IVec2::new(1, 1);
    let nw_pos = pos + IVec2::new(-1, 1);
    let se_pos = pos + IVec2::new(1, -1);
    let sw_pos = pos + IVec2::new(-1, -1);
    
    if n_ok && e_ok && is_valid_neighbor_with_buffer(ne_pos, goal, map_data) {
        neighbors.push(ne_pos);
    }
    if n_ok && w_ok && is_valid_neighbor_with_buffer(nw_pos, goal, map_data) {
        neighbors.push(nw_pos);
    }
    if s_ok && e_ok && is_valid_neighbor_with_buffer(se_pos, goal, map_data) {
        neighbors.push(se_pos);
    }
    if s_ok && w_ok && is_valid_neighbor_with_buffer(sw_pos, goal, map_data) {
        neighbors.push(sw_pos);
    }

    neighbors
}

/// Helper to check if a position is valid with 1-tile shore buffer.
/// Goal tile is exempt from the shore buffer requirement.
fn is_valid_neighbor_with_buffer(pos: IVec2, goal: IVec2, map_data: &MapData) -> bool {
    // Basic bounds and navigability check
    if pos.x < 0 || pos.y < 0 
        || (pos.x as u32) >= map_data.width 
        || (pos.y as u32) >= map_data.height {
        return false;
    }
    if !map_data.is_navigable(pos.x as u32, pos.y as u32) {
        return false;
    }
    
    // Goal is always valid (no shore buffer)
    if pos == goal {
        return true;
    }
    
    // Enforce 1-tile shore buffer: reject if any adjacent tile is land
    !is_coastal(pos, map_data)
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
        // Theta* should produce a more direct path (possibly just 2 waypoints for straight line)
        assert!(path.len() >= 2);
        assert_eq!(path[0], IVec2::new(0, 0));
        assert_eq!(*path.last().unwrap(), IVec2::new(3, 0));
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

    #[test]
    fn test_line_of_sight_clear() {
        let map = create_test_map();
        // Clear line of sight in open water
        assert!(line_of_sight(IVec2::new(0, 0), IVec2::new(3, 3), &map));
    }

    #[test]
    fn test_line_of_sight_blocked() {
        let map = create_test_map();
        // Line through land should be blocked
        assert!(!line_of_sight(IVec2::new(4, 5), IVec2::new(6, 5), &map));
    }

    #[test]
    fn test_diagonal_movement() {
        let map = MapData::new(10, 10); // All water
        let path = find_path(IVec2::new(0, 0), IVec2::new(5, 5), &map);
        assert!(path.is_some());
        let path = path.unwrap();
        // Theta* on open water should produce a nearly direct path
        // Just start and end for a diagonal line with LOS
        assert!(path.len() <= 3, "Expected short path, got {} waypoints", path.len());
    }

    #[test]
    fn test_corner_cutting_prevention() {
        let mut map = MapData::new(10, 10);
        // Create a diagonal wall that should block corner cutting
        map.set(4, 5, TileType::Land);
        map.set(5, 4, TileType::Land);

        // Trying to go from (3, 4) to (5, 5) should not cut through the diagonal
        // Use (6, 6) as goal - far enough to not affect neighbor calculations
        let neighbors = neighbors_8(IVec2::new(4, 4), IVec2::new(6, 6), &map);

        // (5, 5) should NOT be a valid diagonal neighbor because (5, 4) is land
        assert!(!neighbors.contains(&IVec2::new(5, 5)));
    }
}
