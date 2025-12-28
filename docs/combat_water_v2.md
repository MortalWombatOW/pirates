# Epic 8.8: Combat Water Simulation

**Status:** Approved for Implementation  
**Owner:** Engineering  
**Context:** Implements a custom "Stable Fluids" solver to replace the static background. This v3 design replaces the library-based approach to ensure cross-platform compatibility (specifically fixing Metal/M1 race conditions).

## 1. Overview
We will implement a custom, grid-based fluid simulation using Bevy's Render Graph and Compute Shaders.
* **Goal:** A dynamic ocean that reacts to ships (wakes) and pushes them (currents).
* **Constraint:** Must run on Metal (macOS) without "read-write" race conditions.
* **Visuals:** Quantized "Blue-to-White" palette based on water speed.

## 2. Architecture: Double-Buffered Stable Fluids
To guarantee stability on all hardware, we use a **Ping-Pong (Double Buffering)** architecture. We never read and write to the same texture in the same dispatch.

### 2.1 Data Structures (Resources)
We need a custom resource `FluidSimulation` containing texture pairs.
* **Grid Resolution:** `256 x 256` (Upscaled to cover the ~1000px arena).
* **Buffers:**
    * `Velocity`: `RG32Float` (Current, Next)
    * `Pressure`: `R32Float` (Current, Next)
    * `Divergence`: `R32Float` (Single texture, intermediate)

### 2.2 The Simulation Loop (Compute Pipeline)
Runs every `FixedUpdate` (or every frame if perf allows).

1.  **Advection Pass:** Moves fluid properties along the velocity field.
    * *Input:* `Velocity (Read)`, `Velocity (Read)` (yes, it advects itself).
    * *Output:* `Velocity (Write)`.
2.  **Integration (Splat) Pass:** Injects external forces (Ships).
    * *Input:* `Velocity (Read)`, `ShipBuffer (Uniform/Storage)`.
    * *Output:* `Velocity (Write)`.
3.  **Divergence Pass:** Calculates where water is "compressing".
    * *Input:* `Velocity (Read)`.
    * *Output:* `Divergence (Write)`.
4.  **Pressure Solve (Jacobi) Pass:** Runs ~20-40 times.
    * *Input:* `Pressure (Read)`, `Divergence (Read)`.
    * *Output:* `Pressure (Write)`.
    * *Note:* Swaps Read/Write handles every iteration.
5.  **Gradient Subtract Pass:** Enforces incompressibility.
    * *Input:* `Velocity (Read)`, `Pressure (Read)`.
    * *Output:* `Velocity (Write)` (Final result for rendering).

---

## 3. Game Integration

### 3.1 Ship to Water (Wakes)
Instead of complex collider rasterization, we use **Gaussian Splatting**.
* **System:** `prepare_fluid_forces`
* **Logic:**
    1.  Query all active Ships (`Transform`, `LinearVelocity`, `Collider`).
    2.  Extract `position`, `velocity`, and rough `radius`.
    3.  Upload this list to a `StorageBuffer` visible to the **Integration Pass**.
* **Shader Logic:** For each pixel, check distance to ships. If inside radius, `mix()` fluid velocity towards ship velocity.

### 3.2 Water to Ship (Drift)
We need to apply the water's velocity back to the `avian2d` rigid bodies.
* **Option A (Accurate):** Async Readback of the Velocity Texture to CPU. (Can have 1-2 frame latency, which is acceptable for "drift").
* **Option B (Fast):** If readback is too slow, use a simplified "Wind" logic on CPU that approximates the main current direction, while the GPU handles the visual turbulence.
* **Recommendation:** Start with Option A. If latency causes jitter, switch to Option B.

---

## 4. Rendering (The "Quantized" Look)
The visualization is decoupled from the physics grid resolution.

**Vertex Shader:** Standard Quad covering the arena.
**Fragment Shader:**
* Samples the **Velocity Texture** (Linear filtered).
* Calculates magnitude (`length(vel)`).
* Applies quantization logic:
    ```wgsl
    let speed = length(velocity);
    let t = smoothstep(0.0, max_speed, speed);
    let band = floor(t * 4.0) / 4.0; // 4 color bands
    let color = mix(BLUE, WHITE, band);
    ```

---

## 5. Implementation Roadmap

### Phase 1: The Render Graph Setup
* **Task:** Create `FluidSimulationPlugin`.
* **Task:** Setup the `FluidImages` resource (create textures with `STORAGE_BINDING` usage).
* **Task:** Implement the "Compute Node" boilerplate (the hardest part in Bevy). It needs to dispatch the pipeline stages in order.

### Phase 2: Compute Shaders (WGSL)
* **Task:** Write `fluids.wgsl`.
    * `fn advect(...)`
    * `fn divergence(...)`
    * `fn jacobi(...)`
    * `fn subtract(...)`
* *Reference:* "GPU Gems: Fast Fluid Dynamics Simulation on the GPU".

### Phase 3: Integration
* **Task:** Implement `extract_ships` system to populate the GPU buffer.
* **Task:** Write `integrate.wgsl` to read the buffer and modify velocity.
* **Task:** Connect the final texture to a `Material2d`.

### Phase 4: Tuning
* **Tweak:** `viscosity` (decay rate in advection).
* **Tweak:** `splat_radius` and `splat_force` (how strong the wakes are).
* **Tweak:** Color palette thresholds.

---

## 6. M1/Metal Compatibility Notes
* **Atomic Operations:** Avoid them. Use the ping-pong texture approach.
* **Workgroup Size:** Use `(8, 8, 1)`. Total threads 64 is safe on Apple Silicon (SIMD width is 32, so 64 aligns well).
* **Texture Format:** Use `R32Float` and `RG32Float`. Avoid `R16Float` if possible as storage support varies; 32-bit is safer for physics precision.

---

## 7. Implementation Notes (December 2024)

### 7.1 Current Status
The core Stable Fluids simulation is **implemented and working**:
- ✅ Wake injection from ship velocities (Gaussian splatting)
- ✅ GPU texture upload via `RenderQueue::write_texture`
- ✅ Advection pass with semi-Lagrangian method
- ✅ Divergence calculation
- ✅ Jacobi pressure solve (20 iterations, ping-pong)
- ✅ Gradient subtraction for incompressibility
- ✅ Water surface material with velocity-to-color mapping

### 7.2 Key Files
- `src/plugins/fluid_simulation.rs` - Plugin, compute node, bind groups, wake injection
- `assets/shaders/fluids.wgsl` - All 4 compute shader passes
- `assets/shaders/water_material.wgsl` - Fragment shader for visualization
- `src/resources/water_material.rs` - Material2d definition

### 7.3 Problems Encountered

#### Problem 1: CPU-to-GPU Texture Update
**Issue:** Modifying `Assets<Image>` directly after extraction didn't update the GPU texture due to Bevy's caching.
**Solution:** Use `RenderQueue::write_texture()` in the render world to directly upload texture data. Created `WakeTextureData` resource with `ExtractResource` derive for CPU→render world transfer.

#### Problem 2: White Saturation
**Issue:** Water became completely white very quickly.
**Causes tried:**
1. `WAKE_FORCE_MULTIPLIER` too high (was 50.0)
2. Shader `max_speed` too low relative to actual velocities
3. Wake data accumulating without proper decay

**Notes:** Reduced `WAKE_FORCE_MULTIPLIER` to 0.5, kept `VISCOSITY` at 0.99. Don't clear the wake buffer each frame (preserves simulation history).

#### Problem 3: Y Coordinate Flip
**Issue:** Wake positions were vertically inverted.
**Solution:** In `inject_ship_wakes`, flip Y when mapping world to grid: `grid_y = grid_size - 1 - ((pos.y + half_arena) * grid_scale)`

#### Problem 4: Ring Pattern (Empty Center)
**Issue:** When clearing the wake buffer each frame, only saw wake ring at border, not filled circle.
**Cause:** Clearing the buffer each frame destroyed simulation history. The advection/pressure solve naturally pushes velocity outward, so only the freshly-injected edge showed.
**Solution:** Remove the buffer clear to preserve advected velocities from previous frames.

### 7.4 Current Tuning Parameters
```rust
const FLUID_GRID_SIZE: u32 = 256;
const WORKGROUP_SIZE: u32 = 8;
const WAKE_SPLAT_RADIUS: i32 = 8;
const WAKE_FORCE_MULTIPLIER: f32 = 0.5;
const JACOBI_ITERATIONS: u32 = 20;
```
```wgsl
const VISCOSITY: f32 = 0.95;  // Per-frame velocity decay (reduced from 0.99 to prevent white saturation)
const DT: f32 = 0.016667;     // 1/60 for 60Hz
```

### 7.5 Known Issues / Future Work
1. **Water-to-Ship drift not implemented** - Need async GPU readback or CPU approximation
2. **Texture ping-pong state not swapped per frame** - Currently uses fixed buffer assignment
3. **No boundary conditions** - Fluid at edges may behave oddly
4. **Consider decoupling injection from simulation** - Currently wake injection happens via CPU texture overwrite, could be a compute shader pass instead for better integration

### 7.6 Performance Notes
- Running at 60fps on Apple M1
- 256x256 grid with 20 Jacobi iterations
- 4 compute passes per frame + wake injection