use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use crate::components::ship::Ship;
use crate::features::water::morton::{morton_decode, morton_encode};
use crate::features::water::quadtree::{OceanQuadtree, WaterCell};
use crate::plugins::core::GameState;

#[derive(Default)]
pub struct OceanGridAdaptationPlugin;

impl Plugin for OceanGridAdaptationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GridAdaptationConfig>()
            .add_systems(Startup, initialize_root)
            .add_systems(FixedUpdate, (grid_adaptation_system,).run_if(in_state(GameState::Combat)))
            .add_systems(Update, dynamic_resolution_system.run_if(in_state(GameState::Combat)));
    }
}

/// Configuration for the AMR grid adaptation.
#[derive(Resource, Debug)]
pub struct GridAdaptationConfig {
    /// Multiplier for cell size to determine split distance.
    pub split_threshold_multiplier: f32,
    /// Multiplier for cell size to determine merge distance.
    pub merge_threshold_multiplier: f32,
    /// Target FPS to maintain.
    pub target_fps: f64,
    /// Minimum quadtree depth (coarsest resolution).
    pub min_depth: u8,
    /// Maximum quadtree depth (finest resolution safety cap).
    pub max_depth_cap: u8,
}

impl Default for GridAdaptationConfig {
    fn default() -> Self {
        Self {
            split_threshold_multiplier: 1.5,
            merge_threshold_multiplier: 2.5,
            target_fps: 60.0,
            min_depth: 6,
            max_depth_cap: 16, // Safety cap requested by user
        }
    }
}

/// System to dynamically adjust simulation resolution based on FPS.
fn dynamic_resolution_system(
    mut ocean: ResMut<OceanQuadtree>,
    config: Res<GridAdaptationConfig>,
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    // Check every 1.0 second
    *timer += time.delta_secs();
    if *timer < 1.0 {
        return;
    }
    *timer = 0.0;

    if let Some(fps_diag) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps) = fps_diag.smoothed() {
            let current_depth = ocean.max_depth;
            
            // Decrease resolution (Depth - 1) if FPS is too low
            if fps < (config.target_fps - 5.0) {
                if current_depth > config.min_depth {
                    ocean.max_depth -= 1;
                    warn!("Low FPS ({:.1}). reducing water resolution to Depth {}", fps, ocean.max_depth);
                }
            } 
            // Increase resolution (Depth + 1) if FPS is high and stable
            else if fps > (config.target_fps + 10.0) {
                if current_depth < config.max_depth_cap {
                    ocean.max_depth += 1;
                    info!("High FPS ({:.1}). increasing water resolution to Depth {}", fps, ocean.max_depth);
                }
            }
        }
    }
}

/// Initialize the root node of the quadtree if empty.
fn initialize_root(mut ocean: ResMut<OceanQuadtree>) {
    if ocean.nodes.is_empty() {
        // Depth 0, Morton 0
        ocean.nodes.insert((0, 0), WaterCell::new(-10.0)); // Deep water default
        info!("Initialized Ocean Quadtree root.");
    }
}

/// Dynamic AMR system.
/// Splits nodes near ships.
/// Merges nodes far from ships.
fn grid_adaptation_system(
    mut ocean: ResMut<OceanQuadtree>,
    ships: Query<&GlobalTransform, With<Ship>>,
    config: Res<GridAdaptationConfig>,
) {
    if ocean.nodes.is_empty() {
        return;
    }

    // Parameters
    let split_threshold_multiplier = config.split_threshold_multiplier;
    let merge_threshold_multiplier = config.merge_threshold_multiplier;

    // Collect all ship positions (2D)
    let ship_positions: Vec<Vec2> = ships.iter().map(|t| t.translation().truncate()).collect();
    if ship_positions.is_empty() {
        return;
    }

    // 1. Identification Phase
    // Iterate all current nodes and decide if they should split or merge.
    // Since we modify the map, we collect actions first.
    
    let mut to_split = Vec::new();
    let mut to_merge_candidates = bevy::utils::HashSet::new(); // Parent keys
    
    for (&(depth, code), _cell) in ocean.nodes.iter() {
        let (gx, gy) = morton_decode(code);
        let cell_size = ocean.cell_size(depth);
        
        // Calculate world position of cell center
        // Grid coordinates (gx, gy) are in the range [0, 2^depth - 1]
        // World Space:
        // map (0,0) to (-domain_size/2, -domain_size/2) ? 
        // Or (0,0) is origin? Let's assume (0,0) is center of domain for now, 
        // mirroring typical quadtree. 
        // Actually, Morton usually maps [0, N] positive integers. 
        // Let's assume Grid Space [0, 2^depth] maps to World Space [-HalfSize, HalfSize].
        
        let grid_dim = 1u32 << depth;
        let normalized_x = gx as f32 / grid_dim as f32; // 0.0 to 1.0
        let normalized_y = gy as f32 / grid_dim as f32;
        
        // Map 0..1 to -Half..Half
        let half_size = ocean.domain_size / 2.0;
        let world_x = (normalized_x * ocean.domain_size) - half_size + (cell_size / 2.0);
        let world_y = (normalized_y * ocean.domain_size) - half_size + (cell_size / 2.0);
        let cell_center = Vec2::new(world_x, world_y);

        // Check distance to nearest ship
        let mut min_dist_sq = f32::MAX;
        for &pos in &ship_positions {
            min_dist_sq = min_dist_sq.min(cell_center.distance_squared(pos));
        }
        let min_dist = min_dist_sq.sqrt();

        // SPLIT Logic
        if depth < ocean.max_depth {
            if min_dist < cell_size * split_threshold_multiplier {
                to_split.push((depth, code));
                continue; // Cannot merge if we split
            }
        }

        // MERGE Logic
        if depth > 0 {
            if min_dist > cell_size * merge_threshold_multiplier {
                // If this node wants to merge, its parent is the candidate.
                // Parent code is simply code >> 2? No, Morton code composition is spread bits.
                // dx = gx / 2, dy = gy / 2.
                // parent_code = morton_encode(gx/2, gy/2).
                // Or simply: (depth - 1, parent_code)
                // We need to check if ALL 4 siblings are ready to merge.
                // We just mark the parent as a candidate. 
                // Later verify all 4 children exist and are merge-ready.
                
                let parent_x = gx / 2;
                let parent_y = gy / 2;
                let parent_code = morton_encode(parent_x, parent_y);
                to_merge_candidates.insert((depth - 1, parent_code));
            }
        }
    }

    // 2. Execution Phase: Split
    for (depth, code) in to_split {
        if let Some(parent_cell) = ocean.nodes.remove(&(depth, code)) {
            // Create 4 children
            let (gx, gy) = morton_decode(code);
            let next_depth = depth + 1;
            
            // Offsets for children: (2x, 2y), (2x+1, 2y), ...
            let child_coords = [
                (gx * 2, gy * 2),
                (gx * 2 + 1, gy * 2),
                (gx * 2, gy * 2 + 1),
                (gx * 2 + 1, gy * 2 + 1),
            ];

            for (cx, cy) in child_coords {
                let child_code = morton_encode(cx, cy);
                // Inherit state from parent (interpolated ideally, but copy for now)
                let child_cell = parent_cell.clone();
                // Conservation of mass / volume? 
                // Height is level, so it stays same. 
                // Volume would be height * area. Since area is 1/4, implicit volume is 1/4.
                ocean.nodes.insert((next_depth, child_code), child_cell);
            }
        }
    }

    // 3. Execution Phase: Merge
    // Valid merges require all 4 children to be present in 'ocean.nodes' 
    // AND all 4 children must effectively "want" to merge (implied by being in candidate set? 
    // No, strictly, we found candidates by looking at children.
    // Iterate candidates, check if all 4 children exist in 'ocean.nodes' (and are not split).
    // Note: If we just split a child, it's gone from nodes, so we won't merge.
    
    for (depth, code) in to_merge_candidates {
        let (gx, gy) = morton_decode(code);
        let child_depth = depth + 1;
        
        let child_keys = [
            (child_depth, morton_encode(gx * 2, gy * 2)),
            (child_depth, morton_encode(gx * 2 + 1, gy * 2)),
            (child_depth, morton_encode(gx * 2, gy * 2 + 1)),
            (child_depth, morton_encode(gx * 2 + 1, gy * 2 + 1)),
        ];

        let all_children_exist = child_keys.iter().all(|k| ocean.nodes.contains_key(k));
        
        if all_children_exist {
            // Check parent distance.
            
            let cell_size = ocean.cell_size(depth);
            // ... calculate parent center ...
            let grid_dim = 1u32 << depth;
            let normalized_x = gx as f32 / grid_dim as f32; 
            let normalized_y = gy as f32 / grid_dim as f32;
            let half_size = ocean.domain_size / 2.0;
            let world_x = (normalized_x * ocean.domain_size) - half_size + (cell_size / 2.0);
            let world_y = (normalized_y * ocean.domain_size) - half_size + (cell_size / 2.0);
            let cell_center = Vec2::new(world_x, world_y);
            
             let mut min_dist_sq = f32::MAX;
            for &pos in &ship_positions {
                min_dist_sq = min_dist_sq.min(cell_center.distance_squared(pos));
            }
            let min_dist = min_dist_sq.sqrt();
            
            // Use tighter threshold for parent check to ensure safety
            if min_dist > cell_size * merge_threshold_multiplier {
                 // MERGE!
                 // Average the state
                 let mut avg_height = 0.0;
                 let mut avg_flow_r = 0.0;
                 let mut avg_flow_d = 0.0;
                 let mut avg_bottom = 0.0;
                 
                 for k in &child_keys {
                     if let Some(c) = ocean.nodes.remove(k) {
                         avg_height += c.height;
                         avg_flow_r += c.flow_right;
                         avg_flow_d += c.flow_down;
                         avg_bottom += c.bottom;
                     }
                 }
                 avg_height /= 4.0;
                 avg_flow_r /= 4.0;
                 avg_flow_d /= 4.0;
                 avg_bottom /= 4.0;
                 
                 let parent_cell = WaterCell {
                     height: avg_height,
                     flow_right: avg_flow_r,
                     flow_down: avg_flow_d,
                     bottom: avg_bottom,
                 };
                 ocean.nodes.insert((depth, code), parent_cell);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::water::quadtree::OceanGridPlugin;
    use crate::plugins::core::GameState;

    #[test]
    fn test_initialization() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(OceanGridPlugin);
        app.add_plugins(OceanGridAdaptationPlugin);
        app.add_plugins(bevy::diagnostic::DiagnosticsPlugin);
        
        // Run startup systems
        app.update();
        
        let ocean = app.world().resource::<OceanQuadtree>();
        assert!(!ocean.nodes.is_empty());
        assert!(ocean.nodes.contains_key(&(0, 0)));
    }

    #[test]
    fn test_splitting() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(OceanGridPlugin);
        app.add_plugins(OceanGridAdaptationPlugin);
        app.add_plugins(bevy::diagnostic::DiagnosticsPlugin);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.insert_resource(State::new(GameState::Combat));
        
        // Configure domain size small enough that a ship splits it
        let mut ocean_setup = OceanQuadtree::default();
        ocean_setup.domain_size = 1000.0;
        app.insert_resource(ocean_setup);
        
        // Spawn a ship at center (0,0)
        app.world_mut().spawn((
            Ship,
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::from_xyz(0.0, 0.0, 0.0),
        ));
        
        // Run one frame (Startup + Update)
        app.update(); 
        
        // Run FixedUpdate
        app.world_mut().run_schedule(FixedUpdate);
        
        let ocean = app.world().resource::<OceanQuadtree>();
        
        // Thresholds: split < size * 1.5. 
        // Size at depth 0 = 1000. 0 < 1500. Split!
        
        assert!(!ocean.nodes.contains_key(&(0, 0)), "Root should be split");
        assert!(ocean.nodes.values().len() >= 4, "Should have at least 4 nodes");
        
        assert!(ocean.nodes.contains_key(&(1, 0)));
        assert!(ocean.nodes.contains_key(&(1, 1)));
        assert!(ocean.nodes.contains_key(&(1, 2)));
        assert!(ocean.nodes.contains_key(&(1, 3)));
    }
}
