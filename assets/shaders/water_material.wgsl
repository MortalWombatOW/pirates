// Water Material Fragment Shader
// Visualizes fluid simulation velocity texture with a quantized blue-to-white palette

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// Settings uniform from WaterMaterial
struct WaterSettings {
    max_speed: f32,
    time: f32,
    _padding1: f32,
    _padding2: f32,
}

@group(2) @binding(0) var<uniform> settings: WaterSettings;
@group(2) @binding(1) var velocity_texture: texture_2d<f32>;
@group(2) @binding(2) var velocity_sampler: sampler;

// Quantized color palette (4 bands)
const COLOR_DEEP: vec3<f32> = vec3<f32>(0.05, 0.15, 0.35);      // Deep blue
const COLOR_MID: vec3<f32> = vec3<f32>(0.1, 0.3, 0.55);         // Medium blue
const COLOR_LIGHT: vec3<f32> = vec3<f32>(0.3, 0.5, 0.7);        // Light blue
const COLOR_FOAM: vec3<f32> = vec3<f32>(0.85, 0.9, 0.95);       // Foam white

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample velocity texture
    let velocity = textureSample(velocity_texture, velocity_sampler, in.uv).xy;
    
    // Calculate speed (magnitude of velocity)
    let speed = length(velocity);
    
    // Normalize speed to 0-1 range using max_speed
    let normalized_speed = clamp(speed / settings.max_speed, 0.0, 1.0);
    
    // Quantize into 4 bands for stylized look
    var color: vec3<f32>;
    if (normalized_speed < 0.25) {
        color = COLOR_DEEP;
    } else if (normalized_speed < 0.5) {
        color = COLOR_MID;
    } else if (normalized_speed < 0.75) {
        color = COLOR_LIGHT;
    } else {
        color = COLOR_FOAM;
    }
    
    // Add subtle wave animation based on time
    let wave = sin(in.uv.x * 20.0 + settings.time * 2.0) * 0.02;
    let wave2 = sin(in.uv.y * 15.0 + settings.time * 1.5) * 0.02;
    color = color + vec3<f32>(wave + wave2);
    
    return vec4<f32>(color, 1.0);
}
