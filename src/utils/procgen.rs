//! Procedural generation utilities for world map creation.
//!
//! Uses the `noise` crate to generate natural-looking terrain with
//! landmasses, coastlines, and ports.

use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use crate::resources::{MapData, TileType};

/// Configuration for procedural map generation.
pub struct MapGenConfig {
    /// Random seed for reproducible generation
    pub seed: u32,
    /// Map width in tiles
    pub width: u32,
    /// Map height in tiles
    pub height: u32,
    /// Noise frequency (lower = larger landmasses)
    pub frequency: f64,
    /// Number of noise octaves for detail
    pub octaves: usize,
    /// Minimum number of ports to generate
    pub min_ports: usize,
    /// Maximum number of ports to generate
    pub max_ports: usize,
}

impl Default for MapGenConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            width: 512,
            height: 512,
            frequency: 0.015, // Lower frequency = larger islands
            octaves: 6,
            min_ports: 8,
            max_ports: 15,
        }
    }
}

/// Generates a procedural world map using multi-octave fractal noise.
///
/// The generation process:
/// 1. Uses Fbm (Fractal Brownian Motion) noise for natural landmasses
/// 2. Applies radial gradient to create ocean at map edges
/// 3. Maps noise values to tile types via thresholds
/// 4. Ensures center spawn area is navigable
/// 5. Places ports on coastlines
///
/// # Arguments
/// * `config` - Generation configuration parameters
///
/// # Returns
/// A populated `MapData` resource ready for tilemap rendering.
pub fn generate_world_map(config: MapGenConfig) -> MapData {
    let mut map_data = MapData::new(config.width, config.height);

    // Create Fbm noise for natural-looking terrain
    let fbm: Fbm<Perlin> = Fbm::new(config.seed)
        .set_frequency(config.frequency)
        .set_octaves(config.octaves);

    let center_x = config.width as f64 / 2.0;
    let center_y = config.height as f64 / 2.0;
    let max_dist = (center_x.min(center_y)) * 0.85; // Falloff starts at 85% from center

    // First pass: Generate base terrain
    for y in 0..config.height {
        for x in 0..config.width {
            let nx = x as f64;
            let ny = y as f64;

            // Sample noise (returns -1.0 to 1.0)
            let noise_value = fbm.get([nx, ny]);

            // Apply radial gradient to push edges toward ocean
            let dx = nx - center_x;
            let dy = ny - center_y;
            let dist = (dx * dx + dy * dy).sqrt();
            let gradient = if dist > max_dist {
                // Fade to ocean at edges
                -((dist - max_dist) / (center_x - max_dist)).min(1.0) * 0.5
            } else {
                0.0
            };

            let final_value = noise_value + gradient;

            // Map noise to tile types
            let tile_type = noise_to_tile(final_value);
            map_data.set(x, y, tile_type);
        }
    }

    // Second pass: Remove lakes (enforce single contiguous ocean)
    fill_lakes(&mut map_data);

    // Third pass: Ensure spawn area (center) is navigable
    ensure_spawn_navigable(&mut map_data);

    // Fourth pass: Add shallow water transitions
    add_coastal_transitions(&mut map_data);

    // Fourth pass: Place ports on coastlines
    place_ports(&mut map_data, config.min_ports, config.max_ports, config.seed);

    bevy::log::info!(
        "Generated procedural map: {}x{} tiles, seed: {}",
        config.width,
        config.height,
        config.seed
    );

    map_data
}

/// Maps a noise value to a tile type.
/// Thresholds are tuned for archipelago-style maps with ~30% land coverage.
fn noise_to_tile(value: f64) -> TileType {
    if value < -0.1 {
        TileType::DeepWater
    } else if value < 0.05 {
        TileType::ShallowWater
    } else if value < 0.12 {
        TileType::Sand
    } else {
        TileType::Land
    }
}

/// Removes landlocked water bodies ("lakes") by flood-filling from map edges.
/// Any navigable tile not reachable from the ocean perimeter is converted to land.
fn fill_lakes(map_data: &mut MapData) {
    use std::collections::VecDeque;

    let width = map_data.width;
    let height = map_data.height;

    // Visited grid for BFS
    let mut visited = vec![false; (width * height) as usize];
    let mut queue = VecDeque::new();

    // Seed BFS from all edge tiles that are navigable
    for x in 0..width {
        for y in [0, height - 1] {
            if map_data.is_navigable(x, y) {
                let idx = (y * width + x) as usize;
                if !visited[idx] {
                    visited[idx] = true;
                    queue.push_back((x, y));
                }
            }
        }
    }
    for y in 0..height {
        for x in [0, width - 1] {
            if map_data.is_navigable(x, y) {
                let idx = (y * width + x) as usize;
                if !visited[idx] {
                    visited[idx] = true;
                    queue.push_back((x, y));
                }
            }
        }
    }

    // BFS to mark all reachable ocean tiles
    while let Some((x, y)) = queue.pop_front() {
        for (nx, ny) in neighbors_4(x, y, width, height) {
            let idx = (ny * width + nx) as usize;
            if !visited[idx] && map_data.is_navigable(nx, ny) {
                visited[idx] = true;
                queue.push_back((nx, ny));
            }
        }
    }

    // Convert unreachable navigable tiles (lakes) to land
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if !visited[idx] && map_data.is_navigable(x, y) {
                map_data.set(x, y, TileType::Land);
            }
        }
    }
}

/// Ensures the center spawn area (16x16 tiles) is navigable water.
fn ensure_spawn_navigable(map_data: &mut MapData) {
    let center_x = map_data.width / 2;
    let center_y = map_data.height / 2;
    let spawn_radius = 8;

    for y in (center_y.saturating_sub(spawn_radius))..=(center_y + spawn_radius).min(map_data.height - 1) {
        for x in (center_x.saturating_sub(spawn_radius))..=(center_x + spawn_radius).min(map_data.width - 1) {
            let dx = (x as i32 - center_x as i32).abs();
            let dy = (y as i32 - center_y as i32).abs();
            
            // Circular spawn area
            if dx * dx + dy * dy <= (spawn_radius as i32 * spawn_radius as i32) {
                if let Some(tile) = map_data.get(x, y) {
                    if !tile.is_navigable() {
                        map_data.set(x, y, TileType::DeepWater);
                    }
                }
            }
        }
    }
}

/// Adds shallow water transitions around coastlines for visual polish.
fn add_coastal_transitions(map_data: &mut MapData) {
    let width = map_data.width;
    let height = map_data.height;

    // Collect tiles that need transition (can't modify while iterating)
    let mut transitions: Vec<(u32, u32)> = Vec::new();

    for y in 0..height {
        for x in 0..width {
            if let Some(TileType::DeepWater) = map_data.get(x, y) {
                // Check if adjacent to land or sand
                let has_land_neighbor = neighbors_4(x, y, width, height)
                    .iter()
                    .any(|&(nx, ny)| {
                        matches!(
                            map_data.get(nx, ny),
                            Some(TileType::Land) | Some(TileType::Sand)
                        )
                    });

                if has_land_neighbor {
                    transitions.push((x, y));
                }
            }
        }
    }

    for (x, y) in transitions {
        map_data.set(x, y, TileType::ShallowWater);
    }
}

/// Places ports on valid coastline locations.
fn place_ports(map_data: &mut MapData, min_ports: usize, max_ports: usize, seed: u32) {
    use rand::prelude::*;
    
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed as u64);
    let width = map_data.width;
    let height = map_data.height;

    // Find valid port locations (sand tiles adjacent to both land and water)
    let mut candidates: Vec<(u32, u32)> = Vec::new();

    for y in 0..height {
        for x in 0..width {
            if let Some(TileType::Sand) = map_data.get(x, y) {
                let neighbors = neighbors_4(x, y, width, height);
                
                let has_land = neighbors.iter().any(|&(nx, ny)| {
                    matches!(map_data.get(nx, ny), Some(TileType::Land))
                });
                
                let has_water = neighbors.iter().any(|&(nx, ny)| {
                    matches!(
                        map_data.get(nx, ny),
                        Some(TileType::DeepWater) | Some(TileType::ShallowWater)
                    )
                });

                if has_land && has_water {
                    candidates.push((x, y));
                }
            }
        }
    }

    if candidates.is_empty() {
        bevy::log::warn!("No valid port locations found!");
        return;
    }

    // Shuffle candidates
    candidates.shuffle(&mut rng);

    // Determine number of ports to place
    let num_ports = rng.gen_range(min_ports..=max_ports).min(candidates.len());

    // Place ports with minimum spacing
    let min_spacing: u32 = 50; // Minimum tiles between ports
    let mut placed_ports: Vec<(u32, u32)> = Vec::new();

    for (x, y) in candidates {
        // Check spacing from existing ports
        let too_close = placed_ports.iter().any(|&(px, py)| {
            let dx = (x as i32 - px as i32).unsigned_abs();
            let dy = (y as i32 - py as i32).unsigned_abs();
            dx + dy < min_spacing
        });

        if !too_close {
            map_data.set(x, y, TileType::Port);
            placed_ports.push((x, y));

            if placed_ports.len() >= num_ports {
                break;
            }
        }
    }

    bevy::log::info!("Placed {} ports on the map", placed_ports.len());
}

/// Returns the 4-directional neighbors of a tile (N, S, E, W).
fn neighbors_4(x: u32, y: u32, width: u32, height: u32) -> Vec<(u32, u32)> {
    let mut result = Vec::with_capacity(4);

    if x > 0 {
        result.push((x - 1, y));
    }
    if x < width - 1 {
        result.push((x + 1, y));
    }
    if y > 0 {
        result.push((x, y - 1));
    }
    if y < height - 1 {
        result.push((x, y + 1));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_map_produces_valid_data() {
        let config = MapGenConfig {
            width: 64,
            height: 64,
            ..Default::default()
        };
        let map = generate_world_map(config);
        
        assert_eq!(map.width, 64);
        assert_eq!(map.height, 64);
    }

    #[test]
    fn test_spawn_area_is_navigable() {
        let config = MapGenConfig {
            width: 64,
            height: 64,
            ..Default::default()
        };
        let map = generate_world_map(config);
        
        // Center should be navigable
        let center_x = 32;
        let center_y = 32;
        assert!(map.is_navigable(center_x, center_y));
    }

    #[test]
    fn test_same_seed_produces_same_map() {
        let config1 = MapGenConfig {
            seed: 12345,
            width: 64,
            height: 64,
            ..Default::default()
        };
        let config2 = MapGenConfig {
            seed: 12345,
            width: 64,
            height: 64,
            ..Default::default()
        };
        
        let map1 = generate_world_map(config1);
        let map2 = generate_world_map(config2);

        // Compare a sample of tiles
        for y in 0..64 {
            for x in 0..64 {
                assert_eq!(map1.get(x, y), map2.get(x, y));
            }
        }
    }
}
