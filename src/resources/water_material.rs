//! Water Material for Combat view water surface.
//!
//! Uses Material2d to render the fluid simulation velocity texture with
//! a quantized blue-to-white color palette.

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};
use bevy::sprite::Material2d;

/// Settings for water rendering, passed to the shader as a uniform.
#[derive(Clone, Copy, Debug, ShaderType)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct WaterSettings {
    /// Maximum velocity for color mapping
    pub max_speed: f32,
    /// Time for animated effects
    pub time: f32,
    /// Padding for alignment
    pub _padding1: f32,
    pub _padding2: f32,
}

impl Default for WaterSettings {
    fn default() -> Self {
        Self {
            max_speed: 100.0, // Tune based on simulation output
            time: 0.0,
            _padding1: 0.0,
            _padding2: 0.0,
        }
    }
}

/// Material for rendering the water surface in Combat view.
/// Samples the velocity texture from the fluid simulation and
/// visualizes it with a quantized blue-to-white palette.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaterMaterial {
    /// Water settings uniform (max speed, time)
    #[uniform(0)]
    pub settings: WaterSettings,

    /// Velocity texture from fluid simulation
    #[texture(1)]
    #[sampler(2)]
    pub velocity_texture: Handle<Image>,
}

impl Material2d for WaterMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/water_material.wgsl".into()
    }
}

impl Default for WaterMaterial {
    fn default() -> Self {
        Self {
            settings: WaterSettings::default(),
            velocity_texture: Handle::default(),
        }
    }
}
