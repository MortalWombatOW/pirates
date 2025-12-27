// Stable Fluids Compute Shader
// Implements advection, divergence, pressure solve, and gradient subtraction
// Using ping-pong textures to avoid Metal read-write hazards

// Workgroup size: 8x8 is safe for Apple Silicon (SIMD width 32)
const WORKGROUP_SIZE: u32 = 8u;
const GRID_SIZE: f32 = 256.0;
const DT: f32 = 0.016667; // 1/60 for 60Hz
const VISCOSITY: f32 = 0.99;

// ============================================================================
// Advection Pass
// ============================================================================
// Moves fluid properties along the velocity field using semi-Lagrangian advection.

@group(0) @binding(0) var velocity_read: texture_2d<f32>;
@group(0) @binding(1) var velocity_write: texture_storage_2d<rg32float, write>;

@compute @workgroup_size(8, 8, 1)
fn advect(@builtin(global_invocation_id) id: vec3<u32>) {
    let grid_size = vec2<f32>(GRID_SIZE, GRID_SIZE);
    let pos = vec2<f32>(f32(id.x), f32(id.y));
    
    // Bounds check
    if (id.x >= u32(GRID_SIZE) || id.y >= u32(GRID_SIZE)) {
        return;
    }
    
    // Sample current velocity at this cell
    let vel = textureLoad(velocity_read, vec2<i32>(id.xy), 0).xy;
    
    // Trace back in time to find source position (semi-Lagrangian)
    let source_pos = pos - vel * DT;
    
    // Clamp to grid bounds
    let clamped_pos = clamp(source_pos, vec2<f32>(0.5), grid_size - vec2<f32>(0.5));
    
    // Bilinear interpolation coordinates
    let base = floor(clamped_pos - vec2<f32>(0.5));
    let frac = clamped_pos - base - vec2<f32>(0.5);
    
    // Sample 4 neighbors
    let i = vec2<i32>(base);
    let v00 = textureLoad(velocity_read, i, 0).xy;
    let v10 = textureLoad(velocity_read, i + vec2<i32>(1, 0), 0).xy;
    let v01 = textureLoad(velocity_read, i + vec2<i32>(0, 1), 0).xy;
    let v11 = textureLoad(velocity_read, i + vec2<i32>(1, 1), 0).xy;
    
    // Bilinear interpolation
    let v0 = mix(v00, v10, frac.x);
    let v1 = mix(v01, v11, frac.x);
    var advected_vel = mix(v0, v1, frac.y);
    
    // Apply viscosity decay
    advected_vel *= VISCOSITY;
    
    // Write advected velocity
    textureStore(velocity_write, vec2<i32>(id.xy), vec4<f32>(advected_vel, 0.0, 1.0));
}

// ============================================================================
// Divergence Pass
// ============================================================================
// Calculates where water is "compressing" (needed for pressure solve).

@group(0) @binding(0) var div_velocity_read: texture_2d<f32>;
@group(0) @binding(1) var divergence_write: texture_storage_2d<r32float, write>;

@compute @workgroup_size(8, 8, 1)
fn divergence(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= u32(GRID_SIZE) || id.y >= u32(GRID_SIZE)) {
        return;
    }
    
    let i = vec2<i32>(id.xy);
    let grid_max = i32(GRID_SIZE) - 1;
    
    // Central differences with boundary clamping
    let iL = vec2<i32>(max(i.x - 1, 0), i.y);
    let iR = vec2<i32>(min(i.x + 1, grid_max), i.y);
    let iB = vec2<i32>(i.x, max(i.y - 1, 0));
    let iT = vec2<i32>(i.x, min(i.y + 1, grid_max));
    
    let vL = textureLoad(div_velocity_read, iL, 0).x;
    let vR = textureLoad(div_velocity_read, iR, 0).x;
    let vB = textureLoad(div_velocity_read, iB, 0).y;
    let vT = textureLoad(div_velocity_read, iT, 0).y;
    
    // Divergence = dVx/dx + dVy/dy
    let div = 0.5 * ((vR - vL) + (vT - vB));
    
    textureStore(divergence_write, i, vec4<f32>(div, 0.0, 0.0, 1.0));
}

// ============================================================================
// Jacobi Pressure Solve Pass
// ============================================================================
// Iteratively solves for pressure field (run 20-40 times, ping-pong).

@group(0) @binding(0) var pressure_read: texture_2d<f32>;
@group(0) @binding(1) var divergence_read: texture_2d<f32>;
@group(0) @binding(2) var pressure_write: texture_storage_2d<r32float, write>;

@compute @workgroup_size(8, 8, 1)
fn jacobi(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= u32(GRID_SIZE) || id.y >= u32(GRID_SIZE)) {
        return;
    }
    
    let i = vec2<i32>(id.xy);
    let grid_max = i32(GRID_SIZE) - 1;
    
    // Sample neighboring pressures with boundary clamping
    let iL = vec2<i32>(max(i.x - 1, 0), i.y);
    let iR = vec2<i32>(min(i.x + 1, grid_max), i.y);
    let iB = vec2<i32>(i.x, max(i.y - 1, 0));
    let iT = vec2<i32>(i.x, min(i.y + 1, grid_max));
    
    let pL = textureLoad(pressure_read, iL, 0).x;
    let pR = textureLoad(pressure_read, iR, 0).x;
    let pB = textureLoad(pressure_read, iB, 0).x;
    let pT = textureLoad(pressure_read, iT, 0).x;
    
    let div = textureLoad(divergence_read, i, 0).x;
    
    // Jacobi iteration: p_new = (neighbors - div) / 4
    let p_new = (pL + pR + pB + pT - div) * 0.25;
    
    textureStore(pressure_write, i, vec4<f32>(p_new, 0.0, 0.0, 1.0));
}

// ============================================================================
// Gradient Subtraction Pass
// ============================================================================
// Subtracts pressure gradient from velocity to enforce incompressibility.

@group(0) @binding(0) var sub_velocity_read: texture_2d<f32>;
@group(0) @binding(1) var sub_pressure_read: texture_2d<f32>;
@group(0) @binding(2) var sub_velocity_write: texture_storage_2d<rg32float, write>;

@compute @workgroup_size(8, 8, 1)
fn subtract(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= u32(GRID_SIZE) || id.y >= u32(GRID_SIZE)) {
        return;
    }
    
    let i = vec2<i32>(id.xy);
    let grid_max = i32(GRID_SIZE) - 1;
    
    // Current velocity
    let vel = textureLoad(sub_velocity_read, i, 0).xy;
    
    // Pressure gradient (central differences with boundary clamping)
    let iL = vec2<i32>(max(i.x - 1, 0), i.y);
    let iR = vec2<i32>(min(i.x + 1, grid_max), i.y);
    let iB = vec2<i32>(i.x, max(i.y - 1, 0));
    let iT = vec2<i32>(i.x, min(i.y + 1, grid_max));
    
    let pL = textureLoad(sub_pressure_read, iL, 0).x;
    let pR = textureLoad(sub_pressure_read, iR, 0).x;
    let pB = textureLoad(sub_pressure_read, iB, 0).x;
    let pT = textureLoad(sub_pressure_read, iT, 0).x;
    
    let grad = vec2<f32>(pR - pL, pT - pB) * 0.5;
    
    // Subtract gradient from velocity
    let new_vel = vel - grad;
    
    textureStore(sub_velocity_write, i, vec4<f32>(new_vel, 0.0, 1.0));
}
