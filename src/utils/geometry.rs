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
    
    // Edge midpoints relative to tile center (0,0) for each direction
    // N edge: (0.0, 0.5), E edge: (0.5, 0.0), S edge: (0.0, -0.5), W edge: (-0.5, 0.0)
    let edge_midpoints: [(f32, f32); 4] = [(0.0, 0.5), (0.5, 0.0), (0.0, -0.5), (-0.5, 0.0)];
    
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

use rand::Rng;

/// Number of subdivisions per segment (higher = smoother but more vertices)
pub const COASTLINE_SUBDIVISIONS: usize = 4;
/// Spline tension (0.0 = Catmull-Rom/Smooth, 1.0 = Linear/Sharp)
pub const COASTLINE_TENSION: f32 = 0.0;
/// Strength of random noise added to smoothed points (0.0 = no noise)
/// Represents max displacement in world units
pub const COASTLINE_JITTER_AMOUNT: f32 = 5.0;

/// Smooths a coastline polygon using Cardinal Splines.
/// 
/// Uses the constants `COASTLINE_SUBDIVISIONS`, `COASTLINE_TENSION`, and `COASTLINE_JITTER_AMOUNT` for configuration.
/// 
/// # Arguments
/// * `points` - The original closed loop of points.
/// 
/// Returns a new vector of points defining the smoothed loop.
pub fn smooth_coastline(points: &[Vec2]) -> Vec<Vec2> {
    if points.len() < 3 || COASTLINE_SUBDIVISIONS == 0 {
        return points.to_vec();
    }

    let mut smoothed = Vec::with_capacity(points.len() * COASTLINE_SUBDIVISIONS);
    let len = points.len();
    let mut rng = rand::thread_rng();

    // Cardinal Spline parameter s = (1 - t) / 2
    let s = (1.0 - COASTLINE_TENSION) / 2.0;

    // Iterate through all segments of the closed loop
    for i in 0..len {
        // We interpolate between P1 (i) and P2 (i+1)
        // using P0 (i-1) and P3 (i+2) as control points
        
        let p0 = points[(i + len - 1) % len];
        let p1 = points[i];
        let p2 = points[(i + 1) % len];
        let p3 = points[(i + 2) % len];

        for step in 0..COASTLINE_SUBDIVISIONS {
            let t = step as f32 / COASTLINE_SUBDIVISIONS as f32;
            let t2 = t * t;
            let t3 = t2 * t;

            // Identity: 1
            // t: s(p2 - p0)
            // t^2: 2sp0 + (s-3)p1 + (3-2s)p2 - sp3
            // t^3: -sp0 + (2-s)p1 + (s-2)p2 + sp3
            
            // Group coefficients by point:
            // P0: -s*t^3 + 2s*t^2 - s*t
            // P1: (2-s)*t^3 + (s-3)*t^2 + 1
            // P2: (s-2)*t^3 + (3-2s)*t^2 + s*t
            // P3: s*t^3 - s*t^2

            // Polynomial Basis:
            let h0 = -s * t3 + 2.0 * s * t2 - s * t; // Coeff for P0
            let h1 = (2.0 - s) * t3 + (s - 3.0) * t2 + 1.0; // Coeff for P1
            let h2 = (s - 2.0) * t3 + (3.0 - 2.0 * s) * t2 + s * t; // Coeff for P2
            let h3 = s * t3 - s * t2; // Coeff for P3
            
            let mut x = p0.x * h0 + p1.x * h1 + p2.x * h2 + p3.x * h3;
            let mut y = p0.y * h0 + p1.y * h1 + p2.y * h2 + p3.y * h3;
            
            // Apply jitter (noise)
            if COASTLINE_JITTER_AMOUNT > 0.0 {
                x += rng.gen_range(-COASTLINE_JITTER_AMOUNT..=COASTLINE_JITTER_AMOUNT);
                y += rng.gen_range(-COASTLINE_JITTER_AMOUNT..=COASTLINE_JITTER_AMOUNT);
            }
            
            smoothed.push(Vec2::new(x, y));
        }
    }

    smoothed
}

/// Computes the intersection point of two line segments if they cross.
/// Returns None if segments don't intersect or are collinear.
fn line_intersect(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> Option<Vec2> {
    let d1 = a2 - a1;
    let d2 = b2 - b1;
    
    let cross = d1.x * d2.y - d1.y * d2.x;
    
    // Parallel or collinear
    if cross.abs() < 1e-10 {
        return None;
    }
    
    let delta = b1 - a1;
    let t = (delta.x * d2.y - delta.y * d2.x) / cross;
    let u = (delta.x * d1.y - delta.y * d1.x) / cross;
    
    // Check if intersection is within both segments (exclusive of endpoints to avoid corner issues)
    if t > 0.001 && t < 0.999 && u > 0.001 && u < 0.999 {
        Some(a1 + d1 * t)
    } else {
        None
    }
}

/// Removes self-intersections from a polygon by detecting edge crossings
/// and keeping only the outer boundary.
/// 
/// Algorithm:
/// 1. Scan all edge pairs for intersections
/// 2. When an intersection is found, "skip" the looped section
/// 3. Continue building the result from the other side of the loop
fn remove_self_intersections(points: &[Vec2]) -> Vec<Vec2> {
    if points.len() < 4 {
        return points.to_vec();
    }

    let n = points.len();
    let mut result = Vec::new();
    let mut i = 0;
    let mut visited = vec![false; n];
    
    // Maximum iterations to prevent infinite loops
    let max_iter = n * 2;
    let mut iter_count = 0;
    
    while i < n && iter_count < max_iter {
        iter_count += 1;
        
        if visited[i] {
            i += 1;
            continue;
        }
        
        visited[i] = true;
        let a1 = points[i];
        let a2 = points[(i + 1) % n];
        
        // Check for intersection with all non-adjacent edges ahead
        let mut found_intersection = false;
        
        for j in (i + 2)..n {
            // Skip adjacent edges
            if j == (i + n - 1) % n {
                continue;
            }
            
            let b1 = points[j];
            let b2 = points[(j + 1) % n];
            
            if let Some(intersection) = line_intersect(a1, a2, b1, b2) {
                // Found intersection - add current point and intersection
                result.push(a1);
                result.push(intersection);
                
                // Skip the loop by jumping to edge j+1
                // Mark all skipped vertices as visited
                for k in (i + 1)..=j {
                    visited[k] = true;
                }
                
                i = (j + 1) % n;
                found_intersection = true;
                break;
            }
        }
        
        if !found_intersection {
            result.push(a1);
            i += 1;
        }
    }
    
    // Clean up duplicate/very close points
    if result.len() < 3 {
        return points.to_vec(); // Failed to simplify, return original
    }
    
    let mut cleaned = Vec::with_capacity(result.len());
    for (idx, p) in result.iter().enumerate() {
        let prev = if idx == 0 { result.last().unwrap() } else { &result[idx - 1] };
        if p.distance(*prev) > 0.1 {
            cleaned.push(*p);
        }
    }
    
    if cleaned.len() >= 3 {
        cleaned
    } else {
        points.to_vec()
    }
}

/// Offsets a polygon by a fixed distance outwards (to the right, assuming CCW winding).
/// 
/// # Arguments
/// * `points` - The CCW polygon points.
/// * `distance` - Distance to offset (positive = right/outwards, negative = left/inwards).
/// 
/// Returns a new polygon with self-intersections removed.
pub fn offset_polygon(points: &[Vec2], distance: f32) -> Vec<Vec2> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut offset_points = Vec::with_capacity(points.len());
    let len = points.len();

    for i in 0..len {
        // Calculate tangent at current point using neighbors
        let prev = points[(i + len - 1) % len];
        let next = points[(i + 1) % len];
        
        let tangent = (next - prev).normalize_or_zero();
        
        // Normal is tangent rotated 90 degrees CW (Right) for CCW polygons
        let normal = Vec2::new(tangent.y, -tangent.x);
        
        offset_points.push(points[i] + normal * distance);
    }

    // Remove any self-intersections caused by the offset
    remove_self_intersections(&offset_points)
}

use crate::resources::navmesh::{NavMeshResource, TieredNavMesh, ShoreBufferTier};
use spade::{Point2, Triangulation};

/// Builds NavMesh resources from coastline polygons for all shore buffer tiers.
///
/// # Arguments
/// * `polygons` - Coastline polygons with CCW winding (land on left)
/// * `map_bounds` - World-space bounds of the map (min_x, min_y, max_x, max_y)
///
/// # Returns
/// A NavMeshResource with meshes for each shore buffer tier.
pub fn build_navmesh_from_polygons(
    polygons: &[CoastlinePolygon],
    map_bounds: (f32, f32, f32, f32),
) -> NavMeshResource {
    let mut resource = NavMeshResource::new();
    
    for &tier in ShoreBufferTier::all() {
        let buffer = tier.buffer_distance();
        
        if let Some(mesh) = build_single_tier_navmesh(polygons, map_bounds, buffer, tier) {
            info!(
                "Built NavMesh for {:?} tier: {} vertices, {} triangles",
                tier, mesh.vertex_count, mesh.triangle_count
            );
            resource.set_mesh(tier, mesh);
        } else {
            warn!("Failed to build NavMesh for {:?} tier", tier);
        }
    }
    
    resource
}

/// Builds a single NavMesh for a specific shore buffer distance.
/// Uses Delaunay triangulation and filters triangles by centroid location.
fn build_single_tier_navmesh(
    polygons: &[CoastlinePolygon],
    map_bounds: (f32, f32, f32, f32),
    shore_buffer: f32,
    tier: ShoreBufferTier,
) -> Option<TieredNavMesh> {
    use spade::DelaunayTriangulation;
    
    let (min_x, min_y, max_x, max_y) = map_bounds;
    
    // Offset land polygons outward by shore buffer (they become obstacles)
    // Since coastlines are CCW with land on left, offsetting "outward" (positive distance)
    // expands the land area, which shrinks the navigable water.
    let obstacle_polygons: Vec<Vec<Vec2>> = polygons
        .iter()
        .filter_map(|poly| {
            if poly.points.len() < 3 {
                return None;
            }
            // Offset outward (positive = away from land = into water)
            let offset = offset_polygon(&poly.points, shore_buffer);
            if offset.len() >= 3 {
                Some(offset)
            } else {
                None
            }
        })
        .collect();
    
    // Build simple Delaunay triangulation (no constraints - more robust)
    let mut dt: DelaunayTriangulation<Point2<f64>> = DelaunayTriangulation::new();
    
    // Add boundary points with margin
    let margin = shore_buffer;
    let bounds_with_margin = [
        Vec2::new(min_x + margin, min_y + margin),
        Vec2::new(max_x - margin, min_y + margin),
        Vec2::new(max_x - margin, max_y - margin),
        Vec2::new(min_x + margin, max_y - margin),
    ];
    
    for p in &bounds_with_margin {
        let _ = dt.insert(Point2::new(p.x as f64, p.y as f64));
    }
    
    // Add intermediate points along map edges for better triangulation
    let edge_step = 500.0; // Add points every 500 world units along edges
    
    // Bottom edge
    let mut x = min_x + margin + edge_step;
    while x < max_x - margin {
        let _ = dt.insert(Point2::new(x as f64, (min_y + margin) as f64));
        x += edge_step;
    }
    // Top edge
    x = min_x + margin + edge_step;
    while x < max_x - margin {
        let _ = dt.insert(Point2::new(x as f64, (max_y - margin) as f64));
        x += edge_step;
    }
    // Left edge
    let mut y = min_y + margin + edge_step;
    while y < max_y - margin {
        let _ = dt.insert(Point2::new((min_x + margin) as f64, y as f64));
        y += edge_step;
    }
    // Right edge
    y = min_y + margin + edge_step;
    while y < max_y - margin {
        let _ = dt.insert(Point2::new((max_x - margin) as f64, y as f64));
        y += edge_step;
    }
    
    // Add obstacle polygon vertices (simplified - take every Nth point)
    let point_stride = 10; // Take every 10th point for performance
    for obstacle in &obstacle_polygons {
        for (i, p) in obstacle.iter().enumerate() {
            if i % point_stride != 0 {
                continue;
            }
            // Skip points outside bounds
            if p.x < min_x + margin || p.x > max_x - margin 
                || p.y < min_y + margin || p.y > max_y - margin {
                continue;
            }
            let _ = dt.insert(Point2::new(p.x as f64, p.y as f64));
        }
    }
    
    // Extract vertices
    let vertices: Vec<Vec2> = dt
        .vertices()
        .map(|v| {
            let p = v.position();
            Vec2::new(p.x as f32, p.y as f32)
        })
        .collect();
    
    if vertices.is_empty() {
        return None;
    }
    
    // Build vertex index lookup
    let vertex_lookup: std::collections::HashMap<_, _> = dt
        .vertices()
        .enumerate()
        .map(|(i, v)| (v.fix(), i))
        .collect();
    
    // Extract triangles, filtering out those inside obstacles
    let mut triangles: Vec<[usize; 3]> = Vec::new();
    
    for face in dt.inner_faces() {
        let verts = face.vertices();
        
        // Get vertex indices
        let Some(&i0) = vertex_lookup.get(&verts[0].fix()) else { continue };
        let Some(&i1) = vertex_lookup.get(&verts[1].fix()) else { continue };
        let Some(&i2) = vertex_lookup.get(&verts[2].fix()) else { continue };
        
        // Calculate triangle centroid
        let centroid = (vertices[i0] + vertices[i1] + vertices[i2]) / 3.0;
        
        // Check if centroid is inside any obstacle polygon (land)
        let inside_obstacle = obstacle_polygons
            .iter()
            .any(|obs| point_in_polygon(centroid, obs));
        
        // Also check if centroid is inside any ORIGINAL polygon (unoffset land)
        let inside_original_land = polygons
            .iter()
            .any(|poly| point_in_polygon(centroid, &poly.points));
        
        if !inside_obstacle && !inside_original_land {
            triangles.push([i0, i1, i2]);
        }
    }
    
    if triangles.is_empty() {
        return None;
    }
    
    TieredNavMesh::new(vertices, triangles, tier)
}

/// Tests if a point is inside a polygon using ray casting.
fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    
    let mut inside = false;
    let n = polygon.len();
    let mut j = n - 1;
    
    for i in 0..n {
        let pi = polygon[i];
        let pj = polygon[j];
        
        if ((pi.y > point.y) != (pj.y > point.y))
            && (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x)
        {
            inside = !inside;
        }
        j = i;
    }
    
    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catmull_rom_smoothing() {
        // Square
        let points = vec![
             Vec2::new(0.0, 0.0),
             Vec2::new(10.0, 0.0),
             Vec2::new(10.0, 10.0),
             Vec2::new(0.0, 10.0),
        ];

        let smoothed = smooth_coastline(&points);

        // Should have original_len * subdivisions points
        assert_eq!(smoothed.len(), 4 * COASTLINE_SUBDIVISIONS);
        
        // Check that points are roughly near the expected location
        // Allowing for JITTER_AMOUNT + float error
        let tolerance = COASTLINE_JITTER_AMOUNT + 0.1;
        
        let start = smoothed[0];
        assert!((start.x - points[0].x).abs() <= tolerance);
        assert!((start.y - points[0].y).abs() <= tolerance);
    }


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

    #[test]
    fn test_offset_polygon() {
        // Square CCW: (0,0) -> (10,0) -> (10,10) -> (0,10)
        let points = vec![
             Vec2::new(0.0, 0.0),
             Vec2::new(10.0, 0.0),
             Vec2::new(10.0, 10.0),
             Vec2::new(0.0, 10.0),
        ];

        let offset_dist = 1.0;
        let offset = offset_polygon(&points, offset_dist);

        assert_eq!(offset.len(), 4);

        // For a square, normals at corners are 45 degrees outwards if using average of adjacent edges?
        // Wait, the implementation uses:
        // tangent = (next - prev).normalize()
        // normal = (tangent.y, -tangent.x)
        
        // For point 0 (0,0): prev=(0,10), next=(10,0)
        // next-prev = (10, -10). normalize -> (0.707, -0.707)
        // normal -> (-0.707, -0.707) -> Pointing South-West. Correct for bottom-left corner.
        
        let p0 = offset[0];
        assert!(p0.x < 0.0);
        assert!(p0.y < 0.0);
        
        // For point 1 (10,0): prev=(0,0), next=(10,10)
        // next-prev = (10, 10). normalize -> (0.707, 0.707)
        // normal -> (0.707, -0.707) -> Pointing South-East. Correct.
        
        let p1 = offset[1];
        assert!(p1.x > 10.0);
        assert!(p1.y < 0.0);
    }
}
