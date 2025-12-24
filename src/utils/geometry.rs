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
    let mut l_dir = start_dir; // Direction to the land tile
    
    // Direction vectors: 0=N(+Y), 1=E(+X), 2=S(-Y), 3=W(-X)
    let dir_vectors: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    
    // Edge midpoints relative to tile corner (0,0) for each direction
    // N edge: (0.5, 1.0), E edge: (1.0, 0.5), S edge: (0.5, 0.0), W edge: (0.0, 0.5)
    let edge_midpoints: [(f32, f32); 4] = [(0.5, 1.0), (1.0, 0.5), (0.5, 0.0), (0.0, 0.5)];
    
    let max_iterations = (width * height * 8) as usize; // Safety limit
    
    // Helper to check land
    let check_is_land = |tx: i32, ty: i32| -> bool {
        if tx < 0 || ty < 0 || tx >= width || ty >= height {
            true // Border is land
        } else {
            is_land(map_data.get(tx as u32, ty as u32).unwrap_or(TileType::DeepWater))
        }
    };

    for _ in 0..max_iterations {
        // Mark current edge as visited
        visited.insert((x, y, l_dir));
        
        // Add point
        let (mx, my) = edge_midpoints[l_dir as usize];
        let world_x = offset_x + (x as f32 + mx) * tile_size;
        let world_y = offset_y + (y as f32 + my) * tile_size;
        points.push(Vec2::new(world_x, world_y));

        // Determine next move based on neighbors
        // L = Vector to Land
        // F = Vector Forward (Right relative to L, i.e., CCW walk) -> (l_dir + 1) % 4
        
        let f_dir = (l_dir + 1) % 4;
        let (fdx, fdy) = dir_vectors[f_dir as usize];
        let (ldx, ldy) = dir_vectors[l_dir as usize];
        
        // Check 1: Inner/Convex Corner (Pivot)
        // Check `pos + F` is Land?
        let fx = x + fdx;
        let fy = y + fdy;
        
        if check_is_land(fx, fy) {
            // Blocked by land, must turn to follow it.
            // Stay in current tile, new Land direction is F.
            l_dir = f_dir;
            
            // Check loop closure
            if x == start_x && y == start_y && l_dir == start_dir {
                return Some(CoastlinePolygon { points });
            }
            continue;
        }
        
        // Check 2: Straight vs Outer Corner
        // Need to check the diagonal `pos + F + L`
        let diag_x = x + fdx + ldx;
        let diag_y = y + fdy + ldy;
        
        if check_is_land(diag_x, diag_y) {
            // Case 2: Straight Edge. 
            // `pos + F` is Water (from check 1), and `pos + F + L` is Land.
            // Move to `pos + F`. Land direction unchanged.
            x = fx;
            y = fy;
        } else {
            // Case 3: Outer/Concave Corner (Wrap).
            // Both `pos + F` and `pos + F + L` are Water.
            // We wrap around the corner of the land at `pos + L`.
            // Move to diagonal `pos + F + L`.
            x = diag_x;
            y = diag_y;
            // Land direction rotates -90 deg (Left) -> (l_dir + 3) % 4
            l_dir = (l_dir + 3) % 4;
        }
        
        // Check loop closure
        if x == start_x && y == start_y && l_dir == start_dir {
            return Some(CoastlinePolygon { points });
        }
    }
    
    // If not closed after max iterations, return what we have (shouldn't happen for closed topology)
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
    fn test_complex_island() {
        // Create a 10x10 map
        let mut map = MapData::new(10, 10);
        
        // Create a 'C' shaped island (concave)
        // Land at (3,3) to (3,7)
        for y in 3..=7 { map.set(3, y, TileType::Land); }
        // Top arm (4,7) to (6,7)
        for x in 4..=6 { map.set(x, 7, TileType::Land); }
        // Bottom arm (4,3) to (6,3)
        for x in 4..=6 { map.set(x, 3, TileType::Land); }
        
        // This shape has both convex (outer) and concave (inner) corners
        
        let polygons = extract_contours(&map, 64.0);
        
        // Should find 2 polygons:
        // 1. The Map Border (since border is treated as land)
        // 2. The Island coastline
        assert_eq!(polygons.len(), 2, "Should extract 2 polygons (Board + Island)");
        
        // Check island polygon point count
        // The island polygon should be the one with points within the map (not on border)
        // or simply check that we have a polygon with the expected complexity
        let island_poly = polygons.iter().find(|p| p.points.len() < 30).unwrap();
        assert!(island_poly.points.len() > 10);
    }
}
