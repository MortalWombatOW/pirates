//! Geometry utilities for coastline extraction and vector graphics.

use bevy::prelude::*;
use crate::resources::MapData;
use crate::resources::map_data::TileType;

/// A single closed coastline polygon loop.
/// Points are ordered Counter-Clockwise (CCW), meaning Land is always
/// to the left of the travel direction along the polygon.
#[derive(Debug, Clone)]
pub struct CoastlinePolygon {
    /// CCW-ordered points in world coordinates.
    pub points: Vec<Vec2>,
}

/// Extracts all coastline polygons from the given map data.
/// Uses contour tracing to find edges between Land and Water tiles.
///
/// # Returns
/// A vector of `CoastlinePolygon`, each representing a closed loop.
/// Islands produce CCW loops; lakes (if traced) would produce CW loops.
pub fn extract_contours(map_data: &MapData, tile_size: f32) -> Vec<CoastlinePolygon> {
    let width = map_data.width as i32;
    let height = map_data.height as i32;
    
    // Track which edge segments we've already visited
    // Key: (tile_x, tile_y, edge_direction) where edge_direction is 0=N, 1=E, 2=S, 3=W
    let mut visited_edges: std::collections::HashSet<(i32, i32, u8)> = std::collections::HashSet::new();
    
    let mut polygons = Vec::new();
    
    // Scan for boundary edges (water tile adjacent to land tile)
    for y in 0..height {
        for x in 0..width {
            let tile = map_data.get(x as u32, y as u32).unwrap_or(TileType::DeepWater);
            
            // We trace from water tiles looking at land neighbors
            if !is_water(tile) {
                continue;
            }
            
            // Check each direction for a land neighbor
            for (dir, (dx, dy)) in [(0, (0, 1)), (1, (1, 0)), (2, (0, -1)), (3, (-1, 0))].iter() {
                let nx = x + dx;
                let ny = y + dy;
                
                // Check if neighbor is land (or out of bounds counts as land for border)
                let neighbor_is_land = if nx < 0 || ny < 0 || nx >= width || ny >= height {
                    true // Map border treated as land
                } else {
                    is_land(map_data.get(nx as u32, ny as u32).unwrap_or(TileType::DeepWater))
                };
                
                if neighbor_is_land && !visited_edges.contains(&(x, y, *dir)) {
                    // Start tracing a new contour from this edge
                    if let Some(polygon) = trace_contour(map_data, x, y, *dir, tile_size, &mut visited_edges) {
                        if polygon.points.len() >= 3 {
                            polygons.push(polygon);
                        }
                    }
                }
            }
        }
    }
    
    info!("Extracted {} coastline polygons", polygons.len());
    polygons
}

/// Traces a single contour starting from the given edge.
/// Returns a closed polygon if successful.
fn trace_contour(
    map_data: &MapData,
    start_x: i32,
    start_y: i32,
    start_dir: u8,
    tile_size: f32,
    visited: &mut std::collections::HashSet<(i32, i32, u8)>,
) -> Option<CoastlinePolygon> {
    let width = map_data.width as i32;
    let height = map_data.height as i32;
    
    // Calculate world offset to center the map
    let offset_x = -(width as f32 * tile_size) / 2.0;
    let offset_y = -(height as f32 * tile_size) / 2.0;
    
    let mut points = Vec::new();
    let mut x = start_x;
    let mut y = start_y;
    let mut dir = start_dir;
    
    // Direction vectors: 0=N(+Y), 1=E(+X), 2=S(-Y), 3=W(-X)
    let dir_vectors: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    
    // Edge midpoints relative to tile corner (0,0) for each direction
    // N edge: (0.5, 1.0), E edge: (1.0, 0.5), S edge: (0.5, 0.0), W edge: (0.0, 0.5)
    let edge_midpoints: [(f32, f32); 4] = [(0.5, 1.0), (1.0, 0.5), (0.5, 0.0), (0.0, 0.5)];
    
    let max_iterations = (width * height * 4) as usize; // Safety limit
    
    for _ in 0..max_iterations {
        // Mark this edge as visited
        visited.insert((x, y, dir));
        
        // Add the midpoint of this edge to our polygon
        let (mx, my) = edge_midpoints[dir as usize];
        let world_x = offset_x + (x as f32 + mx) * tile_size;
        let world_y = offset_y + (y as f32 + my) * tile_size;
        points.push(Vec2::new(world_x, world_y));
        
        // Find the next edge to follow (CCW traversal with land on left)
        // Try turning left first, then straight, then right
        let turn_order = [3, 0, 1, 2]; // Left, straight, right, back
        
        let mut found_next = false;
        for turn in turn_order {
            let next_dir = (dir + turn) % 4;
            let (dx, dy) = dir_vectors[next_dir as usize];
            let nx = x + dx;
            let ny = y + dy;
            
            // Check if this direction leads to land
            let neighbor_is_land = if nx < 0 || ny < 0 || nx >= width || ny >= height {
                true
            } else {
                is_land(map_data.get(nx as u32, ny as u32).unwrap_or(TileType::DeepWater))
            };
            
            if neighbor_is_land {
                // This edge is a coastline, continue tracing
                if !visited.contains(&(x, y, next_dir)) {
                    dir = next_dir;
                    found_next = true;
                    break;
                } else if x == start_x && y == start_y && next_dir == start_dir {
                    // We've completed the loop
                    return Some(CoastlinePolygon { points });
                }
            } else {
                // Move to the neighboring water tile and adjust direction
                x = nx;
                y = ny;
                // When entering a new tile, we came from the opposite direction
                dir = (next_dir + 2) % 4;
                
                // Find the next coastline edge in this tile
                for check_turn in turn_order {
                    let check_dir = (dir + check_turn) % 4;
                    let (cdx, cdy) = dir_vectors[check_dir as usize];
                    let cnx = x + cdx;
                    let cny = y + cdy;
                    
                    let check_is_land = if cnx < 0 || cny < 0 || cnx >= width || cny >= height {
                        true
                    } else {
                        is_land(map_data.get(cnx as u32, cny as u32).unwrap_or(TileType::DeepWater))
                    };
                    
                    if check_is_land && !visited.contains(&(x, y, check_dir)) {
                        dir = check_dir;
                        found_next = true;
                        break;
                    }
                }
                if found_next {
                    break;
                }
            }
        }
        
        if !found_next {
            // Check if we're back at start
            if x == start_x && y == start_y {
                return Some(CoastlinePolygon { points });
            }
            break;
        }
        
        // Check if we've returned to the start
        if x == start_x && y == start_y && dir == start_dir && points.len() > 2 {
            return Some(CoastlinePolygon { points });
        }
    }
    
    // Return partial polygon if we have enough points
    if points.len() >= 3 {
        Some(CoastlinePolygon { points })
    } else {
        None
    }
}

/// Returns true if the tile is considered "water" for coastline purposes.
fn is_water(tile: TileType) -> bool {
    matches!(tile, TileType::DeepWater | TileType::ShallowWater)
}

/// Returns true if the tile is considered "land" for coastline purposes.
fn is_land(tile: TileType) -> bool {
    matches!(tile, TileType::Land | TileType::Sand | TileType::Port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_island() {
        // Create a 5x5 map with a 3x3 island in the center
        let mut map = MapData::new(5, 5);
        
        // Fill center with land
        for y in 1..4 {
            for x in 1..4 {
                map.set(x, y, TileType::Land);
            }
        }
        
        let polygons = extract_contours(&map, 64.0);
        
        // Should find exactly one polygon (the island coastline)
        assert_eq!(polygons.len(), 1);
        
        // Polygon should have multiple points forming a closed loop
        assert!(polygons[0].points.len() >= 4);
    }
}
