# Work Log

## 2025-12-24: Tasks 8.5.8-8.5.9 - ScaleBar UI Component
*   **8.5.8**: Created `ScaleBarPlugin` in `src/plugins/scale_bar.rs`:
    *   Authentic 18th-century nautical chart scale with alternating ink/parchment segments.
    *   Uses Lyon vector graphics on RenderLayer 1 (overlay camera).
    *   Positioned in bottom-left corner, repositions on window resize.
*   **8.5.9**: Made scale bar zoom-responsive:
    *   Added `ScaleBarConfig` resource to track segment distance and total miles.
    *   Bar width dynamically adjusts via transform scaling.
    *   Label shows total distance (e.g., "10 MILES", "50 MILES").
    *   Counter-scaled label text to prevent stretching.
*   **Refactor**: Extracted shared overlay UI utilities:
    *   Created `overlay_ui.rs` with shared camera and color constants.
    *   Refactored `compass_rose.rs` to use shared module.
    *   Added `UI Transform Scaling` pattern to `AGENT.md`.

## 2025-12-24: BUG FIX - Projectile Collision Event Duplication
*   **Issue**: Physics engine (Avian2D) reports multiple collision events per projectile per frame, causing massive multi-hit damage (12 hits instead of 3).
*   **Cause**: `despawn_recursive()` is deferred, so projectiles trigger multiple collision events before despawn executes.
*   **Fix**: Added `Local<HashSet<Entity>>` to `projectile_collision_system` to track processed projectiles per frame.
*   **Documentation**: This is a common physics engine pattern - always deduplicate collision events when immediate despawn is expected.

## 2025-12-24: BUG FIX - Hit Flash Not Visible
*   **Issue**: Hit flash effect triggered but wasn't visible.
*   **Cause**: Sprites default to white color (1.0, 1.0, 1.0) in Bevy. Flashing to white has no visible effect.
*   **Fix**: Changed `FLASH_COLOR` from white to bright red (`Color::srgb(1.0, 0.3, 0.3)`).

## 2025-12-24: Task 8.1.7 - Hit Flash on Ship Damage
*   Created `HitFlash` component in `src/components/hit_flash.rs`:
    *   Stores timer and original sprite color
    *   Default flash duration: 0.3 seconds
*   Created hit flash systems in `src/systems/hit_flash.rs`:
    *   `trigger_hit_flash_system`: Listens for `ShipHitEvent`, adds `HitFlash` component
    *   `update_hit_flash_system`: Lerps sprite color from red to original, removes component when done
    *   `lerp_color()`: Helper function for SRGBA color interpolation
*   Registered systems in `CombatPlugin` after projectile collision

## 2025-12-24: Task 8.1.6 - Screen Shake on Cannon Fire
*   Created `CameraShake` component in `src/components/camera.rs`:
    *   Trauma-based intensity system (shake intensity = trauma²)
    *   Configurable decay rate, max offset, max rotation
    *   Noise-based offset for smooth, organic shake pattern
*   Created `CannonFiredEvent` in `src/events/mod.rs`
*   Created camera shake systems in `src/systems/camera.rs`:
    *   `camera_shake_system`: Applies random offset/rotation based on trauma
    *   `trigger_camera_shake_on_fire`: Adds 0.3 trauma on cannon fire
*   Modified `cannon_firing_system` to emit `CannonFiredEvent`
*   Added `CameraShake::new()` to main camera spawn in `CorePlugin`
*   Registered event and systems in `CombatPlugin`

## 2025-12-24: Task 8.1.5 - UI Scroll/Dagger Decorations
*   Implemented decorative helper functions in `src/plugins/ui_theme.rs`:
    *   `draw_corner_flourishes()`: L-shaped scroll patterns with bezier curls at panel corners.
    *   `draw_ornamental_divider()`: Horizontal line with anchor symbol centerpiece and diamond endpoints.
    *   `draw_rope_divider()`: Wavy rope pattern with end knots for lighter separations.
    *   `draw_panel_border()`: Combines border stroke with corner flourishes.
    *   `draw_bezier_curve()`: Utility for quadratic bezier rendering via line segments.
    *   `INK_COLOR` constant for consistent sepia/ink styling.
*   Applied decorations to:
    *   `port_ui.rs`: Corner flourishes on panel, anchor divider after title, rope divider after tabs.
    *   `main_menu.rs`: Corner flourishes, anchor divider after title, rope divider before selection.
*   All decorations use egui's `Painter` API for procedural vector drawing.

## 2025-12-24: BUG FIX - EguiContexts Panic on State Transition
*   **Issue**: `port_ui_system` panicked with "EguiContexts::ctx_mut was called for an uninitialized context".
*   **Cause**: Systems using `EguiContexts` ran before `EguiSet::InitContexts` during state transitions.
*   **Fix**: Added `.after(EguiSet::InitContexts)` ordering to all egui-using systems:
    *   `port_ui.rs`: `port_ui_system`
    *   `main_menu.rs`: `main_menu_ui_system`
    *   `debug_ui.rs`: `debug_panel`
    *   `ui_theme.rs`: `configure_ui_theme`
*   **Documentation**: Added "Egui Systems" entry to `AGENT.md` Bevy Specifics section.

## 2025-12-23: Epic 8.5 - Historical Cartography (Tasks 8.5.0-8.5.1)
*   **8.5.0**: Integrated `bevy_prototype_lyon` v0.13 for vector graphics. Registered `ShapePlugin` in `WorldMapPlugin`.
*   **8.5.1**: Implemented coastline polygon extraction in `src/utils/geometry.rs`:
    *   `CoastlinePolygon` struct with CCW-ordered points (land-on-left invariant).
    *   `extract_contours()` function using Moore-neighbor contour tracing.
    *   Map borders treated as land to ensure closed polygons.
    *   `CoastlineData` resource stores extracted polygons.
    *   `extract_coastlines_system` runs on `Startup` after map generation.
*   **Design Decisions**:
    *   CCW winding order ensures consistent normal vectors for waterlining (seaward) and labeling (landward).
    *   Map border = land convention simplifies edge cases and guarantees closed contours.

## 2025-12-23: Refine Workflow
*   **Sync**: Updated `INDEX.md` with 5 new Epic 8.5 files (ink_reveal, typewriter, wake_effects, shader).
*   **Friction**: bevy_hanabi v0.14 API differs from docs (no `with_spawner` on ParticleEffect).
*   **Evolution**: Added bevy_hanabi API note to `AGENT.md` Bevy Specifics section.

## 2025-12-23: Epic 8.5 - Living Ink Effects
*   **8.5.1**: Created `InkReveal` component in `components/ink_reveal.rs` tracking tile position, start time, and animation progress with ease-out cubic easing.
*   **8.5.2**: Implemented fog reveal ink animation - replaced static `update_fog_tilemap_system` with animated `spawn_ink_reveals` and `animate_ink_reveals` systems for 0.5s smooth fog fade transitions.
*   **8.5.3**: Added ship wake ink trails using `bevy_hanabi` GPU particles - sepia-toned particles spawn behind moving ships with 1.5s lifetime and linear drag.
*   **8.5.4**: Implemented damage ink splatter VFX - added `ShipHitEvent` and one-shot particle burst (30 particles, 2s lifetime) triggered on projectile hits.
*   **8.5.5**: Added water ink wash shader effect - detects blue water pixels, samples neighbors for edge detection, applies subtle teal wash at coastline transitions.
*   **8.5.6**: Created `TypewriterText` component for UI write-on animation with `TypewriterRegistry` for egui integration. Used for main menu title animation.
*   **Design Decisions**:
    *   Used `bevy_hanabi` v0.14 for GPU-accelerated particles (wake trails, damage splatter).
    *   `ShipHitEvent` decouples combat damage from VFX spawning.
    *   Fog reveal uses `FogOfWar::take_newly_explored()` for efficient batch processing.
    *   Disabled crosshatch shading (set `crosshatch_enabled: 0`) per user request.

## 2025-12-22: Documentation Audit & Fix
*   **Audit**: Identified inconsistency where `docs/protocol` and `.agent/rules` were consolidated into `AGENT.md` but references were not updated.
*   **Fix**: Updated `AGENT.md` to be the single source of truth.
*   **Fix**: Updated `README.md` to point to the new structure.
*   **Fix**: Updated `workflows/*.md` to reference `AGENT.md` and `INDEX.md`.
*   **Restoration**: Restored `workflows/init.md` which was accidentally deleted.

# WORK LOG - 2025-12-18

## Phase 1: Project Foundations
### Epic 1.1: Project Setup [COMPLETED]
- Initialized Rust project with `cargo init`.
- Configured `Cargo.toml` with Bevy 0.14, Rapier, and other dependencies.
- Created the directory structure for plugins, components, systems, etc.
- Implemented minimal `main.rs` with "Pirates" window and camera.
- Verified project builds successfully.

### Tech Stack Modernization
- Upgraded to Bevy 0.15 (latest stable).
- Replaced `bevy_rapier2d` with `avian2d` (ECS-native physics).
- Added `leafwing-input-manager` for action-based input handling.
- Updated all dependencies to Bevy 0.15 compatible versions.
- Added best practices documentation to README (change detection, minimal commands).
- Updated WORK_PLAN.md with new input and physics tasks.
- Fixed deprecated `Camera2dBundle` to use `Camera2d` component.
### Epic 1.2: State Management [COMPLETED]
- Defined `GameState` enum for core game flow.
- Created `CorePlugin` to manage state registration and transitions.
- Implemented debug keybindings (1-5) for state switching.
- Added console logging for state changes.
- Verified compilation and baseline functionality.
### Epic 1.3: Camera & Input [COMPLETED]
- Moved 2D camera setup to `CorePlugin` and updated to Bevy 0.15 standards.
- Integrated `leafwing-input-manager` (0.16).
- Defined `PlayerAction` actions: `Thrust`, `TurnLeft`, `TurnRight`, `Fire`, `Anchor`, `CycleTarget`, `CameraMove`, `CameraZoom`.
- Created `InputPlugin` and default `InputMap` (WASD + Arrows/MouseMotion + Scroll + Space + Shift + Tab).
- Implemented `camera_control` system for resolution-aware panning and zooming.
- Fixed runtime panic by adding `#[actionlike(DualAxis)]` and `#[actionlike(Axis)]` attributes to `PlayerAction` variants (Leafwing 0.16 requirement).
- Verified compilation and baseline app stability.
### Epic 1.4: Debug UI [COMPLETED]
- Integrated `bevy_egui` (0.31) for immediate-mode debugging tools.
- Created `DebugUiPlugin` and registered it in `main.rs`.
- Implemented a "Debug Panel" window using `egui` that displays:
    - The current `GameState`.
    - Real-time FPS counter using `FrameTimeDiagnosticsPlugin`.
    - Buttons to trigger transitions between all game states (`MainMenu`, `Port`, `HighSeas`, `Combat`, `GameOver`).
- Verified implementation with `cargo check`.

### Epic 2.1: Physics Setup [COMPLETED]
- Integrated `avian2d` (0.2) physics engine.
- Created `PhysicsPlugin` and registered it in `main.rs`.
- Configured `FixedUpdate` frequency to 60Hz for deterministic physics.
- Set `Gravity` to zero for top-down perspective.
- Implemented `spawn_test_physics_entity` to verify physics behavior in `Combat` state.
- Verified compilation with `cargo check`.

### Bug Fix: Test Entity Not Visible in Combat View
- **Issue**: Test physics entity was spawning but not visible when entering Combat state.
- **Root Cause**: `MouseMove::default()` was mapped to `CameraMove` action, causing the camera to pan thousands of pixels away from origin on any mouse movement.
- **Fix**: Removed `MouseMove` from `CameraMove` mapping. Arrow keys still work for camera pan.
- **Lesson Learned**: `MouseMove::default()` reports raw mouse deltas *every frame* the mouse moves, not just when a button is held. For mouse-drag panning, use a modifier button and gate the action in the system.

### Epic 2.2: Ship Entity [COMPLETED]
- Defined `Ship` and `Player` marker components in `src/components/ship.rs`.
- Defined `Health` component with sails, rudder, and hull HP fields plus helper methods in `src/components/health.rs`.
- Defined `GoodType` enum, `GoodsTrait` enum, `Cargo` component, and `Gold` component in `src/components/cargo.rs`.
- Created `spawn_player_ship` function in `src/systems/ship.rs` with all required components:
    - Marker: `Ship`, `Player`
    - Data: `Health::default()`, `Cargo::new(100)`, `Gold(100)`
    - Visual: `Sprite` with texture from `assets/sprites/ships/player.png`
    - Physics: `RigidBody::Dynamic`, `Collider::rectangle`, `LinearDamping`, `AngularDamping`
- Created placeholder 64x64 ship sprite at `assets/sprites/ships/player.png`.
- Integrated `spawn_player_ship` into `PhysicsPlugin` to run on `OnEnter(GameState::Combat)`.
- Verified all changes compile with `cargo check`.

### Epic 2.3: Ship Movement [COMPLETED]
- Created `ship_movement_system` in `src/systems/movement.rs`.
- Added `Reverse` action to `PlayerAction` enum and bound to S key.
- Added `ExternalForce` component to player ship spawn for physics-based thrust.
- Created `CombatPlugin` in `src/plugins/combat.rs` to register combat systems.
- Implemented all 8 movement tasks:
    - **2.3.1**: System queries `Ship` + `Player` + physics components
    - **2.3.2**: W key applies forward thrust via `ExternalForce`
    - **2.3.3**: S key applies reverse thrust (half force)
    - **2.3.4**: A/D keys set `AngularVelocity` for rotation
    - **2.3.5**: Drag via `LinearDamping`/`AngularDamping` (set in spawn)
    - **2.3.6**: Shift key (Anchor) zeros velocity but allows rotation
    - **2.3.7**: Speed debuff - thrust and max speed scaled by `sails_ratio()`
    - **2.3.8**: Turn debuff - turn rate scaled by `rudder_ratio()`
- Movement system runs on `FixedUpdate`, gated by `GameState::Combat`.
- Verified all changes compile with `cargo check`.

### Physics Refinement & Anisotropic Drag [COMPLETED]
- **Advanced Physics Model**: Transitioned from direct velocity manipulation to a force-based system using `ExternalForce` and `ExternalTorque`.
- **Anisotropic Water Resistance (Keel Effect)**:
    - Implemented directional drag by decomposing world velocity into ship-local forward and lateral axes.
    - Set lateral drag significantly higher than longitudinal drag to simulate the keel's effect on steering.
    - Disabled isotropic `LinearDamping` in favor of this custom model.
- **Input Buffering Solution**:
    - Identified that `leafwing-input-manager` updates in `Update`, which can cause missed inputs for systems in `FixedUpdate`.
    - Introduced `ShipInputBuffer` resource to capture input states in `Update` and synchronizing them with physics systems in `FixedUpdate`.
- **Mass Calibration**:
    - Added explicit `Mass` and `AngularInertia` components to ensure predictable force-based acceleration.
    - Tuned thrust and torque values to meet the user's "10x speed" requirement while maintaining a heavy nautical feel.
- **Bevy 0.15 Bundle Optimization**:
    - Refactored ship spawning to use chained `.insert()` calls, bypassing the 15-component tuple limit.
- **Verified Behavior**: Ship now correctly "steers its momentum" (heading guides velocity) while allowing for realistic drifting at high speeds.

### Update: Visual Assets Integration
- **Assets**:
    - Replaced placeholder player ship with `ship (1).png` from Kenney Pirate Pack.
    - Added `enemy_ship.png` for future enemy implementation.
    - Added `map_tile.png` (tile_73) for future tilemap implementation.
- **Documentation**:
    - Updated `README.md` with visual references for Player Ship and Enemy Ship in aesthetic and entity sections.
    - Updated `WORK_PLAN.md` with visual reference for Tilemap.
- **Verification**:
    - Verified `spawn_player_ship` uses the correct asset path.
    - Ran `cargo check` to ensure no regressions.

### Visual Assets Integration & Ship Orientation Fix
- **Assets**: Replaced placeholder sprites with Kenney Pirate Pack assets:
  -  - Player ship sprite
  -  - Enemy ship sprite (ready for Epic 2.8)
  -  - Map tile (ready for Epic 3.1)
- **Ship Orientation Fix**: Added 180-degree rotation to ship spawn Transform to align Kenney sprites (face down) with physics forward direction (Y+).
- **Documentation**: Added visual references to README.md and WORK_PLAN.md. Added efficient logging guidance to README Quick Start.

### Epic 2.5: Damage & Hit Detection - Task 2.5.7 [COMPLETED]
- Implemented ship destruction when hull HP <= 0.
- **Event**: Added `ShipDestroyedEvent` to `events/mod.rs` with `entity` and `was_player` fields.
- **Systems**:
  - `ship_destruction_system`: Queries ships with `Health`, despawns those with `is_destroyed() == true`, emits event.
  - `handle_player_death_system`: Listens for `ShipDestroyedEvent`, transitions to `GameOver` when `was_player` is true.
- **Integration**: Registered event and systems in `CombatPlugin` with proper ordering constraints.
- Verified compilation with `cargo check`.

## 2025-12-19: Epic 2.6 - Loot System Complete

### Tasks Completed
- **2.6.1**: Defined `Loot` component with `value` and `good_type` fields in `components/loot.rs`
- **2.6.2**: Created loot sprite at `assets/sprites/loot/gold.png` (using Kenney cannonBall asset)
- **2.6.3**: Added loot spawning to `projectile_collision_system` at hit locations
- **2.6.4**: Loot entities are `RigidBody::Dynamic` with `Sensor` colliders
- **2.6.5**: Implemented `loot_collection_system` for player pickup → updates `Gold`/`Cargo`

### Implementation Details
- `Loot` component with `gold()` and `cargo()` constructors
- `LootTimer` for 30-second auto-despawn
- Loot spawns with random velocity and `LinearDamping` for drift effect
- Golden tint applied to sprite for visibility
- Registered `loot_collection_system` and `loot_timer_system` in `CombatPlugin`

### Files Changed
- `src/components/loot.rs` (NEW)
- `src/components/mod.rs` (loot module export)
- `src/systems/combat.rs` (spawn_loot, loot_collection_system, loot_timer_system)
- `src/plugins/combat.rs` (system registration)
- `assets/sprites/loot/gold.png` (NEW)

## 2025-12-19: Epic 2.7 - Current Zones Complete

### Tasks Completed
- **2.7.1**: Defined `CurrentZone` component with `velocity: Vec2` and `half_extents: Vec2`
- **2.7.2**: Created `current_zone_system` that applies `ExternalForce` to RigidBodies within zone bounds
- **2.7.3**: Implemented `spawn_test_current_zone` for visual testing in Combat state
- **2.7.4**: Zone visualization via semi-transparent blue sprite overlay

### Implementation Details
- `CurrentZone` uses AABB bounds checking with `contains()` method
- Force applied every physics tick to all entities within zone bounds
- Test zone spawns at (200, 0) with rightward push of 80 units/s
- Registered in `CombatPlugin` on `FixedUpdate` schedule

### Files Changed
- `src/components/current.rs` (NEW)
- `src/components/mod.rs` (current module export)
- `src/systems/combat.rs` (current_zone_system, spawn_test_current_zone)
- `src/plugins/combat.rs` (system registration)

## 2025-12-19: Epic 2.8 - Enemy Ships & AI Complete

### Tasks Completed
- **2.8.1**: Defined `AI` marker component in `components/ship.rs`
- **2.8.2**: Defined `Faction` component with `FactionId` enum (Pirates, NationA, NationB, NationC)
- **2.8.3**: Created `spawn_enemy_ship` function with full physics setup
- **2.8.4**: Enemy ship sprite already existed at `assets/sprites/ships/enemy.png`
- **2.8.5**: Created `CombatAISystem` in new `systems/ai.rs` module
- **2.8.6**: Implemented broadside circling behavior (circles to maintain perpendicular angle)
- **2.8.7**: Implemented AI firing logic (fires when player in broadside arc and range)
- **2.8.8**: Implemented flee behavior (triggers when hull HP < 20%)

### Implementation Details
- **AI Behavior**: Uses broadside circling strategy instead of direct pursuit
  - Circles around player to keep them perpendicular (in firing arc)
  - Blends closing/retreating with circular motion based on range
  - Flees when critically damaged
- **Components**: `AI`, `Faction(FactionId)`, `AIState`, `AICannonCooldown`
- **Resource**: `AIPhysicsConfig` for tunable AI parameters
- **Physics**: Same force-based model as player (anisotropic drag for keel effect)
- **Firing**: Same broadside spread pattern as player, 2-second cooldown

### Files Changed
- `src/components/ship.rs` (AI, Faction, FactionId)
- `src/systems/ship.rs` (spawn_enemy_ship function)
- `src/systems/ai.rs` (NEW - combat_ai_system, ai_firing_system, spawn_combat_enemies)
- `src/systems/mod.rs` (ai module export)
- `src/plugins/combat.rs` (AI system registration, AIPhysicsConfig resource)

## 2025-12-19: Tasks 2.5.5-2.5.6 - WaterIntake Complete

### Tasks Completed
- **2.5.5**: Implemented `WaterIntake` component in `components/health.rs`
- **2.5.6**: Modified `projectile_collision_system` to add/increase WaterIntake on hull damage

### Implementation Details
- `WaterIntake` component with `rate` (water per second) and `current` (accumulated water)
- Hull damage adds 0.1 rate per damage point
- If ship already has WaterIntake, rate is increased; otherwise component is added
- Helper methods: `new()`, `increase_rate()`, `tick()`

### Files Changed
- `src/components/health.rs` (added WaterIntake component)
- `src/systems/combat.rs` (modified projectile_collision_system)

## 2025-12-19: Epic 2.9 - Combat Flow Complete

### Tasks Completed
- **2.9.1**: Defined `CombatEndedEvent` with `victory: bool` field in `events/mod.rs`
- **2.9.2**: Implemented `combat_victory_system` to detect when all AI ships are destroyed
- **2.9.3**: Event emitted when all enemies destroyed (while player alive)
- **2.9.4-2.9.6**: Already implemented via `handle_player_death_system`
- **2.9.7**: Implemented `handle_combat_victory_system` to transition to HighSeas on victory

### Implementation Details
- `combat_victory_system`: Checks if AI ship query is empty while player exists
- `handle_combat_victory_system`: Transitions to `GameState::HighSeas` on victory event
- Both systems run in `Update` schedule, ordered after `ship_destruction_system`
- Event and systems registered in `CombatPlugin`

### Files Changed
- `src/events/mod.rs` (added CombatEndedEvent)
- `src/systems/combat.rs` (combat_victory_system, handle_combat_victory_system)
- `src/plugins/combat.rs` (event and system registration)

## 2025-12-19: Task 3.1.1 - Integrate bevy_ecs_tilemap

**Status**: Complete

**Changes**:
- Added `use bevy_ecs_tilemap::prelude::*;` import to `src/main.rs`
- Added `.add_plugins(TilemapPlugin)` to the Bevy app setup

**Verification**:
- `cargo check` passed successfully

**Notes**:
- The `bevy_ecs_tilemap = "0.15"` dependency was already in `Cargo.toml`
- This lays the foundation for Epic 3.1 (Tilemap Setup) and the High Seas map rendering

## 2025-12-19: Task 3.1.2 - Create tileset image

**Status**: Complete

**Changes**:
- Copied Kenney pirate-pack tilesheet to `assets/tilemaps/tileset.png`
- Tilesheet contains 96 tiles at 64x64 pixels each (water, land, coastlines, islands, etc.)

**Notes**:
- Using the pre-made Kenney tilesheet for consistency with existing art style
- Tiles are 64x64 with no margin between them

## 2025-12-19: Task 3.1.3 - Create TilemapPlugin

**Status**: Complete

**Changes**:
- Created `src/plugins/worldmap.rs` with `WorldMapPlugin`
- Plugin spawns tilemap on `GameState::HighSeas` entry
- Plugin despawns tilemap on `GameState::HighSeas` exit
- Test tilemap: 32x32 tiles with water and central land mass
- Uses Kenney tilesheet (64x64 tiles)

**Files**:
- `src/plugins/worldmap.rs` (new)
- `src/plugins/mod.rs` (updated)
- `src/main.rs` (updated)

**Notes**:
- Created custom `WorldMapPlugin` (not to be confused with `bevy_ecs_tilemap::TilemapPlugin`)
- Added `WorldMap` and `WorldMapTile` marker components for entity management
- Tilemap renders at z=-10 to appear behind ships

## 2025-12-19: Task 3.1.4 - Define MapData resource

**Status**: Complete

**Changes**:
- Created `src/resources/map_data.rs` with:
  - `TileType` enum: DeepWater, ShallowWater, Land, Sand, Port
  - `MapData` resource: 2D tile grid with accessor methods
  - Navigation helpers: `is_navigable()`, `in_bounds()`
  - Texture index mapping for rendering

**API**:
- `MapData::new(width, height)` - creates water-filled map
- `MapData::get(x, y)` / `MapData::set(x, y, tile)` - tile access
- `MapData::iter()` - iterate all tiles with coordinates
- `TileType::texture_index()` - get tileset index for rendering

## 2025-12-19: Task 3.1.5 - Spawn tilemap from MapData

**Status**: Complete

**Changes**:
- Updated `WorldMapPlugin` to use `MapData` resource instead of hardcoded values
- Added `initialize_test_map` system to create test islands
- Tiles now use `TileType::texture_index()` for proper tileset mapping
- Plugin now initializes `MapData` resource on app startup

**Test Map Features**:
- 64x64 tile map
- Main circular island with land, sand, and shallow water zones
- Secondary smaller island
- Port location on main island

**Notes**:
- Epic 3.1 (Tilemap Setup) is now complete
- Ready to proceed with Epic 3.2 (Procedural Generation)

## 2025-12-20: Tilemap Rendering Fix

**Problem**: Tilemap was showing incorrect textures (ship parts instead of terrain)

**Root Cause**: The Kenney pirate-pack tilesheet contains ship parts, ports, and mixed content - not proper water/land terrain tiles. The tile indices I used (0, 1, 17, 50) pointed to the wrong tiles.

**Solution**: 
- Created procedural tileset generation in `WorldMapPlugin::create_tileset_texture()`
- Tileset is a 320x64 image with 5 tiles (64x64 each):
  - Index 0: Deep Water (dark blue)
  - Index 1: Shallow Water (teal)
  - Index 2: Sand (tan)
  - Index 3: Land (green)
  - Index 4: Port (brown)
- Updated `TileType::texture_index()` to use correct indices (0-4)
- Added subtle color variation for visual interest

**Files Modified**:
- `src/plugins/worldmap.rs` - Added procedural tileset generation
- `src/resources/map_data.rs` - Fixed texture index mappings

## 2025-12-20: Tilemap Visibility Fix (RenderAssetUsages)

**Problem**: Tilemap was not visible despite tiles being spawned correctly

**Root Cause**: The procedural tileset image was created with `RenderAssetUsages::RENDER_WORLD` only, but it also needs `MAIN_WORLD` for the asset to be properly retained in the main world.

**Solution**: Changed `RenderAssetUsages::RENDER_WORLD` to `RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD`

**Result**: Tilemap now renders correctly showing:
- Dark blue deep water ocean
- Circular islands with green land, tan sand beaches, teal shallow water
- Port location (brown tile)

## 2025-12-20: Epic 3.1 Complete - Tilemap Setup

**Status**: Complete ✅

### Tasks Completed:
- **3.1.1**: Integrated `bevy_ecs_tilemap` plugin
- **3.1.2**: Created tileset image (copied Kenney tilesheet, later replaced with procedural)
- **3.1.3**: Created `WorldMapPlugin` for tilemap management
- **3.1.4**: Defined `MapData` resource with `TileType` enum
- **3.1.5**: Spawn tilemap from `MapData` with proper terrain rendering

### Key Implementation Details:
- **Procedural Tileset**: Created at runtime with proper colors:
  - Index 0: Deep Water (dark blue)
  - Index 1: Shallow Water (teal)
  - Index 2: Sand (tan)
  - Index 3: Land (green)
  - Index 4: Port (brown)
- **RenderAssetUsages Fix**: Must use `MAIN_WORLD | RENDER_WORLD` for procedural images
- **Centered Tilemap**: Offset by half map size so origin is at center

### Files Created/Modified:
- `src/plugins/worldmap.rs` (new)
- `src/resources/map_data.rs` (new)
- `src/plugins/mod.rs` (updated)
- `src/resources/mod.rs` (updated)
- `src/main.rs` (updated)

### Next Steps:
- Epic 3.2: Procedural Generation (noise-based world generation)

## 2025-12-20: Epic 3.2 Complete - Procedural Generation

**Status**: Complete ✅

### Tasks Completed:
- **3.2.1**: `noise` crate already in `Cargo.toml` - verified available
- **3.2.2**: Created `generate_world_map` function in `src/utils/procgen.rs`
- **3.2.3**: Implemented Fbm (Fractal Brownian Motion) noise for natural landmasses
- **3.2.4**: Ensured center spawn area (16x16 radius) is always navigable water
- **3.2.5**: Ports placed procedurally on coastlines (sand tiles adjacent to land and water)

### Implementation Details:
- **Map Size**: 512x512 tiles (user requested upgrade from 64x64)
- **Noise Algorithm**: Fbm (multi-octave Perlin) with 6 octaves, frequency 0.015
- **Radial Gradient**: Applied to push edges toward ocean, creating archipelago feel
- **Coastal Transitions**: Automatic shallow water around landmasses
- **Port Placement**: 8-15 ports with 50-tile minimum spacing
- **Random Seed**: New seed each game session for variety

### Tile Type Thresholds:
- `-1.0 to -0.1`: Deep Water
- `-0.1 to 0.05`: Shallow Water  
- `0.05 to 0.12`: Sand/Beach
- `0.12 to 1.0`: Land

### Files Created/Modified:
- `src/utils/procgen.rs` (NEW) - Procedural generation module
- `src/utils/mod.rs` - Added procgen module export
- `src/plugins/worldmap.rs` - Replaced test map with procedural generation

### Next Steps:
- Epic 3.3: Fog of War

## 2025-12-20
- Implemented `FogOfWar` resource (Task 3.3.1).
- Created `src/resources/fog_of_war.rs` and registered in `src/resources/mod.rs`.
- Verified with `cargo check`.
- Completed functional Fog of War implementation (Tasks 3.3.2 - 3.3.4).
- Added `Vision` component and visibility systems.
- Added a second tilemap layer for FOW visuals (parchment style).
- Added temporary High Seas player movement for testing.
- Fixed camera follow in High Seas view.
- Completed Wind System (Epic 3.4).
- Added `Wind` resource and `wind_system` for dynamic weather.
- Integrated wind display in Debug UI.
- Wind affects High Seas navigation speed and Combat ship physics.
- Completed Navigation System (Epic 3.5).
- Added click-to-navigate with A* pathfinding.
- Path visualization and port arrival detection.
- Removed temporary WASD movement.

## 2025-12-20: Theta* Pathfinding Upgrade

**Status**: Complete ✅

### Summary
Upgraded pathfinding from 4-directional A* with Manhattan distance to **Basic Theta*** for any-angle navigation. Ships now follow straight-line paths across open ocean rather than jagged grid-aligned routes.

### Key Changes to `pathfinding.rs`:
- **OrderedF32 wrapper**: Enables `f32` costs in `BinaryHeap` (which requires `Ord`)
- **Euclidean heuristic**: Replaced Manhattan distance for accurate cost estimation
- **8-way neighbors**: Diagonal movement with corner-cutting prevention
- **Line of Sight (LOS)**: Bresenham's line algorithm with supercover variant for diagonal walls
- **Theta* expansion**: Parent-to-neighbor LOS check enables path shortcuts

### Tests:
All 7 unit tests pass:
- `test_direct_path`
- `test_path_around_obstacle`
- `test_no_path_to_land`
- `test_line_of_sight_clear`
- `test_line_of_sight_blocked`
- `test_diagonal_movement`
- `test_corner_cutting_prevention`

### Files Modified:
- `src/utils/pathfinding.rs` (complete rewrite)


## 2025-12-20: Pathfinding Improvements & FPS Fix

**Status**: Complete ✅

### FPS Issue Fixed
- **Root Cause**: `FogOfWar.explore()` was mutating even for already-explored tiles, triggering Bevy's change detection and causing `update_fog_tilemap_system` to iterate 262k tiles every frame.
- **Fix**: Modified `explore()` to return early for known tiles. Added `newly_explored` tracking so tilemap only updates changed tiles.

### Coastal Penalty & Corner Cutting Prevention
- **5x Coastal Penalty**: Water tiles adjacent to land cost 5x more to traverse, pushing paths toward open water.
- **Supercover LOS**: Replaced Bresenham's line algorithm with supercover variant that checks ALL cells a line passes through, preventing corner cutting in Theta* paths.
- **1-Tile Shore Buffer**: Pathfinding now enforces that all waypoints (except the goal) stay 1+ tiles away from land. This gives Catmull-Rom smoothing room to curve without clipping corners.

### Path Smoothing
- **Catmull-Rom Splines**: Theta* waypoints become control points for smooth flowing curves (8 samples per segment).
- **Reflected Phantom Points**: Endpoints use reflection instead of duplication to prevent overshoot at path start/end.

### Files Modified
- `src/resources/fog_of_war.rs` - Efficient change tracking
- `src/systems/worldmap.rs` - Only update newly explored tiles
- `src/systems/combat.rs` - Removed excessive logging
- `src/systems/navigation.rs` - Catmull-Rom path smoothing
- `src/utils/pathfinding.rs` - Shore buffer, supercover LOS, coastal penalty


## 2025-12-20: Task 3.6.1 - SpatialHash Utility

Created `src/utils/spatial_hash.rs` with generic `SpatialHash<T>` struct for O(1) proximity queries.

**Features:**
- `insert()`, `remove()`, `clear()` for managing items
- `query(position, radius)` for circular range queries
- `query_rect(min, max)` for AABB queries
- 7 unit tests

**Also fixed:** Pre-existing broken test in `pathfinding.rs` (missing `goal` parameter).

## 2025-12-20: Task 3.6.2 - AI Ships on World Map

Added AI ship spawning on the High Seas world map.

**Changes:**
- Added `HighSeasAI` marker component in `worldmap.rs`
- Created `spawn_high_seas_ai_ships` system (spawns 5-10 random ships)
- Created `despawn_high_seas_ai_ships` system for cleanup
- Ships spawn at random navigable tiles with random factions

## 2025-12-20: AI Ship Visibility Improvements

Enhanced AI ship behavior on the High Seas map:

**Changes:**
- Increased AI ship count from 5-10 to 50 for better visibility
- Added `fog_of_war_ai_visibility_system` to hide AI ships in unexplored tiles
- Ships now only appear when the player has explored their location

## 2025-12-20: Task 3.6.3 - EncounterSystem

Created the EncounterSystem for detecting player proximity to AI ships.

**Changes:**
- Added `EncounterSpatialHash` resource using `SpatialHash<Entity>`
- Created `rebuild_encounter_spatial_hash` system to update positions each frame
- Created `encounter_detection_system` to log when player is within 128 units of AI ships
- Integrated systems into WorldMapPlugin Update schedule

## 2025-12-20: Tasks 3.6.4-3.6.6 - Encounter Combat Flow

Completed the encounter-to-combat system.

**Changes:**
- Added `CombatTriggeredEvent` to `events/mod.rs`
- Added hostility check: Pirates always hostile (3.6.4)
- Updated `encounter_detection_system` to emit `CombatTriggeredEvent` (3.6.5)
- Added `handle_combat_trigger_system` to transition to Combat state (3.6.6)
- Added `EncounterCooldown` resource to prevent rapid re-triggering

## 2025-12-20: Task 3.6.7 - Entity Transfer to Combat

Implemented encounter data transfer to combat scene.

**Changes:**
- Added `EncounteredEnemy` resource to store pending encounter faction
- Updated `handle_combat_trigger_system` to store encounter faction
- Modified `spawn_combat_enemies` to use `EncounteredEnemy` data
- Combat now spawns enemy with correct faction from encounter

## 2025-12-20: Epic 4.1 - Port Entity

Implemented the Port entity components and spawning system for Phase 4 (Ports & Economy).

### Changes

- **New**: `src/components/port.rs`
  - `Port` marker component
  - `PortName` component for display names
  - `Inventory` component with `InventoryItem` for goods, quantities, prices
  - Helper methods for buying/selling operations

- **New**: `src/plugins/port.rs`
  - `PortPlugin` structure
  - `spawn_port()` function to create port entities
  - `generate_random_inventory()` with base prices and randomized stock
  - `generate_port_name()` with thematic pirate-era names

- **Modified**: `src/plugins/worldmap.rs`
  - Added `spawn_port_entities` system on `OnEnter(HighSeas)`
  - Added `despawn_port_entities` system on `OnExit(HighSeas)`
  - Ports spawn at port tile locations with random faction assignment

### Tasks Completed
- [x] 4.1.1: Define `Port` marker component
- [x] 4.1.2: Define `Inventory` component
- [x] 4.1.3: Create `spawn_port` function
- [x] 4.1.4: Generate initial inventory for ports

## 2025-12-20: Epic 4.2 - Port UI

Implemented the Port View UI with tabbed panel layout using `bevy_egui`.

### Changes

- **New**: `src/plugins/port_ui.rs`
  - `PortUiPlugin` with `CurrentPort` and `PortUiState` resources
  - Tabbed layout: Market, Tavern, Docks, Contracts
  - Market panel shows goods from port `Inventory` with Buy/Sell buttons
  - Docks panel shows ship HP with progress bars and repair buttons
  - Tavern/Contracts panels with placeholders for future epics
  - Depart button to transition back to HighSeas

- **Modified**: `main.rs` - added `PortUiPlugin`

### Tasks Completed
- [x] 4.2.1: Create Port View layout
- [x] 4.2.2: Implement Market panel
- [x] 4.2.3: Implement Tavern panel
- [x] 4.2.4: Implement Docks panel
- [x] 4.2.5: Implement Contracts panel
- [x] 4.2.6: Implement Depart button

## 2025-12-20: Epic 4.3 - Trading

Implemented the trading system connecting port inventory to player cargo.

### Changes

- **Modified**: `src/events/mod.rs`
  - Added `TradeExecutedEvent` for buy/sell transactions

- **Modified**: `src/plugins/port_ui.rs`
  - Market panel now shows player gold and cargo capacity in header
  - Buy/Sell buttons validate gold/stock/capacity before enabling
  - Added `trade_execution_system` to process `TradeExecutedEvent`
  - Auto-selects first available port for testing

- **Modified**: `src/plugins/worldmap.rs`
  - Added `Cargo` and `Gold` components to High Seas player spawn

### Tasks Completed
- [x] 4.3.1: Create MarketSystem
- [x] 4.3.2: Implement buy goods
- [x] 4.3.3: Implement sell goods
- [x] 4.3.4: Implement cargo capacity check
- [x] 4.3.5: Emit TradeExecutedEvent

## 2025-12-20: Skipping Epic 4.4 (Price Dynamics)

> [!NOTE]
> **Deferred**: Epic 4.4 (Price Dynamics) is skipped for now because it depends on `WorldClock` from Epic 5.1 (World Tick). We will return to implement 4.4 after completing Phase 5.1.

Proceeding to Epic 4.5 (Contracts) instead.

## 2025-12-20: Epic 4.5 - Contracts

Implemented the contract/quest system for jobs.

### Changes

- **New**: `src/components/contract.rs`
  - `Contract` marker component
  - `ContractType` enum (Transport, Explore, Escort, Hunt)
  - `ContractDetails` with factory methods
  - `AcceptedContract` marker component
  - `ContractProgress` tracking component

- **Modified**: `src/events/mod.rs`
  - Added `ContractAcceptedEvent`, `ContractCompletedEvent`

- **Modified**: `src/plugins/port_ui.rs`
  - Added `PlayerContracts` resource
  - Added `generate_port_contracts` system (OnEnter)
  - Updated Contracts panel with available/active lists
  - Added `contract_acceptance_system`

### Tasks Completed
- [x] 4.5.1-4.5.5: Contract components, generation, acceptance, tracking, completion
- [~] 4.5.6: Contract expiry deferred (requires WorldClock from 5.1)

## 2025-12-20: Epic 5.1 World Tick (5.1.1, 5.1.2, 5.1.3)

### Changes Made
- **[NEW]** `src/resources/world_clock.rs`: `WorldClock` resource with `day`, `hour`, `tick` fields
  - `advance()` method for tick progression
  - `formatted_time()` for HUD display
  - 6 unit tests for clock behavior
- **[NEW]** `src/systems/world_tick.rs`: `world_tick_system` running on `FixedUpdate`
- **[MODIFIED]** `src/plugins/core.rs`: Added `WorldClock` resource init and `world_tick_system` registration
- **[MODIFIED]** `src/plugins/debug_ui.rs`: Added "World Clock" section showing "Day X, Hour Y"
- **[MODIFIED]** `src/resources/mod.rs`: Exported `world_clock` module
- **[MODIFIED]** `src/systems/mod.rs`: Exported `world_tick` module

### Design Notes
- Clock runs unconditionally across all game states
- At 60Hz FixedUpdate: 1 hour ≈ 1 second, 1 day ≈ 24 seconds
- `TICKS_PER_HOUR = 60` constant controls time scaling

### Verification
- `cargo check`: PASSED
- `cargo test resources::world_clock`: 6/6 tests passed

## 2025-12-21: Task 4.4.1 - Create PriceCalculationSystem

### Changes Made
- **[NEW]** `src/systems/economy.rs`: `price_calculation_system` with supply-based pricing
  - Low stock increases prices, high stock decreases prices
  - Price formula: `base_price * (supply_ratio ^ -sensitivity)`
  - Configurable via `price_config` module constants
  - 5 unit tests for price behavior
- **[MODIFIED]** `src/systems/mod.rs`: Exported `economy` module
- **[MODIFIED]** `src/plugins/core.rs`: Added `price_calculation_system` to `FixedUpdate` after `world_tick_system`

### Verification
- `cargo check`: PASSED
- `cargo test systems::economy`: 5/5 tests passed

## 2025-12-21: Task 4.4.2 - Adjust prices based on supply

**Status**: Already implemented as part of 4.4.1.

The `calculate_supply_price` function directly implements supply-based pricing:
- Low stock → supply_ratio small → multiplier large → higher price
- High stock → supply_ratio large → multiplier small → lower price

Verified by existing tests: `test_low_supply_increases_price`, `test_high_supply_decreases_price`

## 2025-12-21: Task 4.4.3 - Adjust prices based on demand (global)

### Changes Made
- **[MODIFIED]** `src/systems/economy.rs`:
  - Added `GlobalDemand` resource tracking demand per good type
  - Updated `calculate_price` to include demand multiplier
  - Added `DEMAND_SENSITIVITY` constant (0.5)
  - Added 3 new tests for demand-based pricing
- **[MODIFIED]** `src/plugins/core.rs`: Initialize `GlobalDemand` resource

### Price Formula
```
price = base_price * supply_multiplier * demand_multiplier
demand_multiplier = global_demand ^ demand_sensitivity
```

### Verification
- `cargo check`: PASSED
- `cargo test systems::economy`: 8/8 tests passed

## 2025-12-21: Task 4.4.4 - Implement goods decay (perishables)

### Changes Made
- **[MODIFIED]** `src/systems/economy.rs`:
  - Added `PERISHABLE_DECAY_RATE` constant (0.0001 per tick)
  - Added `goods_decay_system` for perishable goods decay
  - System reduces quantity of perishable goods over time
- **[MODIFIED]** `src/plugins/core.rs`: Added `goods_decay_system` to `FixedUpdate`

### Decay Mechanics
- Goods with `GoodsTrait::Perishable` (Rum, Sugar) decay
- ~0.6% lost per hour, ~14% per day at 60Hz tick rate
- Reduces quantity, not price directly

### Verification
- `cargo check`: PASSED
- `cargo test systems::economy`: 8/8 tests passed

---

## Epic 4.4 Price Dynamics - COMPLETE

All 4 tasks completed:
- 4.4.1: PriceCalculationSystem ✅
- 4.4.2: Supply-based pricing ✅
- 4.4.3: Demand-based pricing ✅
- 4.4.4: Perishable goods decay ✅

## 2025-12-21: Task 4.5.6 - Implement contract expiry

### Changes Made
- **[MODIFIED]** `src/components/contract.rs`:
  - Added `expiry_tick: Option<u32>` field to `ContractDetails`
  - Added `DEFAULT_DURATION_TICKS` constant (2 days = 2880 ticks)
  - Added `transport_with_expiry()` and `explore_with_expiry()` constructors
  - Added `is_expired()` method
- **[NEW]** `src/systems/contract.rs`: `contract_expiry_system` with 3 unit tests
- **[MODIFIED]** `src/events/mod.rs`: Added `ContractExpiredEvent`
- **[MODIFIED]** `src/systems/mod.rs`: Exported `contract` module
- **[MODIFIED]** `src/plugins/core.rs`: Registered event and system on `FixedUpdate`

### Expiry Mechanics
- Contracts can have optional `expiry_tick`
- System checks against `WorldClock::total_ticks()` each tick
- Expired contracts emit `ContractExpiredEvent` and are despawned
- Default duration: 2 in-game days (~48 real seconds at 60Hz)

### Verification
- `cargo check`: PASSED
- `cargo test systems::contract`: 3/3 tests passed

---

## Epic 4.5 Contracts - COMPLETE

All 6 tasks completed:
- 4.5.1-4.5.5: Previously completed
- 4.5.6: Contract expiry ✅

## 2025-12-21: Epic 4.6 Ship Repair - COMPLETE

### Changes Made
- **[NEW]** `src/systems/repair.rs`:
  - `repair_execution_system` handling sails, rudder, hull repairs
  - `calculate_repair_cost()` function with configurable costs per HP
  - Sails: 1g/HP, Rudder: 1.5g/HP, Hull: 2g/HP
  - Hull repair removes `WaterIntake` component
  - 4 unit tests for cost calculation
- **[MODIFIED]** `src/events/mod.rs`:
  - Added `RepairType` enum (Sails, Rudder, Hull)
  - Added `RepairRequestEvent`
- **[MODIFIED]** `src/systems/mod.rs`: Exported `repair` module
- **[MODIFIED]** `src/plugins/port_ui.rs`:
  - Added `repair_events` EventWriter to `port_ui_system`
  - Updated `render_docks_panel` to display repair costs
  - Repair buttons show cost and enabled/disabled based on gold
  - Emits `RepairRequestEvent` on button click

### Tasks Completed
- 4.6.1: Create RepairSystem ✅
- 4.6.2: Implement repair sails ✅
- 4.6.3: Implement repair rudder ✅
- 4.6.4: Implement repair hull ✅
- 4.6.5: Display repair costs ✅

### Verification
- `cargo check`: PASSED
- `cargo test systems::repair`: 4/4 tests passed

## 2025-12-21: Protocol Evolution

**Changes**:
- **MANIFESTO.md**: Added "Task Splitting" directive. Complex tasks in `WORK_PLAN.md` must be split into atomic sub-tasks (e.g., 5.3.5a, 5.3.5b) before implementation.
- **INVARIANTS.md**: Added "Simulation Layers" section. Explicitly defined the separation between visual-only High Seas entities (no physics, cached pathfinding) and physical Combat entities (Avian physics, colliders).

**Reasoning**:
- **Task Splitting**: Reduces context load and allows for clearer progress tracking on complex features.
- **Simulation Layers**: Prevents confusion about which systems apply to which game state, ensuring performance (physics only where needed) and correctness.

## 2025-12-21: Protocol Evolution - Entity Persistence Patterns

**Friction Point**:
- When implementing Ship Capture (5.4.1), the agent created `PlayerFleet` to persist ship data but forgot to create a companion resource to track spawned entity IDs. This made it impossible to link Fleet UI selections to the live entities.

**Mutation Applied**:
- **INVARIANTS.md Section 8**: Added "Entity Persistence Patterns" rule. When a persistent resource spawns transient entities, a companion resource must map indices to Entity IDs.

**Example**:
- `PlayerFleet` (persistent) + `FleetEntities` (transient).

## 2025-12-22: Epic 6.2 Companions [COMPLETED]

**Summary**: Implemented all companion abilities for the Companions system.

### Files Modified
- `src/systems/navigation.rs`: Navigator ability (+25% speed bonus)
- `src/systems/worldmap.rs`: Lookout ability (+50% vision radius)
- `src/systems/combat.rs`: Gunner ability (-30% cannon cooldown)
- `docs/protocol/INVARIANTS.md`: Section 12 - Companion Ability Pattern

### Tasks Completed
- 6.2.7: Navigator ability ✅
- 6.2.8: Lookout ability ✅
- 6.2.9: Gunner ability ✅

### Verification
- `cargo check`: PASSED

## 2025-12-22: Epic 7.1 Meta Profile [COMPLETED]

**Summary**: Implemented meta-progression with persistent player profile.

### Files Created/Modified
- `src/resources/meta_profile.rs`: MetaProfile resource with stats, unlocks, wrecks
- `src/plugins/core.rs`: Load on Startup, save on GameOver
- `Cargo.toml`: Added `dirs` and `serde_json` dependencies
- `docs/protocol/INVARIANTS.md`: Section 13 - MetaProfile Persistence

### Tasks Completed
- 7.1.1: Define MetaProfile resource ✅
- 7.1.2: Load on app start ✅
- 7.1.3: Save on death/quit ✅
- 7.1.4: Define player stats ✅
- 7.1.5: Implement stat progression ✅

### Verification
- `cargo check`: PASSED

---

## 2025-12-21: Epic 7.2 Archetypes [COMPLETED]

**Summary**: Implemented starting archetype selection for roguelike variety.

### Files Created/Modified
- `src/components/ship.rs`: Added `ShipType` enum (Sloop, Frigate, Schooner, Raft)
- `src/resources/meta_profile.rs`: Added `ArchetypeConfig`, `ArchetypeRegistry`, `UnlockCondition`
- `src/plugins/core.rs`: Registered `ArchetypeRegistry`, added `check_archetype_unlocks` system
- `src/plugins/main_menu.rs`: **NEW** - MainMenu UI with archetype selection
- `src/plugins/mod.rs`: Added `main_menu` module
- `src/main.rs`: Registered `MainMenuPlugin`
- `src/plugins/worldmap.rs`: Modified `spawn_high_seas_player` to apply archetype bonuses
- `docs/protocol/INVARIANTS.md`: Section 14 - Archetype System
- `docs/protocol/INDEX.md`: Added new key file entries

### Tasks Completed
- 7.2.1a: Define Archetype enum and ArchetypeConfig struct ✅
- 7.2.1b: Create ArchetypeRegistry resource ✅
- 7.2.1c: Add unlocked_archetypes to MetaProfile ✅
- 7.2.2: Implement check_archetype_unlocks system ✅
- 7.2.3: Implement archetype selection UI ✅
- 7.2.4: Apply archetype bonuses on game start ✅

### Archetypes Defined
| Archetype | Gold | Ship | Unlock Condition |
|-----------|------|------|------------------|
| Default (Freebooter) | 500 | Sloop | Always |
| RoyalNavyCaptain | 1000 | Frigate | 5 runs completed |
| Smuggler | 300 | Schooner | 10,000 lifetime gold |
| Castaway | 0 | Raft | Die within 24 hours |

### Verification
- `cargo check`: PASSED

---

## 2025-12-21: Epic 7.3 Legacy Wrecks [COMPLETED]

**Summary**: Implemented legacy wreck system for roguelike persistence across deaths.

### Files Created/Modified
- `src/resources/meta_profile.rs`: Added `PlayerDeathData` resource
- `src/systems/combat.rs`: Modified `ship_destruction_system` to capture death data
- `src/plugins/core.rs`: Enhanced `save_profile_on_death` to create wrecks
- `src/plugins/worldmap.rs`: Added `LegacyWreckMarker`, spawn/despawn/exploration systems
- `docs/protocol/INVARIANTS.md`: Section 15 - Legacy Wreck System

### Tasks Completed
- 7.3.1: Record wreck on player death ✅
- 7.3.2: Spawn legacy wrecks on new run ✅
- 7.3.3: Implement wreck exploration ✅

### System Flow
1. Player dies → `ship_destruction_system` captures position/gold/cargo in `PlayerDeathData`
2. `OnEnter(GameOver)` → `save_profile_on_death` creates `LegacyWreck`, saves to profile
3. New run → `spawn_legacy_wrecks` places wreck entities on map
4. Player approaches wreck → `wreck_exploration_system` transfers loot, removes wreck

### Verification
- `cargo check`: PASSED

## 2025-12-22: Task 8.1.1 - Ink and Parchment Shader

**Summary**: Created the WGSL shader for the game's signature visual style.

### Files Created
- `assets/shaders/ink_parchment.wgsl`: Post-processing shader that converts the scene to grayscale and maps luminance to "Ink" (dark brown) and "Parchment" (cream) colors.

### Tasks Completed
- 8.1.1: Create "Ink and Parchment" shader ✅

### Notes
- The shader uses a hardcoded palette:
  - Paper: `vec3(0.93, 0.88, 0.78)`
  - Ink: `vec3(0.15, 0.12, 0.10)`
- Contrast is boosted (`pow(gray, 1.2)`) to simulate the sharpness of drawn ink.

## 2025-12-22: Task 8.1.2 - Post-Processing Application

**Summary**: Applied the Ink and Parchment shader to the game camera using a full Render Graph implementation.

### Files Created/Modified
- `src/plugins/graphics.rs`: Created `GraphicsPlugin` which sets up the `PostProcessNode` in the render graph.
- `src/plugins/core.rs`: Added `InkParchmentSettings` to the main camera spawn.
- `src/main.rs`: Registered `GraphicsPlugin`.
- `docs/protocol/INVARIANTS.md`: Added section on Post-Processing Architecture.

### Tasks Completed
- 8.1.2: Apply shader as post-processing ✅

### Implementation Details
- Used Bevy 0.15 `ViewNode` pattern for post-processing.
- Inserted custom node into `Core2d` graph after `Tonemapping`.
- Settings are controlled via `InkParchmentSettings` component on the Camera2d entity.

## 2025-12-22: Task 8.1.0 - Fix EguiUserTextures Panic

**Summary**: Fixed a startup panic caused by missing `EguiPlugin`.

### Files Modified
- `src/main.rs`: Added `add_plugins(EguiPlugin)` to the app initialization.

### Tasks Completed
- 8.1.0: Fix EguiUserTextures panic on startup ✅

### Notes
- The panic `could not access system parameter ResMut<'_, EguiUserTextures>` occurred because `EguiPlugin` was not registered, despite being imported. `DebugUiPlugin` and `FleetUiPlugin` depend on resources provided by `EguiPlugin`.

## 2025-12-22: Debugging Task 8.1.2 - Shader Visibility

**Summary**: Fixed issue where the post-processing shader was not visible.

### Files Modified
- `assets/shaders/ink_parchment.wgsl`: Reverted debug color change (verified shader logic is correct).
- `src/plugins/graphics.rs`: Changed `TextureFormat` from `Bgra8UnormSrgb` to `Bgra8Unorm` to match the swapchain/view format on Metal/M1.

### Notes
- The pipeline likely failed to bind because of a texture format mismatch between the render pass output (ViewTarget) and the pipeline definition. Bevy 0.15's default view format is typically `Bgra8Unorm` on macOS.

## 2025-12-22: Refactor Task 8.1.2 - Specialized Render Pipeline

**Summary**: Upgraded the Post-Processing pipeline to use `SpecializedRenderPipeline`.

### Files Modified
- `src/plugins/graphics.rs`: Implemented `SpecializedRenderPipeline` for `PostProcessPipeline`. The pipeline now dynamically adapts to the output texture format of the view (`view_target.out_texture_format()`).

### Notes
- This prevents crashes caused by TextureFormat mismatches between the pipeline and the render pass (wgpu validation errors). The previous hardcoded `Bgra8Unorm` crashed on `Bgra8UnormSrgb` swapchains. The specialized pipeline handles any format.

## 2025-12-22: Refinement - Process Improvements

**Summary**: Updated project documentation to prevent recurring friction points identified during development.

### Friction Points Addressed
- **`WORK_LOG.md` Integrity**: Agent repeatedly overwrote the work log instead of appending, causing data loss.
- **Render Pipeline Fragility**: Hardcoding texture formats led to visibility bugs and crashes. `SpecializedRenderPipeline` pattern should be used instead.
- **Missing Dependencies**: Panics occurred due to plugins like `EguiPlugin` not being registered in `main.rs`.

### Process Improvements Implemented
- **INVARIANTS.md**:
  - Added "Log Integrity" rule to `Documentation Standards`, mandating append-only updates.
  - Updated `Post-Processing Architecture` to require `SpecializedRenderPipeline` to prevent format-related crashes.
- **INDEX.md**:
  - Added `src/plugins/graphics.rs` to the key file index for better discoverability.

## 2025-12-23: Post-Processing Shader Fix

**Summary**: Fixed critical shader crash and visibility issues in the Ink and Parchment post-processing effect.

### Issues Fixed
1. **Index Out of Bounds Crash**: `queue_render_pipeline()` was called every frame in `run()`, creating new pipeline IDs without proper caching.
2. **Invisible Shader**: Used `main_texture_view()` + `out_texture()` which don't provide proper double-buffering for read/write operations.
3. **Texture Format Mismatch**: Hardcoded `Bgra8UnormSrgb` (swapchain format) instead of `Rgba8UnormSrgb` (internal render texture format).

### Solution Applied
- **Cached Pipeline ID**: Store `CachedRenderPipelineId` in `PostProcessPipeline` during `FromWorld`
- **Double-Buffering**: Use `view_target.post_process_write()` which provides `source` and `destination` textures
- **Correct Format**: Use `TextureFormat::Rgba8UnormSrgb` for non-HDR 2D camera internal textures

### Files Modified
- `src/plugins/graphics.rs` - Complete rewrite of pipeline caching and render pass logic

### Verification
- `cargo run` succeeds without crashes
- Shader effect visible in High Seas view## 2025-12-22 23:44 - UI Styling and Warnings
- Implemented 'UiThemePlugin' to apply 'parchment.png' background to UI panels.
- Configured Egui visual style (colors, transparency) to match 'Ink & Parchment' aesthetic.
- Fixed 'bevy_egui' panic by resolving 'ResMut' conflict.
- Fixed UI texture tiling by implementing manual UV tiling in 'draw_parchment_bg'.
- Fixed 'InkParchmentSettings' fields warning in 'graphics.rs'.
- Fixed duplicate import warning in 'port_ui.rs'.

## 2025-12-23: Epic 8.3.1-8.3.2 - Weathered Document Foundation

**Summary**: Expanded post-processing pipeline to support paper texture overlay and vignette effects.

### Task 8.3.1: Paper Texture Overlay
- **Expanded `AestheticSettings`**: Replaced minimal `InkParchmentSettings` with full settings struct:
  - `paper_texture_strength`, `vignette_strength`, `vignette_radius`
  - `grain_strength`, `grain_scale`, `stain_strength`, `ink_feather_radius`, `time`
- **Extended Bind Group Layout**: Added paper texture (binding 2), paper sampler (binding 3), settings uniform (binding 4)
- **Texture Loading**: Created `PaperTextureHandle` resource with `ExtractResourcePlugin` for render world access
- **Shader Enhancement**: Paper texture sampled with tiling (3x scale) and slight rotation (0.02 rad) to break grid alignment
- **Paper Blending**: Luminance deviation from 0.5 modulates scene color, weighted by brightness

### Task 8.3.2: Vignette Darkening
- **Radial Darkening**: Distance-based vignette from screen center using `smoothstep`
- **Asymmetric Shaping**: Y-axis scaled 1.1x with -0.05 offset for heavier bottom (aged document effect)
- **Palette Preservation**: Vignette darkens toward `INK_COLOR` rather than black
- **Configurable**: `vignette_radius` (default 0.4) and `vignette_strength` (default 0.4) tunable

### Files Modified
- `src/plugins/graphics.rs` - Complete pipeline expansion with new bindings and settings
- `src/plugins/core.rs` - Updated camera spawn to use `AestheticSettings`
- `assets/shaders/ink_parchment.wgsl` - Paper overlay and vignette effects
- `Cargo.toml` - Added `bytemuck` dependency for uniform buffer serialization

### Verification
- `cargo check`: PASSED
- `cargo run`: Shader renders correctly with paper texture and vignette visible

## 2025-12-23: Task 8.3.3 - Procedural Paper Grain Noise

**Summary**: Added FBM noise for subtle paper fiber texture.

### Implementation
- **Noise Functions**: Added `hash21`, `noise2d`, and `fbm` to shader
- **FBM Parameters**: 4 octaves, controlled by `grain_scale` (default 100.0)
- **Animation**: Slow time-based shift (0.001 * time) for organic feel
- **Luminance Modulation**: Grain more visible on lighter paper areas

### Files Modified
- `assets/shaders/ink_parchment.wgsl` - Added noise functions and grain effect

### Verification
- `cargo check`: PASSED
- Visual test: Grain visible at boosted strength (0.3), subtle at default (0.08)

## 2025-12-23: Tasks 8.4.1-8.4.2 - Sobel Edge Detection & Ink Strokes

**Summary**: Implemented hand-drawn linework effect via edge detection.

### Task 8.4.1: Sobel Edge Detection
- **Luminance Function**: Extracted to reusable `luminance()` helper
- **Sobel Operator**: Classic 3x3 neighborhood sampling for gradient magnitude
- **Texel Calculation**: Uses `textureDimensions()` for resolution-independent sizing

### Task 8.4.2: Edge Rendering
- **Soft Threshold**: `smoothstep(threshold, threshold + 0.1, edge)` for smooth transitions
- **Muted Fills**: Non-edge areas pushed 30% toward paper color
- **Ink Strokes**: Detected edges rendered in `INK_COLOR`
- **Toggle**: `edge_detection_enabled` uniform for performance/accessibility

### Files Modified
- `src/plugins/graphics.rs` - Added `edge_detection_enabled` and `edge_threshold` fields
- `assets/shaders/ink_parchment.wgsl` - Added edge detection and rendering

### Verification
- `cargo check`: PASSED
- Audit: No temporal words, proper struct alignment

## 2025-12-23: Task 8.5.7 - Compass Rose UI (Complete)

Implements a 32-point compass rose using `bevy_prototype_lyon` vector graphics, rendered via a dedicated Overlay Camera for visual stability.

### Tasks Completed
*   **8.5.7**: Created `CompassRosePlugin` and `compass_rose.rs` module.
*   **Design**: Implemented 32-point Rose of the Winds with layered spikes (Red/Green/Gold) and decorative center.
*   **Refactor**: Replaced initial camera-child implementation with **Overlay Camera** approach using `RenderLayers(1)`.
    *   Eliminates jitter during camera movement.
    *   Ensures consistent vector quality at all zoom levels.
    *   Decouples UI positioning from world camera transform.
*   **Fix**: Resolved input ambiguity (click-through issue) by adding `MainCamera` marker component to the primary world camera.

## 2025-12-23: Phase 8 - Refine
*   **Sync**: Updated `INDEX.md` to include `src/plugins/compass_rose.rs`.
*   **Retrospective**: Identified that adding an Overlay Camera inherently introduces input ambiguity for world-space queries (clicks, mouse position).
*   **Evolution**: Documented the "Vector UI / Overlay Camera" architectural pattern in `AGENT.md`, specifically noting the need for a `MainCamera` marker component to disambiguate input handling.
