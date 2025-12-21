# Project Invariants & Technical Constraints

> **Purpose**: This file captures architectural rules, invisible knowledge, and constraints that cannot be learned simply by reading the code.

---

## 1. Technical Stack & Versions
* **Engine**: Bevy 0.15
* **Physics**: `avian2d` 0.2 (ECS-native, replaces Rapier)
* **Input**: `leafwing-input-manager` 0.16 (Action-based)
* **Language**: Rust 2021 Edition

## 2. ECS Architecture Patterns
* **Change Detection**: Always use `Changed<T>` or `Added<T>` in queries to avoid unnecessary computation.
    ```rust
    // Correct
    fn update_health(query: Query<&Health, Changed<Health>>) { ... }
    ```
* **Commands vs Mutation**: Prefer direct component mutation over `Commands` for performance. Use `Commands` only for structural changes (spawn/despawn/insert/remove).
* **System Sets**: All systems must be registered to a plugin. Use `Update` for logic and `FixedUpdate` for physics/simulation.

## 3. Physics & Simulation
* **Timestep**: Physics runs on `FixedUpdate` (60Hz deterministic).
* **Anisotropic Drag**: Ships do NOT use isotropic `LinearDamping`. They use custom force calculations to simulate "Keel Effect" (high lateral drag, low forward drag).
* **Input Buffering**: `leafwing` inputs update in `Update`. To prevent missed inputs in `FixedUpdate`, inputs must be buffered into a `ShipInputBuffer` resource.

## 4. Documentation Standards
* **Temporal Contamination**: Comments must never describe *changes*.
    * *Forbidden*: `// Fixed bug where ship spins`
    * *Required*: `// Clamps angular velocity to prevent spinning`
* **Source of Truth**: `README.md` is the GDD. This file (`INVARIANTS.md`) is the Technical Constraint list.

## 5. UI & Rendering
* **Debug UI**: Use `bevy_egui` for all debug tools.
* **Camera**: Use `Camera2d` component. Panning/Zooming logic must handle resolution scaling.