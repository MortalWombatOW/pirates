use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::features::water::quadtree::OceanQuadtree;
use crate::features::water::morton::morton_decode;
use crate::plugins::core::GameState;

#[derive(Default)]
pub struct WaterDebugPlugin;

impl Plugin for WaterDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterDebugConfig>()
           .add_systems(Update, (
               water_debug_ui,
               draw_velocity_vectors,
               update_material_debug_flags,
           ).run_if(in_state(GameState::Combat)));
    }
}

#[derive(Resource, Default, Debug)]
pub struct WaterDebugConfig {
    pub show_height: bool,
    pub show_velocity: bool,
    pub show_foam: bool,
}

fn water_debug_ui(
    mut contexts: EguiContexts,
    mut config: ResMut<WaterDebugConfig>,
) {
    egui::Window::new("Water Debug").show(contexts.ctx_mut(), |ui| {
        ui.checkbox(&mut config.show_height, "Show Height Map");
        ui.checkbox(&mut config.show_velocity, "Show Velocity Vectors");
        ui.checkbox(&mut config.show_foam, "Show Foam Map");
    });
}

fn draw_velocity_vectors(
    ocean: Res<OceanQuadtree>,
    config: Res<WaterDebugConfig>,
    mut gizmos: Gizmos,
) {
    if !config.show_velocity {
        return;
    }

    for (&(depth, code), cell) in ocean.nodes.iter() {
        // Only draw for significant flow to reduce clutter
        let speed_sq = cell.flow_right * cell.flow_right + cell.flow_down * cell.flow_down;
        if speed_sq < 0.01 {
            continue;
        }

        let (gx, gy) = morton_decode(code);
        let cell_size = ocean.cell_size(depth);
        let domain_size = ocean.domain_size;
        
        let grid_dim = 1u32 << depth;
        let normalized_x = gx as f32 / grid_dim as f32;
        let normalized_y = gy as f32 / grid_dim as f32;
        let half_size = domain_size / 2.0;
        let world_x = (normalized_x * domain_size) - half_size + (cell_size / 2.0);
        let world_y = (normalized_y * domain_size) - half_size + (cell_size / 2.0);
        let center = Vec2::new(world_x, world_y);

        let velocity = Vec2::new(cell.flow_right, cell.flow_down);
        // Scale vector for visibility
        let end = center + velocity * 0.5; 
        
        let color = if velocity.length() > 1.0 { Color::srgb(1.0, 0.0, 0.0) } else { Color::srgb(0.0, 0.0, 1.0) };
        
        gizmos.arrow_2d(center, end, color);
    }
}

use crate::features::water::render::{WaterMaterial, WaterMesh};

fn update_material_debug_flags(
    config: Res<WaterDebugConfig>,
    mut materials: ResMut<Assets<WaterMaterial>>,
    query: Query<&MeshMaterial2d<WaterMaterial>, With<WaterMesh>>,
) {
    if !config.is_changed() {
        return;
    }

    let mut flags = 0u32;
    if config.show_height { flags |= 1; }
    if config.show_foam { flags |= 2; }

    for handle in &query {
        if let Some(mat) = materials.get_mut(&handle.0) {
            mat.flags = flags;
        }
    }
}
