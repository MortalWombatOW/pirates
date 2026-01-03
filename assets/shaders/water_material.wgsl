// Water Material Fragment Shader
// Visualizes fluid simulation velocity texture with a quantized blue-to-white palette

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// Settings uniform from WaterMaterial
struct WaterSettings {
    max_speed: f32,
    time: f32,
    quantize_steps: f32,
    _padding2: f32,
    color_deep: vec4<f32>,
    color_mid: vec4<f32>,
    color_light: vec4<f32>,
    color_foam: vec4<f32>,
}

@group(2) @binding(0) var<uniform> settings: WaterSettings;
@group(2) @binding(1) var velocity_texture: texture_2d<f32>;
@group(2) @binding(2) var velocity_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample velocity texture
    let velocity = textureSample(velocity_texture, velocity_sampler, in.uv).xy;
    
    // Calculate speed (magnitude of velocity)
    let speed = length(velocity);
    
    // Normalize speed to 0-1 range using max_speed
    let normalized_speed = clamp(speed / settings.max_speed, 0.0, 1.0);
    
    // N-band quantization
    let bands = settings.quantize_steps;
    // t goes from 0.0 to 1.0 in discrete steps
    let t = floor(normalized_speed * bands) / (bands - 1.0);
    
    // Interpolate between the 4 key colors based on t
    // 0.0 - 0.33: Deep -> Mid
    // 0.33 - 0.66: Mid -> Light
    // 0.66 - 1.0: Light -> Foam
    
    var color: vec4<f32>;
    
    if (t < 0.333) {
        let local_t = t / 0.333;
        color = mix(settings.color_deep, settings.color_mid, local_t);
    } else if (t < 0.666) {
        let local_t = (t - 0.333) / 0.333;
        color = mix(settings.color_mid, settings.color_light, local_t);
    } else {
        let local_t = (t - 0.666) / 0.334;
        color = mix(settings.color_light, settings.color_foam, local_t);
    }

    // Add subtle wave animation based on time
    let wave = sin(in.uv.x * 20.0 + settings.time * 2.0) * 0.02;
    let wave2 = sin(in.uv.y * 15.0 + settings.time * 1.5) * 0.02;
    let wave_offset = vec4<f32>(wave + wave2, wave + wave2, wave + wave2, 0.0);
    
    return color + wave_offset;
}
