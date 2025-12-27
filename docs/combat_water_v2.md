# Epic 8.8: Combat Water Simulation (Design v2)

**Context:** Replaces the static "Ink & Parchment" water background in the Combat Phase with a dynamic, grid-based Eulerian fluid simulation.

## 1. Overview
The goal is to implement a top-down 2D fluid simulation for the "Open Ocean" combat arena. The water must interact physically with ships (wakes, drift) and render using a specific stylized palette (quantized blue-to-white based on speed).

We will use the **`bevy_eulerian_fluid`** crate, which provides a robust Eulerian solver and native integration with our physics backend, **`avian2d`**.

## 2. Architecture & Library Selection

### Selected Library: `bevy_eulerian_fluid`
* **Why:** It implements a stable grid-based solver (ideal for "ocean currents" and "wakes") and supports **two-way coupling** with `avian2d` rigid bodies.
* **Features Used:**
    * **Incompressible Flow:** Water behaves like water, not gas.
    * **Solid Coupling:** Ships (colliders) displace water; water velocity exerts drag/drift on ships.
    * **Velocity Texture Access:** The simulation exports the velocity field as a `Image` (texture) which we will bind to our custom shader.

### Integration Architecture
1.  **Simulation Domain:** A rectangular grid entity spawned only during `GameState::Combat`.
2.  **Physics Layer:** The library handles the fluid-body interaction automatically via `avian2d` components (`Collider`, `RigidBody`).
3.  **Rendering Layer:** We will **not** use the library's default debug rendering. Instead, we will render a quad (`MaterialMesh2dBundle`) covering the arena, using a custom `WaterShaderMaterial` that samples the simulation's velocity texture.

---

## 3. Visual Style (The "Quantized" Look)
**Requirement:** Realistic fluid motion rendered in a stylized manner.
* **Palette:** Deep Blue (Slow) $\to$ White (Fast).
* **Quantization:** Adjustable "steps" (bands) of color to fit the pixel-art aesthetic.

### Shader Logic (Draft)
The fragment shader will sample the **Velocity Field** texture provided by the simulation.

```wgsl
struct WaterMaterial {
    color_deep: vec4<f32>,
    color_fast: vec4<f32>,
    quantize_steps: f32,
    max_speed_ref: f32, // Speed at which water turns fully white
}

@group(1) @binding(0) var velocity_texture: texture_2d<f32>;
@group(1) @binding(1) var velocity_sampler: sampler;
@group(1) @binding(2) var<uniform> material: WaterMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Sample velocity vector (RG components usually store XY velocity)
    let vel = textureSample(velocity_texture, velocity_sampler, in.uv).xy;
    let speed = length(vel);

    // 2. Normalize speed (0.0 to 1.0)
    let t = clamp(speed / material.max_speed_ref, 0.0, 1.0);

    // 3. Quantize (Posterization)
    // Example: steps = 4.0 turns smooth gradient into 4 solid bands
    let stepped_t = floor(t * material.quantize_steps) / material.quantize_steps;

    // 4. Mix Colors
    return mix(material.color_deep, material.color_fast, stepped_t);
}

```

---

## 4. Implementation Plan

### Phase 1: Core Setup

1. **Dependencies:** Add `bevy_eulerian_fluid` to `Cargo.toml`.
2. **Plugin Registration:** Add `FluidPlugin` to `CombatPlugin`.
3. **Setup System:** Implement `spawn_water_grid` system running `OnEnter(GameState::Combat)`.
* Grid Size: Start with `128x128` or `256x256` covering the play area. High resolution impacts CPU performance; tune carefully.
* Physical Size: Match the camera view (e.g., 1000x1000 world units).
* Density/Viscosity: Tune for "watery" feel (low viscosity).



### Phase 2: Physics Integration

1. **Ship Colliders:** Ensure Player and AI ships use `Collider::capsule` or `Collider::rectangle`. (Existing `rectangle` on test target is fine, but `capsule` is more hydrodynamic).
2. **Coupling:** Enable the library's two-way coupling feature.
* *Verify:* Moving ships should leave a trail in the debug view.
* *Verify:* Stationary ships should drift if we inject velocity into the grid.



### Phase 3: Custom Rendering

1. **Material:** Create `WaterMaterial` (struct + `impl Material2d`).
2. **Pipeline:**
* In `Update`, query the `FluidTextures` component from the simulation entity.
* Extract the `velocity_texture` handle.
* Update the `WaterMaterial` handle on the visible mesh to point to this new texture handle (if it changes) or bind it initially.


3. **Tuning:** Adjust `max_speed_ref` and `quantize_steps` to match the game's art style.

### Phase 4: Cleanup Legacy Systems

The following systems in `src/systems/combat.rs` and `movement.rs` are obsolete and must be removed/disabled:

* `current_zone_system`: Fluid simulation now handles drift.
* `spawn_test_current_zone`: No longer needed.
* `wake_effects.rs`: The particle-based wake trail is redundant; the white water "foam" from the shader will act as the visual wake.

---

## 5. Technical Considerations

### Performance

* **Grid Resolution:** This is the #1 performance bottleneck.
* *Constraint:* The simulation runs on CPU (typically) or Compute Shader depending on backend config.
* *Target:* 60 FPS. If `256x256` is too slow, drop to `128x128` and use linear filtering in the texture sampler to smooth the look.


* **Simulate Frequency:** Consider running the fluid step in `FixedUpdate` (physics tick) rather than every frame if FPS drops.

### Collision Shapes

* **Capsules:** Preferred for ships. They cut through the grid smoothly.
* **Boxes:** Acceptable, but will generate stronger turbulence at the corners (which might actually look cool for "boxy" pirate ships).
