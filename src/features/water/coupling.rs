use bevy::prelude::*;
use avian2d::prelude::*;
use crate::components::ship::Ship;
use crate::features::water::quadtree::OceanQuadtree;
use crate::features::water::morton::{morton_decode, morton_encode};
use crate::plugins::core::GameState;

#[derive(Default)]
pub struct OceanPhysicsCouplingPlugin;

impl Plugin for OceanPhysicsCouplingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (apply_ship_displacement, apply_water_boudary_forces).run_if(in_state(GameState::Combat)));
    }
}

/// Ships displace water: Inject velocity into the grid based on ship movement.
/// Uses a "Dipole" model:
/// - Bow: Pushes water outwards and forwards.
/// - Stern: Pulls water inwards (wake filling).
fn apply_ship_displacement(
    mut ocean: ResMut<OceanQuadtree>,
    ships: Query<(&GlobalTransform, &LinearVelocity, &Collider), With<Ship>>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();
    if dt == 0.0 { return; }

    let domain_size = ocean.domain_size;

    for (transform, velocity, _collider) in ships.iter() {
        let ship_pos = transform.translation().truncate();
        let ship_vel = velocity.0;
        let speed_sq = ship_vel.length_squared();

        // Minimum speed to generate wake
        if speed_sq < 1.0 { continue; }
        
        let speed = speed_sq.sqrt();
        let forward = ship_vel / speed;
        // let right = Vec2::new(-forward.y, forward.x); // Unused in segment logic if we use dist_vec
        
        // Ship Dimensions (Approximation)
        let hull_length = 40.0;
        let half_length = hull_length / 2.0;
        let hull_width_influence = 12.0; // How far distinct hull displacement reaches
        
        let stern_pos = ship_pos - forward * half_length;
        let bow_pos = ship_pos + forward * half_length;
        let segment_vec = bow_pos - stern_pos;
        let segment_len_sq = segment_vec.length_squared();
        
        let interaction_strength = 0.5;

        for (&(depth, code), cell) in ocean.nodes.iter_mut() {
             let (gx, gy) = morton_decode(code);
             let cell_size = domain_size / (1u32 << depth) as f32;
             
             let grid_dim = 1u32 << depth;
             let normalized_x = gx as f32 / grid_dim as f32;
             let normalized_y = gy as f32 / grid_dim as f32;
             let half_size = domain_size / 2.0;
             let world_x = (normalized_x * domain_size) - half_size + (cell_size / 2.0);
             let world_y = (normalized_y * domain_size) - half_size + (cell_size / 2.0);
             let cell_center = Vec2::new(world_x, world_y);
             
             // Project cell onto hull segment
             let cell_to_stern = cell_center - stern_pos;
             // t: 0.0 = Stern, 1.0 = Bow
             let t = if segment_len_sq > 0.001 {
                 (cell_to_stern.dot(segment_vec) / segment_len_sq).clamp(0.0, 1.0)
             } else {
                 0.5
             };
             
             let closest_point = stern_pos + segment_vec * t;
             let dist_vec = cell_center - closest_point;
             let dist_sq = dist_vec.length_squared();
             
             if dist_sq < hull_width_influence * hull_width_influence {
                 let dist = dist_sq.sqrt();
                 let linear_falloff = 1.0 - (dist / hull_width_influence);
                 let falloff = linear_falloff * linear_falloff;
                 
                 // Direction away from hull centerline
                 let lateral_dir = dist_vec.normalize_or_zero();
                 
                 let force = if t > 0.85 {
                     // Bow: Strong Outward Push + Forward Component
                     (lateral_dir * 1.0 + forward * 0.5) * speed * interaction_strength
                 } else if t < 0.15 {
                     // Stern: Suction (Inward Pull) + Forward Drag
                     (-lateral_dir * 0.5 + forward * 0.2) * speed * interaction_strength * 0.8
                 } else {
                     // Midships: Pure displacement (Outward)
                     lateral_dir * speed * interaction_strength * 0.4
                 };
                 
                 // Apply
                 let dt_scale = 0.1;
                 cell.flow_right += force.x * falloff * dt_scale;
                 cell.flow_down += force.y * falloff * dt_scale;
             }
        }
    }
}

/// Water applies forces to ships (Drag / Drift).
fn apply_water_boudary_forces(
    ocean: Res<OceanQuadtree>,
    mut ships: Query<(&GlobalTransform, &LinearVelocity, &mut ExternalForce), With<Ship>>,
) {
    for (transform, velocity, mut force) in ships.iter_mut() {
        let ship_pos = transform.translation().truncate();
        
        let half_size = ocean.domain_size / 2.0;
        let norm_x = (ship_pos.x + half_size) / ocean.domain_size;
        let norm_y = (ship_pos.y + half_size) / ocean.domain_size;
        
        if norm_x < 0.0 || norm_x > 1.0 || norm_y < 0.0 || norm_y > 1.0 {
            continue;
        }
        
        let mut sample_flow = Vec2::ZERO;
        let mut found = false;
        
        for d in (0..=ocean.max_depth).rev() {
            let grid_dim = 1u32 << d;
            let gx = (norm_x * grid_dim as f32) as u16;
            let gy = (norm_y * grid_dim as f32) as u16;
            let code = morton_encode(gx, gy);
            
            if let Some(cell) = ocean.nodes.get(&(d, code)) {
                sample_flow = Vec2::new(cell.flow_right, cell.flow_down);
                found = true;
                break;
            }
        }
        
        if found {
            let drag_coeff = 1.0; 
            let rel_vel = sample_flow - velocity.0;
            let drag_force = rel_vel * drag_coeff;
            
            force.apply_force(drag_force);
        }
    }
}
