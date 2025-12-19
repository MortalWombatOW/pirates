# Pirates - Work Plan

> **Task Decomposition for AI Agent Implementation**
> Each task is atomic, actionable, and references the relevant README section.
> Tasks are ordered by dependencies within each phase.

---

## Legend

- `[ ]` = Not started
- `[/]` = In progress
- `[x]` = Complete
- `[B]` = Blocked (dependency not met)

**Priority**: `P0` = Critical path, `P1` = Important, `P2` = Nice to have

---

## Phase 1: Project Foundations

> **Goal**: Set up the Bevy project, implement state management, and establish core infrastructure.
> **Reference**: README §6 (Technical Architecture), §7 (Plugin Structure), §8 (Dependencies)

### Epic 1.1: Project Setup

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| [x] 1.1.1 | Create Rust project with `cargo new pirates --bin` | P0 | None | `Cargo.toml` exists, `cargo build` succeeds. |
| [x] 1.1.2 | Add Bevy and all dependencies to `Cargo.toml` | P0 | 1.1.1 | All crates from README §8.2 are listed, `cargo build` succeeds. |
| [x] 1.1.3 | Create directory structure per README §9 | P0 | 1.1.1 | All directories (`src/plugins/`, `src/components/`, `assets/`, etc.) exist. |
| [x] 1.1.4 | Create `main.rs` with minimal Bevy app | P0 | 1.1.2 | App runs, displays empty window with title "Pirates". |
| [x] 1.1.5 | Create `lib.rs` with module re-exports | P1 | 1.1.3 | Compiles without errors, exposes `plugins`, `components`, `systems`, `resources`, `events`. |

### Epic 1.2: State Management

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| [x] 1.2.1 | Define `GameState` enum in `src/plugins/core.rs` | P0 | 1.1.4 | Enum with variants: `MainMenu`, `Port`, `HighSeas`, `Combat`, `GameOver`. |
| [x] 1.2.2 | Create `CorePlugin` and register `GameState` | P0 | 1.2.1 | Plugin added to app, state defaults to `MainMenu`. |
| [x] 1.2.3 | Implement state transition system (placeholder) | P0 | 1.2.2 | Pressing `1-5` keys switches between states (debug feature). |
| [x] 1.2.4 | Add logging for state transitions | P1 | 1.2.3 | Console logs state changes. |

### Epic 1.3: Camera & Input

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| [x] 1.3.1 | Add 2D camera (`Camera2dBundle`) | P0 | 1.1.4 | Camera renders to window. |
| [x] 1.3.2 | Implement camera pan (drag or arrow keys) | P1 | 1.3.1 | Camera position moves with input. |
| [x] 1.3.3 | Implement camera zoom (scroll wheel) | P1 | 1.3.1 | Camera scale changes with scroll. |
| [x] 1.3.4 | Integrate `leafwing-input-manager` | P0 | 1.1.2 | Plugin added to app, no errors. |
| [x] 1.3.5 | Define `PlayerAction` enum (Thrust, Turn, Fire, Anchor, CycleTarget) | P0 | 1.3.4 | Enum implements `Actionlike`. |
| [x] 1.3.6 | Create default `InputMap<PlayerAction>` for WASD + mouse | P0 | 1.3.5 | Input bindings configured. |

### Epic 1.4: Debug UI

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| [x] 1.4.1 | Integrate `bevy_egui` | P0 | 1.1.2 | `EguiPlugin` added to app, no errors. |
| [x] 1.4.2 | Create debug panel showing current `GameState` | P1 | 1.4.1, 1.2.2 | Panel displays current state name. |
| [x] 1.4.3 | Add FPS counter to debug panel | P2 | 1.4.2 | FPS displayed in corner. |
| [x] 1.4.4 | Add state transition buttons to debug panel | P1 | 1.4.2, 1.2.3 | Buttons change state on click. |

---

## Phase 2: Combat MVP

> **Goal**: Implement the core combat loop with physics, weapons, and damage.
> **Reference**: README §3.3.C (Combat View), §4.4 (Combat Systems), §5 (Components)

### Epic 2.1: Physics Setup

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| [x] 2.1.1 | Integrate `avian2d` physics | P0 | 1.1.2 | `PhysicsPlugins::default()` added, no errors. |
| [x] 2.1.2 | Configure physics timestep for combat | P0 | 2.1.1 | Physics runs on `FixedUpdate` at 60Hz. |
| [x] 2.1.3 | Create test `RigidBody` entity that falls/moves | P0 | 2.1.2 | Entity responds to gravity/forces. |

### Epic 2.2: Ship Entity

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| [x] 2.2.1 | Define `Ship` marker component | P0 | 1.1.3 | Component exists in `src/components/ship.rs`. |
| [x] 2.2.2 | Define `Health` component (sails, rudder, hull) | P0 | 1.1.3 | Component with `sails`, `rudder`, `hull` fields and maxes. |
| [x] 2.2.3 | Define `Cargo` component | P1 | 1.1.3 | Component with `goods: HashMap<GoodType, u32>` and `capacity`. |
| [x] 2.2.4 | Define `Gold` component | P1 | 1.1.3 | Newtype component `Gold(u32)`. |
| [x] 2.2.5 | Create `spawn_player_ship` function | P0 | 2.2.1, 2.2.2, 2.1.1 | Spawns entity with `Ship`, `Player`, `Health`, `RigidBody`, `Sprite`. |
| [x] 2.2.6 | Create placeholder ship sprite | P0 | None | 64x64 PNG in `assets/sprites/ships/player.png`. |
| [x] 2.2.7 | Load and display ship sprite | P0 | 2.2.5, 2.2.6 | Player ship visible on screen. |

### Epic 2.3: Ship Movement

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| [x] 2.3.1 | Create `ShipMovementSystem` | P0 | 2.2.5, 1.3.5 | System queries `Ship` + `RigidBody`. |
| [x] 2.3.2 | Implement thrust (W key) | P0 | 2.3.1 | Ship accelerates forward when W pressed. |
| [x] 2.3.3 | Implement reverse (S key) | P1 | 2.3.1 | Ship accelerates backward when S pressed. |
| [x] 2.3.4 | Implement turn (A/D keys) | P0 | 2.3.1 | Ship rotates left/right. |
| [x] 2.3.5 | Apply drag to simulate water resistance | P0 | 2.3.1 | Ship slows down when no input. |
| [x] 2.3.6 | Implement anchor drop (Shift key) | P1 | 2.3.1 | Ship velocity set to zero, rotation allowed. |
| [x] 2.3.7 | Apply speed debuff based on sail damage | P0 | 2.3.1, 2.2.2 | `MaxSpeed` reduced proportionally to sail damage. |
| [x] 2.3.8 | Apply turn debuff based on rudder damage | P0 | 2.3.1, 2.2.2 | `TurnRate` reduced proportionally to rudder damage. |

### Epic 2.4: Cannons & Projectiles

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| [x] 2.4.1 | Define `Projectile` component | P0 | 1.1.3 | Component with `damage`, `target`, `source`. |
| [x] 2.4.2 | Define `TargetComponent` enum | P0 | 1.1.3 | Enum (Sails, Rudder, Hull) in `src/components/combat.rs`. |
| [x] 2.4.3 | Create `CannonState` resource | P0 | 1.1.3 | Resource tracks cooldown. |
| [x] 2.4.4 | Create `CannonFiringSystem` (Q/E Broadside) | P0 | 2.4.1, 2.4.3 | Q/E spawns spread of 3 projectiles from respective side. |
| [x] 2.4.5 | Create projectile sprite | P0 | None | 16x16 PNG loaded. |
| [x] 2.4.6 | Spawn projectile with velocity | P0 | 2.4.4, 2.4.5 | Projectiles spawn with ship velocity + ejection speed. |
| [x] 2.4.7 | Create `ProjectileSystem` (timers) | P0 | 2.4.6 | Projectiles despawn after 5s. |
| [x] 2.4.8 | Handle hit detection (Ships) | P0 | 2.4.7 | Collisions reduce `Health`. Self-hits prevented. |
| [x] 2.4.9 | Implement "Sticky" Input Buffering | P1 | 2.4.4 | Ensure firing intent is conserved during cooldown. |
| [x] 2.4.10| Add Visual Reference Grid | P2 | 2.4.4 | Draw background grid for movement cues. |

### Epic 2.5: Damage & Hit Detection

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| [x] 2.5.1 | Define `ShipHitEvent` (or specialized system) | P0 | 1.1.3 | Handled directly in `projectile_collision_system`. |
| [x] 2.5.2 | Implement collision detection (projectile vs ship) | P0 | 2.4.7, 2.2.5 | `avian2d` collision events processed. |
| [x] 2.5.3 | Emit `ShipHitEvent` on collision | P0 | 2.5.1, 2.5.2 | (Skipped: Direct component modification for MVP). |
| [x] 2.5.4 | Create `DamageSystem` | P0 | 2.5.3, 2.2.2 | `projectile_collision_system` updates `Health`. |
| 2.5.5 | Implement `WaterIntake` component | P1 | 1.1.3 | Component with `rate` and `current` water level. |
| 2.5.6 | Add `WaterIntake` on hull damage | P1 | 2.5.4, 2.5.5 | Hull damage adds/increases `WaterIntake`. |
| [x] 2.5.7 | Implement ship destruction (hull HP <= 0) | P0 | 2.5.4 | Ship entity despawned, `ShipDestroyedEvent` emitted. |

### Epic 2.6: Loot System

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| [x] 2.6.1 | Define `Loot` component | P0 | 1.1.3 | Component with `value`, `good_type`. |
| [x] 2.6.2 | Create loot sprite | P0 | None | 32x32 PNG in `assets/sprites/loot/gold.png`. |
| [x] 2.6.3 | Spawn loot on ship hit | P0 | 2.5.3, 2.6.1, 2.6.2 | Loot entity spawned at hit location. |
| [x] 2.6.4 | Make loot a `RigidBody` | P0 | 2.6.3 | Loot affected by physics. |
| [x] 2.6.5 | Implement loot collection (player collision) | P0 | 2.6.4, 2.2.5 | Loot despawned, added to player `Gold`/`Cargo`. |

### Epic 2.7: Current Zones

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 2.7.1 | Define `CurrentZone` component | P0 | 1.1.3 | Component with `velocity: Vec2`, `bounds: Rect`. |
| 2.7.2 | Create `CurrentSystem` | P0 | 2.7.1 | Queries all `RigidBody` in zone, applies `ExternalForce`. |
| 2.7.3 | Spawn test current zone | P0 | 2.7.1, 2.7.2 | Visible zone that pushes entities. |
| 2.7.4 | Visualize current zones (subtle overlay) | P2 | 2.7.3 | Directional arrows or flow lines. |

### Epic 2.8: Enemy Ships & AI

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 2.8.1 | Define `AI` marker component | P0 | 1.1.3 | Component exists. |
| 2.8.2 | Define `Faction` component | P0 | 1.1.3 | Component with `FactionId`. |
| 2.8.3 | Create `spawn_enemy_ship` function | P0 | 2.8.1, 2.8.2, 2.2.2 | Spawns enemy with `AI`, `Faction`, `Ship`, `Health`. |
| 2.8.4 | Create enemy ship sprite (different color) | P0 | None | 64x64 PNG in `assets/sprites/ships/enemy.png`. |
| 2.8.5 | Create `CombatAISystem` | P0 | 2.8.3, 2.3.1 | Enemy chases player. |
| 2.8.6 | Implement AI pursuit behavior | P0 | 2.8.5 | Enemy rotates toward and moves toward player. |
| 2.8.7 | Implement AI firing logic | P0 | 2.8.5, 2.4.4 | Enemy fires when in range and facing player. |
| 2.8.8 | Implement AI flee behavior (low HP) | P1 | 2.8.5 | Enemy turns and flees when HP < 20%. |

### Epic 2.9: Combat Flow

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 2.9.1 | Define `CombatEndedEvent` | P0 | 1.1.3 | Event with `victory: bool`. |
| 2.9.2 | Detect all enemies destroyed | P0 | 2.5.7 | System checks if no `AI` + `Ship` entities remain. |
| 2.9.3 | Emit `CombatEndedEvent` on victory | P0 | 2.9.1, 2.9.2 | Event sent when all enemies dead. |
| 2.9.4 | Detect player destroyed | P0 | 2.5.7 | System checks if `Player` + `Ship` is gone. |
| 2.9.5 | Emit `PlayerDiedEvent` | P0 | 2.9.4 | Event sent when player dies. |
| 2.9.6 | Transition to `GameOverState` on player death | P0 | 2.9.5, 1.2.2 | State changes to `GameOver`. |
| 2.9.7 | Transition to `HighSeasState` on combat victory | P0 | 2.9.3, 1.2.2 | State changes to `HighSeas`. |

---

## Phase 3: High Seas Map

> **Goal**: Implement the world map with fog of war, wind, and navigation.
> **Reference**: README §3.3.B (High Seas View), §4.2 (Navigation System), §4.3 (Encounter System)

### Epic 3.1: Tilemap Setup

![Map Tile](assets/docs/map_tile.png)
*Typical Map Tile*

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 3.1.1 | Integrate `bevy_ecs_tilemap` | P0 | 1.1.2 | Plugin added, no errors. |
| 3.1.2 | Create tileset image (water, land variants) | P0 | None | PNG in `assets/tilemaps/tileset.png`. |
| 3.1.3 | Create `TilemapPlugin` | P0 | 3.1.1 | Plugin manages map loading/rendering. |
| 3.1.4 | Define `MapData` resource (tile grid) | P0 | 3.1.3 | Resource holds 2D array of tile types. |
| 3.1.5 | Spawn tilemap from `MapData` | P0 | 3.1.4, 3.1.2 | Tilemap renders water and land tiles. |

### Epic 3.2: Procedural Generation

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 3.2.1 | Integrate `noise` crate | P0 | 1.1.2 | Crate available. |
| 3.2.2 | Create `generate_world_map` function | P0 | 3.2.1, 3.1.4 | Returns `MapData` with procedural land/water. |
| 3.2.3 | Use Perlin/Simplex noise for landmasses | P0 | 3.2.2 | Noise-based threshold determines land vs water. |
| 3.2.4 | Ensure starting area is navigable | P0 | 3.2.2 | Player spawn point is always on water. |
| 3.2.5 | Place port locations procedurally | P0 | 3.2.2 | Ports spawn on coastlines. |

### Epic 3.3: Fog of War

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 3.3.1 | Define `FogOfWar` resource (explored tile set) | P0 | 3.1.4 | Resource holds `HashSet<IVec2>` of explored tiles. |
| 3.3.2 | Create `FogOfWarSystem` | P0 | 3.3.1 | Updates explored tiles based on player position. |
| 3.3.3 | Render fog overlay on unexplored tiles | P0 | 3.3.2 | Dark overlay on tiles not in `FogOfWar`. |
| 3.3.4 | Define player vision radius | P0 | 3.3.2 | Tiles within radius of player are revealed. |
| 3.3.5 | Lookout companion increases vision radius | P1 | 3.3.4 | If Lookout present, radius increased. |

### Epic 3.4: Wind System

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 3.4.1 | Define `Wind` resource | P0 | 1.1.3 | Resource with `direction: Vec2`, `strength: f32`. |
| 3.4.2 | Create `WindSystem` | P0 | 3.4.1 | Updates wind periodically (slowly shifts). |
| 3.4.3 | Display wind direction on HUD (compass rose) | P0 | 3.4.1, 1.4.1 | UI shows arrow indicating wind direction. |
| 3.4.4 | Apply wind to navigation speed | P0 | 3.4.1 | Traveling with wind = faster, against = slower. |
| 3.4.5 | Apply wind to combat movement | P0 | 3.4.1, 2.3.1 | Ships move faster downwind, slower upwind in combat. |

### Epic 3.5: Navigation

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 3.5.1 | Define `Destination` component | P0 | 1.1.3 | Component with `target: Vec2`. |
| 3.5.2 | Create `NavigationSystem` | P0 | 3.5.1, 3.4.4 | Moves player toward destination each tick. |
| 3.5.3 | Implement click-to-navigate | P0 | 3.5.1, 1.3.5 | Clicking on map sets `Destination`. |
| 3.5.4 | Implement A* pathfinding around land | P1 | 3.5.2, 3.1.4 | Path avoids land tiles. |
| 3.5.5 | Visualize planned path | P2 | 3.5.4 | Dotted line shows route. |
| 3.5.6 | Navigator companion auto-routes | P1 | 3.5.4 | If Navigator present, path is optimized. |
| 3.5.7 | Detect arrival at port | P0 | 3.5.2 | When player reaches port tile, trigger `PortState`. |

### Epic 3.6: Encounters

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 3.6.1 | Create `SpatialHash` utility | P0 | 1.1.3 | Utility for efficient proximity queries. |
| 3.6.2 | Spawn AI ships on map (world simulation placeholder) | P0 | 2.8.3 | AI ships exist on world map. |
| 3.6.3 | Create `EncounterSystem` | P0 | 3.6.1, 3.6.2 | Checks if player near AI ship. |
| 3.6.4 | Determine hostility based on faction reputation | P0 | 3.6.3, 2.8.2 | Pirates always hostile, others based on rep. |
| 3.6.5 | Emit `CombatTriggeredEvent` | P0 | 3.6.4 | Event sent when hostile encounter. |
| 3.6.6 | Transition to `CombatState` on encounter | P0 | 3.6.5, 1.2.2 | State changes to `Combat`. |
| 3.6.7 | Transfer relevant entities to combat scene | P0 | 3.6.6 | Player ship and encountered enemies spawn in combat. |

---

## Phase 4: Ports & Economy

> **Goal**: Implement the port view with trading, contracts, and services.
> **Reference**: README §3.3.A (Port View), §4.5 (Economy System), §4.6 (Contract System)

### Epic 4.1: Port Entity

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 4.1.1 | Define `Port` marker component | P0 | 1.1.3 | Component exists. |
| 4.1.2 | Define `Inventory` component | P0 | 1.1.3 | Component with goods, quantities, prices. |
| 4.1.3 | Create `spawn_port` function | P0 | 4.1.1, 4.1.2 | Spawns port entity with inventory. |
| 4.1.4 | Generate initial inventory for ports | P0 | 4.1.3, 3.2.5 | Each port has random starting goods. |

### Epic 4.2: Port UI

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 4.2.1 | Create Port View layout | P0 | 1.4.1, 1.2.2 | UI displays when in `PortState`. |
| 4.2.2 | Implement Market panel | P0 | 4.2.1 | Shows goods, prices, buy/sell buttons. |
| 4.2.3 | Implement Tavern panel | P1 | 4.2.1 | Shows intel options, rumors. |
| 4.2.4 | Implement Docks panel | P1 | 4.2.1 | Shows ship HP, repair options. |
| 4.2.5 | Implement Contracts panel | P1 | 4.2.1 | Shows available contracts. |
| 4.2.6 | Implement Depart button | P0 | 4.2.1 | Transitions to `HighSeasState`. |

### Epic 4.3: Trading

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 4.3.1 | Create `MarketSystem` | P0 | 4.2.2, 2.2.3, 2.2.4 | Handles buy/sell logic. |
| 4.3.2 | Implement buy goods | P0 | 4.3.1 | Player gold decreases, cargo increases. |
| 4.3.3 | Implement sell goods | P0 | 4.3.1 | Player cargo decreases, gold increases. |
| 4.3.4 | Implement cargo capacity check | P0 | 4.3.2 | Cannot buy if cargo full. |
| 4.3.5 | Emit `TradeExecutedEvent` | P1 | 4.3.1 | Event for audio/logging. |

### Epic 4.4: Price Dynamics

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 4.4.1 | Create `PriceCalculationSystem` | P0 | 4.1.2 | Runs on world tick. |
| 4.4.2 | Adjust prices based on supply | P0 | 4.4.1 | Low stock = higher price. |
| 4.4.3 | Adjust prices based on demand (global) | P1 | 4.4.1 | High demand goods = higher price everywhere. |
| 4.4.4 | Implement goods decay (perishables) | P1 | 4.4.1, 2.2.3 | Perishable goods lose value over time. |

### Epic 4.5: Contracts

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 4.5.1 | Define `Contract` entity and components | P0 | 1.1.3 | `Contract`, `ContractType`, `Origin`, `Destination`, `Reward`, `Expiry`. |
| 4.5.2 | Create `ContractGenerationSystem` | P0 | 4.5.1 | Procedurally creates contracts per port. |
| 4.5.3 | Implement contract acceptance | P0 | 4.5.1, 4.2.5 | Player accepts, contract added to active list. |
| 4.5.4 | Implement contract tracking | P0 | 4.5.3 | Track progress (cargo delivered, area explored). |
| 4.5.5 | Implement contract completion | P0 | 4.5.4 | On conditions met, reward paid, `ContractCompletedEvent`. |
| 4.5.6 | Implement contract expiry | P1 | 4.5.1 | Expired contracts removed, reputation penalty. |

### Epic 4.6: Ship Repair

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 4.6.1 | Create `RepairSystem` | P0 | 4.2.4, 2.2.2, 2.2.4 | Repairs cost gold. |
| 4.6.2 | Implement repair sails | P0 | 4.6.1 | Restore `sails` HP. |
| 4.6.3 | Implement repair rudder | P0 | 4.6.1 | Restore `rudder` HP. |
| 4.6.4 | Implement repair hull | P0 | 4.6.1 | Restore `hull` HP, remove `WaterIntake`. |
| 4.6.5 | Display repair costs | P0 | 4.6.1, 4.2.4 | UI shows cost per component. |

---

## Phase 5: World Simulation & Orders

> **Goal**: Implement the background world simulation with 1000 ships and the orders system.
> **Reference**: README §4.1 (World Simulation), §4.7 (Orders System)

### Epic 5.1: World Tick

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 5.1.1 | Define `WorldClock` resource | P0 | 1.1.3 | Resource with `day`, `hour`, `tick`. |
| 5.1.2 | Create `WorldTickSystem` | P0 | 5.1.1 | Runs on `FixedUpdate`, increments clock. |
| 5.1.3 | Display time on HUD | P1 | 5.1.1, 1.4.1 | Shows "Day X, Hour Y". |

### Epic 5.2: Faction AI

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 5.2.1 | Define `FactionRegistry` resource | P0 | 1.1.3 | Holds state for each faction. |
| 5.2.2 | Define `FactionState` struct | P0 | 5.2.1 | Gold, ships, reputation, trade routes. |
| 5.2.3 | Create `FactionAISystem` | P0 | 5.2.1, 5.1.2 | Runs per world tick. |
| 5.2.4 | Implement trade route generation | P0 | 5.2.3 | Faction AI creates routes between ports. |
| 5.2.5 | Implement ship spawning by faction | P0 | 5.2.3, 2.8.3 | Faction spawns ships to fulfill routes. |
| 5.2.6 | Implement threat response | P1 | 5.2.3 | Faction sends ships to combat player if hostile. |

### Epic 5.3: AI Ship Behavior

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 5.3.1 | Define `Order` enum | P0 | 1.1.3 | `TradeRoute`, `Patrol`, `Escort`, `Scout`. |
| 5.3.2 | Define `OrderQueue` component | P0 | 5.3.1 | Queue of `Order`. |
| 5.3.3 | Create `OrderExecutionSystem` | P0 | 5.3.2 | Reads orders, drives ship navigation. |
| 5.3.4 | Implement `TradeRoute` order | P0 | 5.3.3 | Ship navigates from A to B, trades, repeats. |
| 5.3.5 | Implement `Patrol` order | P1 | 5.3.3 | Ship moves around area, engages hostiles. |
| 5.3.6 | Implement `Escort` order | P1 | 5.3.3 | Ship follows target entity. |
| 5.3.7 | Implement `Scout` order | P1 | 5.3.3 | Ship explores area, reports intel. |

### Epic 5.4: Player Fleet Management

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 5.4.1 | Allow player to own multiple ships | P0 | 2.2.5 | Player has list of owned ships. |
| 5.4.2 | Create Fleet Management UI | P0 | 5.4.1, 1.4.1 | Shows owned ships and their orders. |
| 5.4.3 | Allow issuing orders to owned ships | P0 | 5.4.2, 5.3.2 | UI to assign `TradeRoute`, `Patrol`, etc. |
| 5.4.4 | Implement subcontracting (delegate contract) | P1 | 5.4.3, 4.5.3 | Assign contract to owned ship for cut. |

### Epic 5.5: Scale Testing

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 5.5.1 | Spawn 1000 AI ships | P0 | 5.2.5 | 1000 ships exist in simulation. |
| 5.5.2 | Profile frame rate | P0 | 5.5.1 | Game runs > 30 FPS with 1000 ships. |
| 5.5.3 | Optimize with spatial hashing | P0 | 5.5.2, 3.6.1 | Reduce O(n²) checks. |
| 5.5.4 | Optimize with LOD (hide distant ships) | P1 | 5.5.2 | Only render ships near camera. |

---

## Phase 6: Intelligence & Companions

> **Goal**: Implement the intel system and companion abilities.
> **Reference**: README §4.8 (Intelligence System), §4.9 (Companion System)

### Epic 6.1: Intel System

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 6.1.1 | Define `Intel` entity and components | P0 | 1.1.3 | `Intel`, `IntelType`, `MapData`, `Expiry`. |
| 6.1.2 | Create `IntelAcquiredEvent` | P0 | 6.1.1 | Event with intel data. |
| 6.1.3 | Create `IntelSystem` | P0 | 6.1.2, 3.3.1 | Adds revealed data to map on acquisition. |
| 6.1.4 | Implement intel expiry | P0 | 6.1.1, 5.1.2 | Transient intel removed after TTL. |
| 6.1.5 | Implement tavern intel purchase | P0 | 6.1.2, 4.2.3 | Player buys intel at tavern. |
| 6.1.6 | Visualize intel on map (icons, routes) | P0 | 6.1.3 | Ship routes shown as lines, etc. |

### Epic 6.2: Companions

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 6.2.1 | Define `Companion` entity and components | P0 | 1.1.3 | `Companion`, `Role`, `Skills`, `AssignedTo`. |
| 6.2.2 | Define `CompanionRole` enum | P0 | 6.2.1 | `Quartermaster`, `Navigator`, `Lookout`, `Gunner`, `Mystic`. |
| 6.2.3 | Create `spawn_companion` function | P0 | 6.2.1, 6.2.2 | Creates companion entity. |
| 6.2.4 | Implement companion recruitment (tavern) | P0 | 6.2.3, 4.2.3 | Player recruits at tavern for gold. |
| 6.2.5 | Create companion roster UI | P0 | 6.2.4, 1.4.1 | Shows recruited companions. |
| 6.2.6 | Implement Quartermaster ability | P0 | 6.2.2, 4.3.1 | Auto-trades based on market intel. |
| 6.2.7 | Implement Navigator ability | P0 | 6.2.2, 3.5.6 | Auto-routes efficient paths. |
| 6.2.8 | Implement Lookout ability | P0 | 6.2.2, 3.3.5 | Increases vision radius. |
| 6.2.9 | Implement Gunner ability | P1 | 6.2.2, 2.4.3 | Reduces cannon cooldown. |

---

## Phase 7: Progression & Persistence

> **Goal**: Implement roguelike meta-progression and save/load.
> **Reference**: README §4.10 (Progression System), §4.11 (Save/Load)

### Epic 7.1: Meta Profile

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 7.1.1 | Define `MetaProfile` resource | P0 | 1.1.3 | Stats, unlocks, legacy wrecks. |
| 7.1.2 | Load `MetaProfile` on app start | P0 | 7.1.1 | Loaded from file or created fresh. |
| 7.1.3 | Save `MetaProfile` on death/quit | P0 | 7.1.1 | Written to file. |
| 7.1.4 | Define player stats (Charisma, Navigation, Logistics) | P0 | 7.1.1 | Stats affect game systems. |
| 7.1.5 | Implement stat progression (XP or milestones) | P0 | 7.1.4 | Stats increase over runs. |

### Epic 7.2: Archetypes

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 7.2.1 | Define starting archetypes | P0 | 7.1.1 | `Default`, `RoyalNavyCaptain`, `Smuggler`, `Castaway`. |
| 7.2.2 | Implement archetype unlock conditions | P0 | 7.2.1 | Unlocked based on achievements. |
| 7.2.3 | Implement archetype selection on new game | P0 | 7.2.1, 1.2.2 | UI to choose archetype. |
| 7.2.4 | Apply archetype bonuses | P0 | 7.2.3 | Starting gold, faction rep, ship type. |

### Epic 7.3: Legacy Wrecks

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 7.3.1 | Record wreck on player death | P0 | 2.9.5, 7.1.1 | Position and cargo saved to `MetaProfile`. |
| 7.3.2 | Spawn legacy wrecks on new run | P0 | 7.3.1, 3.2.2 | Wrecks placed on map. |
| 7.3.3 | Implement wreck exploration | P0 | 7.3.2 | Player can loot wreck for cargo/gold. |

### Epic 7.4: Save/Load

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 7.4.1 | Integrate `bevy_save` | P0 | 1.1.2 | Plugin added. |
| 7.4.2 | Implement save game | P0 | 7.4.1 | All relevant entities/resources serialized. |
| 7.4.3 | Implement load game | P0 | 7.4.1 | World reconstructed from save file. |
| 7.4.4 | Implement autosave | P0 | 7.4.2 | Auto-triggers on state transitions. |
| 7.4.5 | Add Save/Load to main menu | P0 | 7.4.2, 7.4.3, 4.2.1 | UI buttons work. |

---

## Phase 8: Audio & Polish

> **Goal**: Add audio, shaders, and visual polish.
> **Reference**: README §4.12 (Audio), §4.13 (Rendering), §1.4 (Aesthetic)

### Epic 8.1: Audio Integration

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 8.1.1 | Integrate `bevy_kira_audio` | P0 | 1.1.2 | Plugin added, no errors. |
| 8.1.2 | Create `AudioPlugin` | P0 | 8.1.1 | Manages music and SFX. |
| 8.1.3 | Implement scene-based music | P0 | 8.1.2, 1.2.2 | Different tracks for Port, High Seas, Combat. |
| 8.1.4 | Add placeholder music tracks | P0 | 8.1.3 | MP3/OGG files in `assets/audio/music/`. |
| 8.1.5 | Implement ambient sounds | P1 | 8.1.2 | Layered loops (waves, wind). |
| 8.1.6 | Implement SFX triggers | P0 | 8.1.2 | Cannon fire, hit, purchase, UI click. |
| 8.1.7 | Add placeholder SFX files | P0 | 8.1.6 | Files in `assets/audio/sfx/`. |

### Epic 8.2: Visual Polish

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 8.2.1 | Create "Ink and Parchment" shader | P1 | 1.1.2 | WGSL shader in `assets/shaders/`. |
| 8.2.2 | Apply shader as post-processing | P1 | 8.2.1 | Entire game has parchment tint. |
| 8.2.3 | Create parchment texture for UI | P0 | None | PNG in `assets/sprites/ui/`. |
| 8.2.4 | Style UI panels with parchment texture | P0 | 8.2.3, 1.4.1 | Egui panels have parchment bg. |
| 8.2.5 | Add scroll/dagger decorations to UI | P1 | 8.2.4 | Decorative elements on panels. |
| 8.2.6 | Add screen shake on cannon fire | P1 | 2.4.4 | Camera shakes briefly. |
| 8.2.7 | Add hit flash on ship damage | P1 | 2.5.4 | Ship sprite flashes white. |

---

## Phase 9: Steam Integration

> **Goal**: Integrate Steamworks for achievements and cloud saves.
> **Reference**: README §8.1 (Dependencies)

### Epic 9.1: Steamworks

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 9.1.1 | Integrate `steamworks` crate | P0 | 1.1.2 | Crate added, compiles. |
| 9.1.2 | Initialize Steam on app launch | P0 | 9.1.1 | Steam overlay available. |
| 9.1.3 | Define achievements | P0 | 9.1.2 | List in Steam partner site. |
| 9.1.4 | Implement achievement unlocking | P0 | 9.1.3 | Trigger on events (first win, etc.). |
| 9.1.5 | Implement cloud saves | P1 | 9.1.2, 7.4.2 | Saves sync via Steam Cloud. |
| 9.1.6 | Build and test Steam release | P0 | All above | Runs as Steam game. |

---

## Phase 10: Supernatural Shift

> **Goal**: Implement late-game supernatural content.
> **Reference**: README §4.9 (Companion System - Mystic), §7.2 (Supernatural Shift)

### Epic 10.1: Narrative Trigger

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 10.1.1 | Define success threshold | P0 | 7.1.1 | Gold earned, ships sunk, etc. |
| 10.1.2 | Create `SupernaturalShiftEvent` | P0 | 10.1.1 | Emitted when threshold reached. |
| 10.1.3 | Display narrative reveal | P0 | 10.1.2 | Cutscene or dialog. |

### Epic 10.2: Supernatural Enemies

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 10.2.1 | Define `Supernatural` faction | P0 | 2.8.2 | New `FactionId`. |
| 10.2.2 | Create undead ship sprites | P0 | 2.8.4 | Ghostly/skeletal ship variants. |
| 10.2.3 | Spawn supernatural ships post-shift | P0 | 10.1.2, 10.2.1, 10.2.2 | Undead ships appear on map. |
| 10.2.4 | Implement boss ships | P0 | 10.2.3 | Unique AI, high HP, special attacks. |
| 10.2.5 | Boss ships cannot be captured | P0 | 10.2.4 | Immune to boarding/capture. |

### Epic 10.3: Magic Abilities

| ID | Task | Priority | Dependencies | Acceptance Criteria |
|---|---|---|---|---|
| 10.3.1 | Define `MagicAbility` enum | P0 | 1.1.3 | `BurnSails`, `FreezeRudder`, `Invisibility`, `WindManipulation`. |
| 10.3.2 | Create `MagicSystem` | P0 | 10.3.1 | Handles ability activation and cooldowns. |
| 10.3.3 | Implement `BurnSails` | P0 | 10.3.2 | DoT on enemy sails. |
| 10.3.4 | Implement `FreezeRudder` | P0 | 10.3.2 | Lock enemy turn rate. |
| 10.3.5 | Implement `Invisibility` | P0 | 10.3.2 | Player ship hidden from AI. |
| 10.3.6 | Implement `WindManipulation` | P0 | 10.3.2, 3.4.1 | Change local wind direction. |
| 10.3.7 | Magic granted via Mystic companion | P0 | 10.3.2, 6.2.2 | Mystic enables magic abilities. |
| 10.3.8 | Magic granted via artifacts | P1 | 10.3.2 | Find artifacts in supernatural wrecks. |

---

## Summary

| Phase | Epics | Tasks | Est. Time |
|---|---|---|---|
| 1. Foundations | 4 | 18 | 2 weeks |
| 2. Combat MVP | 9 | 48 | 2 weeks |
| 3. High Seas Map | 6 | 30 | 2 weeks |
| 4. Ports & Economy | 6 | 27 | 2 weeks |
| 5. World Simulation | 5 | 22 | 2 weeks |
| 6. Intel & Companions | 2 | 15 | 1 week |
| 7. Progression | 4 | 16 | 1 week |
| 8. Audio & Polish | 2 | 14 | 1 week |
| 9. Steam | 1 | 6 | 1 week |
| 10. Supernatural | 3 | 14 | 2 weeks |
| **Total** | **42** | **210** | **16 weeks** |

---

**End of Work Plan**
