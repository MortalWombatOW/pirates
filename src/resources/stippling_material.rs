use bevy::prelude::*;
use bevy::sprite::Material2d;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

/// Material for stippling effect.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct StipplingMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(0)]
    pub dot_spacing: f32,
    #[texture(1)]
    #[sampler(2)]
    pub depth_texture: Handle<Image>,
}

impl Material2d for StipplingMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/stippling.wgsl".into()
    }
}

impl Default for StipplingMaterial {
    fn default() -> Self {
        Self {
            color: LinearRgba::BLUE,
            dot_spacing: 1.0,
            depth_texture: Handle::default(),
        }
    }
}
