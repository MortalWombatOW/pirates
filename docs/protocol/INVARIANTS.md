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