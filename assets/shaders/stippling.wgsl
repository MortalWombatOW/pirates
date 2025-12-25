#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct StipplingMaterial {
    color: vec4<f32>,
    // Spacing between dots in world units
    dot_spacing: f32,
}

@group(2) @binding(0) var<uniform> material: StipplingMaterial;
@group(2) @binding(1) var depth_texture: texture_2d<f32>;
@group(2) @binding(2) var depth_sampler: sampler;

// Map dimensions for UV offset calculation (512 tiles * 64 units = 32768 world units)
const MAP_SIZE_WORLD: f32 = 32768.0;

// Hash for randomizing dot positions
fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Sample density with Y-flip
fn sample_density_uv(uv: vec2<f32>) -> f32 {
    let flipped = vec2<f32>(uv.x, 1.0 - uv.y);
    return textureSample(depth_texture, depth_sampler, flipped).r;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Early out: sample density at current position using mesh UV
    let center_density = sample_density_uv(mesh.uv);
    if (center_density < 0.01) {
        discard;
    }
    
    // Create a grid of potential dot locations in world space
    let grid_pos = mesh.world_position.xy / material.dot_spacing;
    let cell = floor(grid_pos);
    let cell_uv = fract(grid_pos);
    
    // Add 15% random jitter to break up grid structure
    let jitter_x = hash21(cell) * 0.3 - 0.15;
    let jitter_y = hash21(cell + vec2<f32>(17.0, 31.0)) * 0.3 - 0.15;
    let jittered_center = vec2<f32>(0.5 + jitter_x, 0.5 + jitter_y);
    
    // Ring dimensions (all rings same size)
    let outer_radius = 0.25;
    let inner_radius = 0.18;
    
    // Calculate ring center position relative to current fragment
    // in cell-local coordinates
    let ring_center_offset = jittered_center - cell_uv;
    
    // Convert ring edge offset from world units to UV units
    // outer_radius is in cell units, dot_spacing converts to world, then to UV
    let edge_world = outer_radius * material.dot_spacing;
    let edge_uv = edge_world / MAP_SIZE_WORLD;
    
    // Calculate ring center in UV space
    let ring_center_world_offset = ring_center_offset * material.dot_spacing;
    let ring_center_uv_offset = ring_center_world_offset / MAP_SIZE_WORLD;
    let ring_center_uv = mesh.uv + vec2<f32>(ring_center_uv_offset.x, -ring_center_uv_offset.y);
    
    // Sample density at 4 cardinal points on ring edge
    let density_n = sample_density_uv(ring_center_uv + vec2<f32>(0.0, edge_uv));
    let density_s = sample_density_uv(ring_center_uv + vec2<f32>(0.0, -edge_uv));
    let density_e = sample_density_uv(ring_center_uv + vec2<f32>(edge_uv, 0.0));
    let density_w = sample_density_uv(ring_center_uv + vec2<f32>(-edge_uv, 0.0));
    
    // All edge samples must be in water (density > 0) to draw ring
    let min_edge_density = min(min(density_n, density_s), min(density_e, density_w));
    if (min_edge_density < 0.01) {
        discard;
    }
    
    // Distance to jittered cell center
    let to_center = cell_uv - jittered_center;
    let dist = length(to_center);
    
    // Randomize dot existence per cell (adds organic feel)
    let random = hash21(cell + vec2<f32>(7.0, 11.0));
    let threshold = 1.0 - center_density;
    
    // Ring: outer_radius > dist > inner_radius
    if (random > threshold && dist < outer_radius && dist > inner_radius) {
        return material.color;
    }
    
    discard;
}

