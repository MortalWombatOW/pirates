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
