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
41: 
42: ### Epic 2.1: Physics Setup [COMPLETED]
43: - Integrated `avian2d` (0.2) physics engine.
44: - Created `PhysicsPlugin` and registered it in `main.rs`.
45: - Configured `FixedUpdate` frequency to 60Hz for deterministic physics.
46: - Set `Gravity` to zero for top-down perspective.
47: - Implemented `spawn_test_physics_entity` to verify physics behavior in `Combat` state.
48: - Verified compilation with `cargo check`.

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

