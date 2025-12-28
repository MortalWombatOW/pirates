// Water Surface Material Shader
// Samples the fluid simulation velocity texture and visualizes it
// with a rich oceanic color palette and smooth wave animations

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

// Rich oceanic color palette - from deep ocean to sun-dappled shallows
const DEEP_OCEAN: vec4<f32> = vec4<f32>(0.02, 0.08, 0.18, 1.0);      // Near-black deep blue
const OCEAN_BLUE: vec4<f32> = vec4<f32>(0.04, 0.14, 0.28, 1.0);      // Deep sea blue
const MID_BLUE: vec4<f32> = vec4<f32>(0.08, 0.22, 0.38, 1.0);        // Mediterranean blue
const TEAL: vec4<f32> = vec4<f32>(0.12, 0.32, 0.45, 1.0);            // Teal-blue
const AQUA: vec4<f32> = vec4<f32>(0.18, 0.42, 0.52, 1.0);            // Caribbean aqua
const LIGHT_AQUA: vec4<f32> = vec4<f32>(0.35, 0.55, 0.62, 1.0);      // Sunlit water
const FOAM_EDGE: vec4<f32> = vec4<f32>(0.55, 0.72, 0.78, 1.0);       // Foam edge
const FOAM_WHITE: vec4<f32> = vec4<f32>(0.82, 0.88, 0.92, 1.0);      // Foam crest

// Hash function for pseudo-random values
fn hash21(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453);
}

// Smooth noise function
fn noise2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));
    
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// Fractal Brownian Motion for complex wave patterns
fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var pos = p;
    
    for (var i = 0; i < 4; i = i + 1) {
        value = value + amplitude * noise2d(pos * frequency);
        amplitude = amplitude * 0.5;
        frequency = frequency * 2.0;
    }
    
    return value;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample velocity from the simulation texture
    let velocity = textureSample(velocity_texture, velocity_sampler, in.uv).xy;
    
    // Calculate speed (magnitude of velocity)
    let speed = length(velocity);
    
    // Apply non-linear mapping for more realistic foam distribution
    // Use square root to make foam appear only at high velocities
    let raw_t = clamp(speed / settings.max_speed, 0.0, 1.0);
    let t = pow(raw_t, 1.5); // Non-linear: keeps more area blue, foam only at high speeds
    
    // Smooth 8-band color interpolation
    var base_color: vec4<f32>;
    if (t < 0.08) {
        base_color = mix(DEEP_OCEAN, OCEAN_BLUE, t / 0.08);
    } else if (t < 0.18) {
        base_color = mix(OCEAN_BLUE, MID_BLUE, (t - 0.08) / 0.10);
    } else if (t < 0.30) {
        base_color = mix(MID_BLUE, TEAL, (t - 0.18) / 0.12);
    } else if (t < 0.45) {
        base_color = mix(TEAL, AQUA, (t - 0.30) / 0.15);
    } else if (t < 0.62) {
        base_color = mix(AQUA, LIGHT_AQUA, (t - 0.45) / 0.17);
    } else if (t < 0.78) {
        base_color = mix(LIGHT_AQUA, FOAM_EDGE, (t - 0.62) / 0.16);
    } else {
        base_color = mix(FOAM_EDGE, FOAM_WHITE, (t - 0.78) / 0.22);
    }
    
    // Multi-layered wave animation
    let time = settings.time;
    
    // Large slow swells
    let swell = sin(in.uv.x * 4.0 + time * 0.3) * 
                sin(in.uv.y * 3.5 + time * 0.25) * 0.015;
    
    // Medium waves with drift
    let wave1 = sin(in.uv.x * 12.0 + in.uv.y * 8.0 + time * 0.7) * 0.012;
    let wave2 = sin(in.uv.x * 8.0 - in.uv.y * 14.0 + time * 0.5) * 0.008;
    
    // Fine ripple detail using FBM
    let ripple_pos = in.uv * 30.0 + vec2<f32>(time * 0.2, time * 0.15);
    let ripple = (fbm(ripple_pos) - 0.5) * 0.025;
    
    // Combine all wave effects
    let total_wave = swell + wave1 + wave2 + ripple;
    
    // Velocity direction influences color shift (gives "flow" appearance)
    var flow_shift = 0.0;
    if (speed > 0.1) {
        let vel_dir = normalize(velocity);
        // Shift toward teal when flowing horizontally, slightly green when vertical
        flow_shift = vel_dir.x * 0.02 * raw_t;
    }
    
    // Add subtle caustic-like shimmer on calm water
    let shimmer_pos = in.uv * 60.0 + time * 0.4;
    let shimmer = max(0.0, noise2d(shimmer_pos) - 0.65) * 0.08 * (1.0 - t);
    
    // Compose final color
    let final_color = vec4<f32>(
        clamp(base_color.r + total_wave + shimmer, 0.0, 1.0),
        clamp(base_color.g + total_wave + shimmer + flow_shift, 0.0, 1.0),
        clamp(base_color.b + total_wave + shimmer * 0.5, 0.0, 1.0),
        base_color.a
    );
    
    return final_color;
}
