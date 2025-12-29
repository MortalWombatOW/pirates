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
*   **Assumption Verification**: Avoid "magic numbers" for data filtering (e.g., `points.len() > 500`). Verify data properties via logging or tests before applying filters.

### Warning Resolution (Zero Tolerance)
*   **Root Cause Analysis**: Never suppress or patch over warnings. Investigate *why* the warning exists.
*   **Unused Variables**: If a variable is unused, determine the *intent*:
    *   If it *should* be unused (e.g., intentionally ignored return value), remove it or use `_` prefix with a comment explaining why.
    *   If it *should* be used but isn't, this indicates incomplete implementation—integrate it correctly.
*   **Dead Code**: Same principle applies. Determine if code was intended to be called but isn't (fix the caller) or is truly obsolete (delete it).
*   **Build Must Be Warning-Free**: `cargo check` must produce zero warnings before committing.

### No-Fallback Rule (Implementation Persistence)
*   **CRITICAL**: When implementing a *replacement* for existing functionality, you MUST NOT "fix" problems by reverting to the old implementation.
*   **Systematic Debugging**: If the new implementation has issues:
    1.  Add diagnostic logging to understand the failure.
    2.  Isolate the specific failure case.
    3.  Fix the root cause in the new implementation.
    4.  Repeat until the new implementation works correctly.
*   **Escalation, Not Regression**: If you cannot fix the new implementation, **STOP** and document the issue in `WORK_PLAN.md` as a blocker. Do not silently fall back.
*   **Old Code Removal**: Once a replacement is working, delete the old implementation. Do not leave it "for reference."

### Save-Based Feature Verification
*   **Test Saves**: Every major feature/invariant requires a corresponding test save named `test_<feature>`.
*   **Log-Based Proof**: Each feature must emit `info!` logs that prove correct behavior. Document expected log patterns in `WORK_PLAN.md`.
*   **Save Locations**: Saves are stored in platform-specific app data: `~/Library/Application Support/pirates/` (macOS), `~/.local/share/pirates/` (Linux), `%APPDATA%/pirates/` (Windows).

#### How to Create a Test Save
```bash
# Step 1: Launch game with --save-as to override the F5 save name
cargo run -- --save-as test_myfeature

# Step 2: In-game, set up the conditions needed to demonstrate the feature
#         (e.g., sail to a specific location, acquire items, trigger state)

# Step 3: Press F5 to save. Console will show:
#         "Game saved successfully to 'test_myfeature'"

# Step 4: Exit the game (Cmd+Q / Alt+F4)
```

#### How to Verify a Feature
```bash
# Load the test save and check for expected log output
cargo run -- --load test_myfeature 2>&1 | grep "Expected log pattern"

# Example: Verify pathfinding finds a route
cargo run -- --load test_pathfinding 2>&1 | grep "route found"

# Example: Verify combat damage calculation
cargo run -- --load test_combat 2>&1 | grep "Damage applied"
```

#### Verification Workflow
1. Create test save with conditions that exercise the feature
2. Add `info!()` logs to the feature code that prove correct behavior
3. Run verification command and confirm expected logs appear
4. Document the verification command in `WORK_PLAN.md` for the task

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
*   **Navigation**: Uses `bevy_landmass` v0.8.0 for velocity-based steering. Ships are `Agent2d` entities with `AgentDesiredVelocity2d`. Set destinations via `Destination` component (synced to `AgentTarget2d`). Three archipelagos exist for ship size tiers (Small/Medium/Large shore buffers).
*   **Coastline Avoidance**: Movement uses facing direction (not desired velocity) for realistic sailing. To prevent shore collisions: (1) speed reduces quadratically with facing/desired misalignment, (2) `coastline_avoidance_system` finds nearest coastline polygon edge and pushes ship to water side if on wrong side of the edge normal. **CRITICAL**: `CoastlineData` contains ALL polygons including map borders. Do NOT filter polygons by point count (e.g. >500) as smoothed local coastlines can be large. Use spatial bounds if border detection is needed.
*   **Stippling Shader Pattern**: Uses `Material2d` with dynamically generated density texture from `MapData`. To prevent rings overlapping coastlines, shader samples density at 4 cardinal edge points and discards if any sample indicates land. UV coordinates require Y-flip (texture y=0 is top, UV v=0 is bottom). Constants: MAP_SIZE = 512 tiles × 64 units = 32768 world units.
*   **GPU Compute Ping-Pong**: When implementing multi-pass compute shaders with texture ping-pong, ensure each pass has distinct read/write targets. Use explicit bind group construction to avoid `TextureUses(RESOURCE)` vs `TextureUses(STORAGE_READ_WRITE)` conflicts. Document data flow (e.g., `Advection: A→B, Integration: B→A, Divergence: A→div, Subtract: A→B`).

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
