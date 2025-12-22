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
* **Commands vs Mutation**: Prefer direct component mutation over `Commands` for performance. Use `Commands` only for entity spawning/despawning or structural changes (adding/removing components).
* **System Sets**: All systems must be registered to a plugin. Use `Update` for logic and `FixedUpdate` for physics/simulation.

## 3. Physics & Simulation
* **Timestep**: Physics runs on `FixedUpdate` (60Hz deterministic).
* **Anisotropic Drag**: Ships do NOT use isotropic `LinearDamping`. They use custom force calculations to simulate "Keel Effect" (high lateral drag, low forward drag).
* **Input Buffering**: `leafwing` inputs update in `Update`. To prevent missed inputs in `FixedUpdate`, inputs must be buffered into a `ShipInputBuffer` resource.
* **Simulation Layers**:
    * **Combat**: Uses `FixedUpdate`, `avian2d` physics, and colliders. Real-time deterministic simulation.
    * **High Seas**: Visual representation only. NO physics colliders. AI uses `RouteCache` and `Theta*` to pathfind around land, but moves via simple transform interpolation. Segregate these logic layers via `HighSeasAI` vs `CombatAI` marker components.

## 4. Documentation Standards
* **Temporal Contamination**: Comments must never describe *changes*.
    * *Forbidden*: `// Fixed bug where ship spins`
    * *Required*: `// Clamps angular velocity to prevent spinning`
* **Source of Truth**: `README.md` is the GDD. This file (`INVARIANTS.md`) is the Technical Constraint list.

## 5. UI & Rendering
* **Debug UI**: Use `bevy_egui` for all debug tools.
* **Camera**: Use `Camera2d` component. Panning/Zooming logic must handle resolution scaling.

## 6. Faction AI System
* **Schedule**: `FactionRegistry` is a global resource, NOT a component. Faction AI systems run on `FixedUpdate` for deterministic simulation.
* **Hourly Ticks**: `faction_ai_system` uses `WorldClock.tick == 0` to run once per in-game hour, not every frame.
* **Daily Events**: Route generation runs at midnight (`hour == 0`), ship spawning at hour 6. This sequencing ensures routes exist before ships spawn.
* **Threat Detection**: `faction_threat_response_system` runs on `Update` (not `FixedUpdate`) because it's proximity-based and needs immediate response, not deterministic replay.
* **Hostility Threshold**: Factions are hostile when `player_reputation < -50`. Pirates start at `-100` (always hostile).

## 7. Order Execution System
* **No Change Detection**: `order_execution_system` intentionally does NOT use `Changed<OrderQueue>` because:
  1. It must check if navigation completed (path becomes empty)
  2. Order state mutates internally (e.g., `outbound` flag toggles)
* **Navigation Integration**: Orders set `Destination` component, which triggers `pathfinding_system`. Ships need both `OrderQueue` AND navigation components.
* **Port Entity References**: `TradeRoute` orders store port `Entity` IDs. If a port is despawned (leaving HighSeas), routes become invalid and are cleaned up.
* **Order Cycling**: Repeating orders (TradeRoute, Patrol) pop themselves and push a modified copy to implement loops.

## 8. Entity Persistence Patterns
* **Companion Resources**: When a persistent `Resource` (e.g., `PlayerFleet`) stores data that becomes spawned entities, create a **companion resource** (e.g., `FleetEntities`) to map indices to `Entity` IDs.
  * Populate the companion resource in the `OnEnter` system that spawns entities.
  * Clear the companion resource in the `OnExit` system.
  * This pattern enables UI/systems to associate persistent data indices with live entities.
* **Example**:
    ```rust
    // Persistent data
    #[derive(Resource, Default)]
    pub struct PlayerFleet { pub ships: Vec<ShipData> }

    // Entity tracking (transient)
    #[derive(Resource, Default)]
    pub struct FleetEntities { pub entities: Vec<Entity> }
    ```

## 9. Contract Delegation (Subcontracting)
* **Simplified Cargo Handling**: Fleet ships assigned to Transport contracts do NOT actually load/unload cargo. They simply navigate to the destination port; arrival triggers contract completion.
  * Rationale: Fleet ships use `ShipData` for persistence (data-only), not full `Cargo` components. Simulating cargo transfers would require significant additional systems.
  * Trade-off: Gamified abstraction acceptable for MVP. Full simulation can be added in a future pass.
* **Player Cut**: The `AssignedShip` component stores `player_cut` (default 70%). The remaining 30% represents "fleet overhead" and is not explicitly tracked—it's simply not awarded.
* **Schedule**: `contract_delegation_system` runs on `Update` (not `FixedUpdate`) because it's proximity-based and needs immediate response, similar to threat detection.

## 10. Intel System
* **Duplicate `tile_to_world`**: `intel_visualization_system` contains its own `tile_to_world` function instead of importing from `utils::pathfinding`. This avoids circular module imports and keeps the system self-contained.
* **Schedule Split**: 
  * `intel_acquisition_system` runs on `Update` in Port state - processes purchase events immediately.
  * `intel_expiry_system` runs on `FixedUpdate` - tied to `WorldClock` ticks for deterministic timing.
  * `intel_visualization_system` runs on `Update` in HighSeas state - visual only, no physics.
* **Intel Type Distribution**: Generated tavern intel uses weighted probabilities: Rumor 40%, MapReveal 20%, ShipRoute 20%, TreasureLocation 10%, FleetPosition 10%. This balances gameplay value with cost.
* **Transient Entity Pattern**: Intel entities are spawned with `IntelExpiry` for automatic cleanup. The default TTL is 1 in-game day (~24 real seconds). This prevents entity accumulation from repeated port visits.

## 11. World Map Tilemap Persistence
* **Tilemap Lifecycle**: The world map and fog tilemaps are spawned once on first `OnEnter(HighSeas)` and persist across state transitions. They are NOT despawned when leaving HighSeas.
  * Rationale: Respawning 512x512 tiles on every state transition causes fog of war state to be lost (all tiles reset to opaque).
* **Visibility Toggle**: Use `hide_tilemap` and `show_tilemap` systems to toggle visibility:
  * `OnEnter(Combat)` → Hide tilemaps
  * `OnEnter(HighSeas)` → Show tilemaps
* **Spawn Guard**: `spawn_tilemap_from_map_data` checks if `WorldMap` entity already exists and early-exits if so. This prevents duplicate tilemaps on re-entry to HighSeas.

## 12. Companion Ability Pattern
* **Global Query Pattern**: Companion abilities query ALL `CompanionRole` components globally, not just those "assigned" to the player. This simplifies implementation since companions are player-owned persistent entities.
* **Bonus Values**:
  * Navigator: +25% sailing speed (`base_speed * 1.25`)
  * Lookout: +50% vision radius (`vision.radius * 1.5`)
  * Gunner: -30% cannon cooldown (`base_cooldown * 0.7`)
* **Schedule Placement**: Abilities integrate into existing systems rather than creating new ones:
  * Navigator → `navigation_movement_system` (Update, HighSeas)
  * Lookout → `fog_of_war_update_system` (Update, HighSeas)
  * Gunner → `cannon_firing_system` (FixedUpdate, Combat)

## 13. MetaProfile Persistence
* **File Location**: Platform-specific via `dirs::data_dir()`:
  * macOS: `~/Library/Application Support/pirates/profile.json`
  * Linux: `~/.local/share/pirates/profile.json`
  * Windows: `%APPDATA%/pirates/profile.json`
* **Load/Save Lifecycle**:
  * Load: `init_meta_profile` in `CorePlugin` Startup schedule
  * Save: `save_profile_on_death` on `OnEnter(GameState::GameOver)`
* **Stat Progression**: Milestone-based (not XP):
  * Charisma: +1 per 2 completed runs (max +4)
  * Navigation: +1 per 5000 lifetime gold (max +4)
  * Logistics: +1 per 3 captured ships (max +4)
  * All stats cap at level 5 (base 1 + 4 from milestones)

## 14. Archetype System
* **Selection Flow**: MainMenu → SelectedArchetype resource → spawn_high_seas_player reads it
* **Bonus Application**: Bonuses apply ONCE when transitioning MainMenu → HighSeas:
  * Starting gold → `Gold` component
  * Ship type → Sprite path + cargo capacity
  * Faction reputation → `FactionRegistry.factions[*].player_reputation`
* **Unlock Conditions**: Checked at startup via `check_archetype_unlocks` system:
  * `AlwaysUnlocked` — Default archetype
  * `RunsCompleted(n)` — Compare against `profile.runs_completed`
  * `LifetimeGold(n)` — Compare against `profile.lifetime_gold`
  * `QuickDeath(hours)` — Special case: tracked via death events, stored in `unlocked_archetypes`
* **Ship Type Stats**:
  | ShipType | Cargo Capacity | Sprite |
  |----------|----------------|--------|
  | Sloop | 100 | player.png |
  | Frigate | 200 | frigate.png |
  | Schooner | 150 | schooner.png |
  | Raft | 30 | raft.png |
* **No Re-Application**: Faction reputation bonuses apply only on first spawn. Re-entering HighSeas (from Port or Combat) does NOT re-apply archetype bonuses because `spawn_high_seas_player` only runs on initial entry.

## 15. Legacy Wreck System
* **Death Data Capture**: State transfer pattern — `PlayerDeathData` resource captures position, gold, and cargo BEFORE despawn in `ship_destruction_system`
* **Wreck Recording**: On `OnEnter(GameOver)`, `save_profile_on_death` creates a `LegacyWreck` from death data and adds it to `MetaProfile.legacy_wrecks`
* **Wreck Cap**: Maximum 10 wrecks stored; oldest removed when cap exceeded
* **Wreck Spawning**: `spawn_legacy_wrecks` runs on `OnEnter(HighSeas)`, converts tile positions to world coordinates
* **Exploration**: `wreck_exploration_system` triggers on proximity (48 world units):
  * Transfers gold to player's `Gold` component
  * Transfers cargo (if space available) to player's `Cargo` component
  * Removes wreck from `MetaProfile` and saves immediately
  * Despawns wreck entity
* **Position Persistence**: Stored as tile coordinates (`IVec2`), converted to/from world coords using `TILE_SIZE = 16.0`
* **Index Limitation**: `LegacyWreckMarker.wreck_index` becomes stale after ANY wreck is removed. Safe only because exploration is single-wreck proximity-based and entities despawn immediately. Do NOT batch wreck removals.

## 16. Save/Load System (bevy_save)
* **Plugin Structure**: `PersistencePlugin` wraps `bevy_save::SavePlugins` to avoid name collision with `bevy_save::SavePlugin`
* **Save Location**: Platform-specific via `bevy_save` defaults:
  * macOS: `~/Library/Application Support/pirates/saves/`
  * Linux: `~/.local/share/pirates/saves/`
  * Windows: `%APPDATA%/pirates/saves/`
* **Reflect Requirement**: All components and resources to be saved MUST derive `Reflect` + `#[reflect(Component)]` or `#[reflect(Resource)]`.
* **Snapshot vs MetaProfile**: Two separate persistence mechanisms:
  * `MetaProfile` — Cross-run progression (stats, unlocks, wrecks). Uses custom JSON via `serde`.
  * `bevy_save` — In-run game state (world entities, components). Uses MessagePack via `rmp_serde`.
* **Pipeline Pattern**: `bevy_save` uses `Pipeline` trait. `&str` implements Pipeline by default (save name → file name).
* **Keyboard Shortcuts**:
  | Key | Action | Available In |
  |-----|--------|--------------|
  | F5 | Quicksave | HighSeas, Port |
  | F9 | Quickload | Any state |
  | F6 | Create "rich" preset (10k gold) | HighSeas |
  | F7 | Create "damaged" preset (25% HP) | HighSeas |
  | F8 | Create "advanced" preset (Day 30) | HighSeas |
* **Autosave Triggers**: `OnEnter(GameState::Port)` and `OnEnter(GameState::HighSeas)` → saves to "autosave"
* **Main Menu Continue**: Checks for `autosave.sav` at startup; shows "Continue" button if found
* **Event-Based Load**: Main menu uses `LoadGameEvent` + exclusive system pattern since `World::load` requires `&mut World`

## 17. Post-Processing Architecture
* **ViewNode Pattern**: Full-screen post-processing effects use the `ViewNode` pattern in the Render Graph.
* **ExtractComponent**: Settings are attached to the Camera as components (e.g., `InkParchmentSettings`) and extracted to the Render World via `ExtractComponentPlugin`.
* **Render Pipeline**: Custom `RenderPipeline` handles the shader application.
* **Core2d Graph**: The post-processing node is inserted into the `Core2d` graph between `Tonemapping` and `EndMainPassPostProcessing`.
