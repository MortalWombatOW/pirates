// Water Surface Material Shader
// Samples the fluid simulation velocity texture and visualizes it
// with a quantized blue-to-white color palette

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// Uniform data
struct WaterSettings {
    // Maximum velocity for color mapping
    max_speed: f32,
    // Time for animated effects
    time: f32,
    // Padding for alignment
    _padding1: f32,
    _padding2: f32,
}

@group(2) @binding(0) var<uniform> settings: WaterSettings;
@group(2) @binding(1) var velocity_texture: texture_2d<f32>;
@group(2) @binding(2) var velocity_sampler: sampler;

// Color palette (deep blue to white foam)
const DEEP_BLUE: vec4<f32> = vec4<f32>(0.05, 0.15, 0.35, 1.0);
const MID_BLUE: vec4<f32> = vec4<f32>(0.15, 0.35, 0.55, 1.0);
const LIGHT_BLUE: vec4<f32> = vec4<f32>(0.4, 0.6, 0.75, 1.0);
const FOAM_WHITE: vec4<f32> = vec4<f32>(0.85, 0.9, 0.95, 1.0);

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample velocity from the simulation texture
    let velocity = textureSample(velocity_texture, velocity_sampler, in.uv).xy;
    
    // Calculate speed (magnitude of velocity)
    let speed = length(velocity);
    
    // Normalize speed to 0-1 range based on max_speed
    let t = clamp(speed / settings.max_speed, 0.0, 1.0);
    
    // Quantize to 4 color bands (as per design spec)
    let band = floor(t * 4.0) / 4.0;
    
    // Select base color based on band
    var base_color: vec4<f32>;
    if (band < 0.25) {
        base_color = DEEP_BLUE;
    } else if (band < 0.5) {
        base_color = MID_BLUE;
    } else if (band < 0.75) {
        base_color = LIGHT_BLUE;
    } else {
        base_color = FOAM_WHITE;
    }
    
    // Add subtle wave animation
    let wave = sin(in.uv.x * 20.0 + settings.time * 0.5) * 
               sin(in.uv.y * 20.0 + settings.time * 0.3) * 0.02;
    
    // Apply wave to color (create new vec4 to avoid assignment issues)
    let final_color = vec4<f32>(
        base_color.r + wave,
        base_color.g + wave,
        base_color.b + wave,
        base_color.a
    );
    
    return final_color;
}
