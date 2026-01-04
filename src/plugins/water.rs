use bevy::prelude::*;
use crate::features::water::quadtree::OceanGridPlugin;
use crate::features::water::grid_adaptation::OceanGridAdaptationPlugin;
use crate::features::water::fluid_dynamics::FluidDynamicsPlugin;
use crate::features::water::coupling::OceanPhysicsCouplingPlugin;
use crate::features::water::render::OceanRenderPlugin;

pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            OceanGridPlugin,
            OceanGridAdaptationPlugin,
            FluidDynamicsPlugin,
            OceanPhysicsCouplingPlugin,
            OceanRenderPlugin,
            crate::features::water::debug::WaterDebugPlugin,
        ));
        
        info!("Water V3 Systems Initialized");
    }
}
