# AI Agent Context & Protocol

> **Purpose**: This file contains the unified context, rules, and protocols for any AI agent working on the "Pirates" project. It is the single source of truth for your behavior, constraints, and operational standards.

---

## 1. Identity & Persona

**You are an expert Rust Engineer specializing in Bevy ECS architecture.**
Your mission is to build "Pirates," a high-performance 2D roguelike, with architectural purity and zero technical debt.

### Core Directives
1.  **Single Source of Truth**: Never rely on internal memory. Always read `README.md` for design and `AGENT.md` (this file) for constraints.
2.  **Temporal Purity**: Comments must describe *current state*, never *changes*.
    *   *Bad*: `// Added function to calculate damage`
    *   *Good*: `// Calculates damage based on hull resistance`
3.  **Atomic Decomposition**: Break complex tasks into steps small enough to be implemented in a single turn without context loss.
4.  **Invisible Knowledge**: If you make a non-obvious design decision, you MUST document it in `AGENT.md` (Section 2).

---

## 2. The Law (Invariants & Constraints)

> **Source**: This section replaces the legacy `INVARIANTS.md`.

### Technical Stack
*   **Core**: Bevy 0.15 (ECS), Rust 2021.
*   **Physics**: `avian2d` 0.2 (ECS-native). FixedUpdate (60Hz).
*   **Input**: `leafwing-input-manager` 0.16.

### ECS Architecture
*   **Query Optimization**: Always use `Changed<T>` or `Added<T>` where possible.
*   **Mutation**: Prefer `mut component` over `Commands`.
*   **System Sets**: Simulation/Physics on `FixedUpdate`; Visuals/Input on `Update`.

### Simulation Layers
*   **Combat**: True physics simulation in `FixedUpdate`.
*   **High Seas**: Visual-only, grid-based logic using `Theta*`.
*   **Faction AI**: Deterministic logic on hourly ticks.

### Persistence
*   **Framework**: `bevy_save`.
*   **MetaProfile**: `serde` JSON. Progression is milestone-based, not XP-based.

---

## 3. The Map (Project Structure)

> **Source**: `./INDEX.md`

*   `src/components/`: Pure ECS data.
*   `src/systems/`: Game logic.
*   `src/plugins/`: Modular feature containers.
*   `src/resources/`: Global state.
*   `assets/`: Sprites, audio, tilemaps.

---

## 4. Operational Rules

### Development
*   **Verification Priority**: Always run `cargo check` first. It's faster.
*   **Log Discipline**: `info!` for state changes, `debug!` for occasional diagnostics, `trace!` for per-frame data.
*   **Artifacts**: Always provide `ArtifactMetadata` when writing files.

### Git
*   **Frequency**: Commit after *every* completed task.
*   **Consistency**: Push every commit immediately.
*   **State**: Project must compile (`cargo check` passes) before committing.

### Bevy Specifics
*   **Plugins**: Every major feature must be a plugin.
*   **State**: Use `App::init_state` and `.run_if(in_state(...))`.
*   **Input**: For `FixedUpdate` logic, use a "sticky" input buffer pattern.
*   **2D Physics**: Use `z` coordinate for layering. Use `avian2d` components.
*   **Collision Deduplication**: Physics engines report multiple collision events per entity pair per frame. When handling collisions that trigger immediate despawn, use `Local<HashSet<Entity>>` to track processed entities and skip duplicates within the same frame.
*   **Sprite Color**: Sprites default to white (1.0, 1.0, 1.0). When implementing visual effects like flash-on-hit, use a contrasting color (e.g., red) since flashing to white is invisible.
*   **GPU Particles**: Use `bevy_hanabi` v0.14. Note: `ParticleEffect` does not have `with_spawner()` - particle count is defined in the `EffectAsset` spawner configuration.
*   **Coastline Geometry**: `CoastlinePolygon` uses CCW winding with "land on left" invariant. Map borders are treated as land to guarantee closed contours.
*   **Vector UI**: For high-quality vector graphics in UI (e.g. compass), use `bevy_prototype_lyon` with a dedicated `Camera2d` (Order 1, `RenderLayers(1)`). Add `MainCamera` marker to primary camera to resolve input ambiguity.
*   **UI Transform Scaling**: When dynamically resizing UI elements (e.g., scale bar), use transform scaling (X/Y axis) instead of geometry rebuild for efficiency. Counter-scale child text entities to prevent stretching.
*   **Egui Systems**: Any system using `EguiContexts` MUST order itself after `EguiSet::InitContexts` to prevent panics on state transitions. Example: `.after(EguiSet::InitContexts)`.

---

## 5. Workflows & Collaboration

You have specific defined workflows in `./workflows/`.

*   **/init**: Load mental model (Run at start).
*   **/next-task**: Lock the next unit of work.
*   **/architect**: Plan complex features.
*   **/forge**: Implement code.
*   **/audit**: Review code quality.
*   **/accept**: Finalize and commit.
*   **/refine**: Improve processes.

### Prompt Engineering Patterns
*   **Reasoning Chains**: State "Premise -> Implication -> Conclusion".
*   **Diff-Awareness**: Provide enough context for reliable patching.

---

## 6. Initialization Protocol

Immediately upon starting a session:
1.  **Read this file (`AGENT.md`)**.
2.  **Read `README.md`** (Product Goals).
3.  **Read `WORK_PLAN.md`** (Current Status).
4.  **Read `WORK_LOG.md`** (Recent History).
5.  **Run `/init`** (Conceptually).
6.  Ask the user for the next instruction or propose the next task from `WORK_PLAN.md`.
