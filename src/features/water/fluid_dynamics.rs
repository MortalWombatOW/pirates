use bevy::prelude::*;
use crate::features::water::morton::{morton_decode, morton_encode};
use crate::features::water::quadtree::{OceanQuadtree, WaterCell};

use crate::plugins::core::GameState;

#[derive(Default)]
pub struct FluidDynamicsPlugin;

impl Plugin for FluidDynamicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FluidConfig>()
           .add_systems(FixedUpdate, (fluid_solver_system,).run_if(in_state(GameState::Combat)));
    }
}

/// Configuration for the fluid dynamics solver.
#[derive(Resource, Debug)]
pub struct FluidConfig {
    /// Gravity constant (m/s^2). Affects wave speed.
    pub gravity: f32,
    /// Damping factor (0.0 to 1.0). Higher means waves decay faster.
    pub damping: f32,
    /// Nominal depth of the water (m). Affects wave speed (c = sqrt(g*h)).
    pub base_depth: f32,
}

impl Default for FluidConfig {
    fn default() -> Self {
        Self {
            gravity: 9.81,
            damping: 1.0, 
            base_depth: 10.0,
        }
    }
}

/// Solves Shallow Water Equations on the Quadtree using a Staggered Grid (Arakawa C-grid).
/// Variables:
/// - h: Defined at cell center.
/// - flow_right (u): Defined at East face center.
/// - flow_down (v): Defined at South face center.
fn fluid_solver_system(
    mut ocean: ResMut<OceanQuadtree>,
    config: Res<FluidConfig>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();
    let gravity = config.gravity;
    let damping = config.damping; 
    let base_depth = config.base_depth;

    // Arrays to store updates
    let mut flow_r_deltas: bevy::utils::HashMap<(u8, u32), f32> = bevy::utils::HashMap::default();
    let mut flow_d_deltas: bevy::utils::HashMap<(u8, u32), f32> = bevy::utils::HashMap::default();
    let mut height_deltas: bevy::utils::HashMap<(u8, u32), f32> = bevy::utils::HashMap::default();

    // Helper: Get height of neighbor at (D, X, Y). 
    // If not found, check parent.
    // If still not found, return 0.0 (Sea Level / Open Boundary).
    let get_h = |nodes: &bevy::utils::HashMap<(u8, u32), WaterCell>, depth: u8, x: u16, y: u16| -> f32 {
        let code = morton_encode(x, y);
        if let Some(cell) = nodes.get(&(depth, code)) {
            return cell.height;
        }
        if depth > 0 {
             let p_code = morton_encode(x / 2, y / 2);
             if let Some(cell) = nodes.get(&(depth - 1, p_code)) {
                 return cell.height;
             }
        }
        0.0
    };

    // Helper: Get flow_right of neighbor at (D, X, Y).
    let get_flow_r = |nodes: &bevy::utils::HashMap<(u8, u32), WaterCell>, depth: u8, x: u16, y: u16| -> f32 {
        let code = morton_encode(x, y);
        if let Some(cell) = nodes.get(&(depth, code)) {
            return cell.flow_right;
        }
        if depth > 0 {
             let p_code = morton_encode(x / 2, y / 2);
             if let Some(cell) = nodes.get(&(depth - 1, p_code)) {
                 // Return parent flow scaled? 
                 // Parent flow covers 2 children faces.
                 // Assume uniform distribution: parent flow.
                 return cell.flow_right;
             }
        }
        0.0
    };
    
    // Helper: Get flow_down of neighbor at (D, X, Y).
    let get_flow_d = |nodes: &bevy::utils::HashMap<(u8, u32), WaterCell>, depth: u8, x: u16, y: u16| -> f32 {
        let code = morton_encode(x, y);
        if let Some(cell) = nodes.get(&(depth, code)) {
            return cell.flow_down;
        }
        if depth > 0 {
             let p_code = morton_encode(x / 2, y / 2);
             if let Some(cell) = nodes.get(&(depth - 1, p_code)) {
                 return cell.flow_down;
             }
        }
        0.0
    };

    // 1. Momentum Pass: Update u (flow_right) and v (flow_down)
    for (&(depth, code), cell) in ocean.nodes.iter() {
        let (gx, gy) = morton_decode(code);
        let cell_size = ocean.cell_size(depth);
        
        let h_self = cell.height;
        
        // Update flow_right (East Face)
        // Driven by Pressure Gradient between Self and East Neighbor
        let h_east = get_h(&ocean.nodes, depth, gx.wrapping_add(1), gy);
        
        let grad_x = (h_east - h_self) / cell_size; 
        // Note: Standard Staggered Grid: u(i+1/2) depends on h(i+1) - h(i).
        // Here: h_east is h(i+1). h_self is h(i). Correct.
        
        let du = -gravity * grad_x * dt;
        let damping_factor = (1.0 - damping * dt).max(0.0);
        
        let new_flow_r = (cell.flow_right + du) * damping_factor;
        flow_r_deltas.insert((depth, code), new_flow_r);
        
        // Update flow_down (South Face)
        // Driven by h_south - h_self
        let h_south = get_h(&ocean.nodes, depth, gx, gy.wrapping_add(1));
        let grad_y = (h_south - h_self) / cell_size;
        
        let dv = -gravity * grad_y * dt;
        let new_flow_d = (cell.flow_down + dv) * damping_factor;
        flow_d_deltas.insert((depth, code), new_flow_d);
    }
    
    // Apply Momentum immediately (Step 1.5) so Step 2 uses updated velocities?
    // Semi-implicit Euler uses old Velocities for Height? Or New?
    // Usually: Update U, then use New U to update H.
    // Let's modify ocean.nodes in place via the buffer.
    
    for (k, v) in &flow_r_deltas {
        if let Some(cell) = ocean.nodes.get_mut(k) {
            cell.flow_right = *v;
        }
    }
    for (k, v) in &flow_d_deltas {
        if let Some(cell) = ocean.nodes.get_mut(k) {
            cell.flow_down = *v;
        }
    }

    // 2. Mass Pass: Update h
    // Driven by Divergence of Flow
    // div = (u_right - u_left + v_down - v_up) / dx
    
    for (&(depth, code), cell) in ocean.nodes.iter() {
        let (gx, gy) = morton_decode(code);
        let cell_size = ocean.cell_size(depth);
        
        let u_right = cell.flow_right;
        let v_down = cell.flow_down;
        
        // u_left is the flow_right of the West neighbor
        let u_left = get_flow_r(&ocean.nodes, depth, gx.wrapping_sub(1), gy);
        
        // v_up is the flow_down of the North neighbor
        let v_up = get_flow_d(&ocean.nodes, depth, gx, gy.wrapping_sub(1));
        
        // Divergence
        // Net Outflow = (u_right - u_left) + (v_down - v_up)
        // If > 0, we lose mass.
        let div = (u_right - u_left) + (v_down - v_up);
        
        // h -= div * (Depth / CellSize) * dt
        // Flow is velocity? No, I stored velocity in flow_right? Yes.
        // So Flux = Vel * CrossSectionArea approx?
        // Actually SWE: dh/dt = -H * div(u).
        // My 'div' is sum of velocities? No, u/v are velocities.
        // div operator usually includes / dx.
        
        let divergence = div / cell_size;
        let dh = -base_depth * divergence * dt;
        
        height_deltas.insert((depth, code), dh);
    }
    
    // Apply Height
    for (k, dh) in height_deltas {
        if let Some(cell) = ocean.nodes.get_mut(&k) {
            cell.height += dh;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::water::quadtree::OceanGridPlugin;
    
    #[test]
    fn test_wave_propagation() {
        let mut app = App::new();
        app.add_plugins(OceanGridPlugin);
        app.add_plugins(FluidDynamicsPlugin);
        app.add_plugins(bevy::time::TimePlugin);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        
        // Transition to Combat state
        app.insert_resource(State::new(GameState::Combat));
        
        let mut ocean = OceanQuadtree::default();
        ocean.domain_size = 100.0;
        
        // Setup: Left (10.0), Right (0.0).
        ocean.nodes.insert((1, 0), WaterCell { height: 10.0, ..default() });
        ocean.nodes.insert((1, 1), WaterCell { height: 0.0, ..default() });
        
        app.insert_resource(ocean);
        
        // Manually step fixed time to ensure non-zero dt
        let mut fixed_time = Time::<Fixed>::default();
        fixed_time.advance_by(std::time::Duration::from_secs_f32(0.1));
        app.insert_resource(fixed_time);
        
        app.update();
        app.world_mut().run_schedule(FixedUpdate);
        
        let ocean = app.world().resource::<OceanQuadtree>();
        let cell_left = ocean.nodes.get(&(1, 0)).unwrap();
        let cell_right = ocean.nodes.get(&(1, 1)).unwrap();
        
        // Check Flow
        // Left Cell: Height=10. East Neighbor=0.
        // Grad = (0 - 10) / dx = negative.
        // u_right += -g * neg = positive.
        // Flow should be to the right.
        
        assert!(cell_left.flow_right > 0.0, "Flow should be positive (Right)");
        
        // Check Height Change
        // Left Cell: u_right > 0. u_left (West) = 0.
        // div = u_right - u_left > 0.
        // dh = -H * div < 0.
        // Height should decrease.
        
        assert!(cell_left.height < 10.0, "Left height should decrease");
        
        // Right Cell: u_right (Self) ?
        // h_self=0. h_east (Neighbor) = 0 (Open Boundary).
        // u_right should stay ~0.
        // u_left (West Neighbor) > 0.
        // div = u_right - u_left < 0.
        // dh = -H * div > 0.
        // Height should increase.
        
        assert!(cell_right.height > 0.0, "Right height should increase");
    }
}
