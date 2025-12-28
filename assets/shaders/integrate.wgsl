// Integration Pass Compute Shader
// Adds wake velocities to advected velocities
// This allows wakes to properly spread and trail behind ships

const GRID_SIZE: f32 = 256.0;

// Bindings:
// 0: velocity_read - advected velocity from previous pass
// 1: wake_read - wake forces from CPU
// 2: velocity_write - combined result

@group(0) @binding(0) var velocity_read: texture_2d<f32>;
@group(0) @binding(1) var wake_read: texture_2d<f32>;
@group(0) @binding(2) var velocity_write: texture_storage_2d<rg32float, write>;

@compute @workgroup_size(8, 8, 1)
fn integrate(@builtin(global_invocation_id) id: vec3<u32>) {
    // Bounds check
    if (id.x >= u32(GRID_SIZE) || id.y >= u32(GRID_SIZE)) {
        return;
    }
    
    let i = vec2<i32>(id.xy);
    
    // Read advected velocity (carries history from previous frames)
    let advected_vel = textureLoad(velocity_read, i, 0).xy;
    
    // Read wake velocity (fresh forces from this frame's ships)
    let wake_vel = textureLoad(wake_read, i, 0).xy;
    
    // Combine: add wake forces to the advected field
    let combined_vel = advected_vel + wake_vel;
    
    // Write result
    textureStore(velocity_write, i, vec4<f32>(combined_vel, 0.0, 1.0));
}
