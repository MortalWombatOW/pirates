#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct WaterMaterial {
    color: vec4<f32>,
    time: f32,
    flags: u32,
}

@group(2) @binding(0) var<uniform> material: WaterMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Vertex Colors stored in in.color
    // R = Height (0..1)
    // G = Flow X (0..1)
    // B = Flow Y (0..1)
    // A = Depth/LOD
    
    let height_norm = in.color.r;
    let flow_x = in.color.g;
    let flow_y = in.color.b;
    
    // Remap height to -1..1 logic if needed, but 0..1 is fine for mixing.
    // 0 = Deep (-10.0), 1 = High (+10.0). Sea Level around 0.5.
    
    // Base Colors
    let deep_color = vec3<f32>(0.0, 0.1, 0.3);
    let shallow_color = material.color.rgb; // Interface color (0.0, 0.4, 0.8)
    let foam_color = vec3<f32>(0.9, 0.95, 1.0);
    
    // height_norm 0.5 is roughly sea level.
    // Gradient: Deep -> Shallow -> Foam
    
    var final_color = mix(deep_color, shallow_color, smoothstep(0.0, 0.6, height_norm));
    
    // Foam at peaks (height > 0.7?)
    // Add time-based modulation to foam threshold for "activity"
    let foam_threshold = 0.65 - (sin(material.time * 2.0) * 0.05);
    let foam_factor = smoothstep(foam_threshold, foam_threshold + 0.1, height_norm);
    
    final_color = mix(final_color, foam_color, foam_factor);
    
    // Debug Visualizations
    // Height Map (Flag 1)
    if ((material.flags & 1u) != 0u) {
        // Visualize height: 0.0 (Deep/Black) -> 1.0 (High/White)
        return vec4<f32>(vec3<f32>(height_norm), 1.0);
    }
    
    // Foam Map (Flag 2)
    // Recalculate foam factor to visualize raw foam mask
    let debug_foam_threshold = 0.55 - (sin(material.time * 2.0) * 0.05);
    let debug_foam = smoothstep(debug_foam_threshold, debug_foam_threshold + 0.1, height_norm);
    
    if ((material.flags & 2u) != 0u) {
        return vec4<f32>(vec3<f32>(debug_foam), 1.0);
    }
    
    // Grid/Cell Debug (optional, using UVs)
    // let grid = max(step(0.98, in.uv.x), step(0.98, in.uv.y));
    // final_color = mix(final_color, vec3<f32>(0.0, 0.0, 0.0), grid * 0.1);

    return vec4<f32>(final_color, 1.0);
}
