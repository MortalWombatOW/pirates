# Pirates - Work Plan

## Legend
- `[ ]` = Not started
- `[/]` = In progress
- `[x]` = Complete
- `[B]` = Blocked (dependency not met)

---

## Phase 1: Project Foundations
### Epic 1.1: Project Setup
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 1.1.1 | Create Rust project with `cargo new pirates --bin` | None | `Cargo.toml` exists, `cargo build` succeeds. |
| [x] 1.1.2 | Add Bevy and all dependencies to `Cargo.toml` | 1.1.1 | All crates from README §8.2 are listed, `cargo build` succeeds. |
| [x] 1.1.3 | Create directory structure per README §9 | 1.1.1 | All directories (`src/plugins/`, `src/components/`, `assets/`, etc.) exist. |
| [x] 1.1.4 | Create `main.rs` with minimal Bevy app | 1.1.2 | App runs, displays empty window with title "Pirates". |
| [x] 1.1.5 | Create `lib.rs` with module re-exports | 1.1.3 | Compiles without errors, exposes `plugins`, `components`, `systems`, `resources`, `events`. |

### Epic 1.2: State Management
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 1.2.1 | Define `GameState` enum in `src/plugins/core.rs` | 1.1.4 | Enum with variants: `MainMenu`, `Port`, `HighSeas`, `Combat`, `GameOver`. |
| [x] 1.2.2 | Create `CorePlugin` and register `GameState` | 1.2.1 | Plugin added to app, state defaults to `MainMenu`. |
| [x] 1.2.3 | Implement state transition system (placeholder) | 1.2.2 | Pressing `1-5` keys switches between states (debug feature). |
| [x] 1.2.4 | Add logging for state transitions | 1.2.3 | Console logs state changes. |

### Epic 1.3: Camera & Input
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 1.3.1 | Add 2D camera (`Camera2dBundle`) | 1.1.4 | Camera renders to window. |
| [x] 1.3.2 | Implement camera pan (drag or arrow keys) | 1.3.1 | Camera position moves with input. |
| [x] 1.3.3 | Implement camera zoom (scroll wheel) | 1.3.1 | Camera scale changes with scroll. |
| [x] 1.3.4 | Integrate `leafwing-input-manager` | 1.1.2 | Plugin added to app, no errors. |
| [x] 1.3.5 | Define `PlayerAction` enum (Thrust, Turn, Fire, Anchor, CycleTarget) | 1.3.4 | Enum implements `Actionlike`. |
| [x] 1.3.6 | Create default `InputMap<PlayerAction>` for WASD + mouse | 1.3.5 | Input bindings configured. |

### Epic 1.4: Debug UI
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 1.4.1 | Integrate `bevy_egui` | 1.1.2 | `EguiPlugin` added to app, no errors. |
| [x] 1.4.2 | Create debug panel showing current `GameState` | 1.4.1, 1.2.2 | Panel displays current state name. |
| [x] 1.4.3 | Add FPS counter to debug panel | 1.4.2 | FPS displayed in corner. |
| [x] 1.4.4 | Add state transition buttons to debug panel | 1.4.2, 1.2.3 | Buttons change state on click. |

---

## Phase 2: Combat MVP
### Epic 2.1: Physics Setup
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 2.1.1 | Integrate `avian2d` physics | 1.1.2 | `PhysicsPlugins::default()` added, no errors. |
| [x] 2.1.2 | Configure physics timestep for combat | 2.1.1 | Physics runs on `FixedUpdate` at 60Hz. |
| [x] 2.1.3 | Create test `RigidBody` entity that falls/moves | 2.1.2 | Entity responds to gravity/forces. |

### Epic 2.2: Ship Entity
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 2.2.1 | Define `Ship` marker component | 1.1.3 | Component exists in `src/components/ship.rs`. |
| [x] 2.2.2 | Define `Health` component (sails, rudder, hull) | 1.1.3 | Component with `sails`, `rudder`, `hull` fields and maxes. |
| [x] 2.2.3 | Define `Cargo` component | 1.1.3 | Component with `goods: HashMap<GoodType, u32>` and `capacity`. |
| [x] 2.2.4 | Define `Gold` component | 1.1.3 | Newtype component `Gold(u32)`. |
| [x] 2.2.5 | Create `spawn_player_ship` function | 2.2.1, 2.2.2, 2.1.1 | Spawns entity with `Ship`, `Player`, `Health`, `RigidBody`, `Sprite`. |
| [x] 2.2.6 | Create placeholder ship sprite | None | 64x64 PNG in `assets/sprites/ships/player.png`. |
| [x] 2.2.7 | Load and display ship sprite | 2.2.5, 2.2.6 | Player ship visible on screen. |

### Epic 2.3: Ship Movement
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 2.3.1 | Create `ShipMovementSystem` | 2.2.5, 1.3.5 | System queries `Ship` + `RigidBody`. |
| [x] 2.3.2 | Implement thrust (W key) | 2.3.1 | Ship accelerates forward when W pressed. |
| [x] 2.3.3 | Implement reverse (S key) | 2.3.1 | Ship accelerates backward when S pressed. |
| [x] 2.3.4 | Implement turn (A/D keys) | 2.3.1 | Ship rotates left/right. |
| [x] 2.3.5 | Apply drag to simulate water resistance | 2.3.1 | Ship slows down when no input. |
| [x] 2.3.6 | Implement anchor drop (Shift key) | 2.3.1 | Ship velocity set to zero, rotation allowed. |
| [x] 2.3.7 | Apply speed debuff based on sail damage | 2.3.1, 2.2.2 | `MaxSpeed` reduced proportionally to sail damage. |
| [x] 2.3.8 | Apply turn debuff based on rudder damage | 2.3.1, 2.2.2 | `TurnRate` reduced proportionally to rudder damage. |

### Epic 2.4: Cannons & Projectiles
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 2.4.1 | Define `Projectile` component | 1.1.3 | Component with `damage`, `target`, `source`. |
| [x] 2.4.2 | Define `TargetComponent` enum | 1.1.3 | Enum (Sails, Rudder, Hull) in `src/components/combat.rs`. |
| [x] 2.4.3 | Create `CannonState` resource | 1.1.3 | Resource tracks cooldown. |
| [x] 2.4.4 | Create `CannonFiringSystem` (Q/E Broadside) | 2.4.1, 2.4.3 | Q/E spawns spread of 3 projectiles from respective side. |
| [x] 2.4.5 | Create projectile sprite | None | 16x16 PNG loaded. |
| [x] 2.4.6 | Spawn projectile with velocity | 2.4.4, 2.4.5 | Projectiles spawn with ship velocity + ejection speed. |
| [x] 2.4.7 | Create `ProjectileSystem` (timers) | 2.4.6 | Projectiles despawn after 5s. |
| [x] 2.4.8 | Handle hit detection (Ships) | 2.4.7 | Collisions reduce `Health`. Self-hits prevented. |
| [x] 2.4.9 | Implement "Sticky" Input Buffering | 2.4.4 | Ensure firing intent is conserved during cooldown. |
| [x] 2.4.10| Add Visual Reference Grid | 2.4.4 | Draw background grid for movement cues. |

### Epic 2.5: Damage & Hit Detection
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 2.5.1 | Define `ShipHitEvent` (or specialized system) | 1.1.3 | Handled directly in `projectile_collision_system`. |
| [x] 2.5.2 | Implement collision detection (projectile vs ship) | 2.4.7, 2.2.5 | `avian2d` collision events processed. |
| [x] 2.5.3 | Emit `ShipHitEvent` on collision | 2.5.1, 2.5.2 | (Skipped: Direct component modification for MVP). |
| [x] 2.5.4 | Create `DamageSystem` | 2.5.3, 2.2.2 | `projectile_collision_system` updates `Health`. |
| [x] 2.5.5 | Implement `WaterIntake` component | 1.1.3 | Component with `rate` and `current` water level. |
| [x] 2.5.6 | Add `WaterIntake` on hull damage | 2.5.4, 2.5.5 | Hull damage adds/increases `WaterIntake`. |
| [x] 2.5.7 | Implement ship destruction (hull HP <= 0) | 2.5.4 | Ship entity despawned, `ShipDestroyedEvent` emitted. |

### Epic 2.6: Loot System
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 2.6.1 | Define `Loot` component | 1.1.3 | Component with `value`, `good_type`. |
| [x] 2.6.2 | Create loot sprite | None | 32x32 PNG in `assets/sprites/loot/gold.png`. |
| [x] 2.6.3 | Spawn loot on ship hit | 2.5.3, 2.6.1, 2.6.2 | Loot entity spawned at hit location. |
| [x] 2.6.4 | Make loot a `RigidBody` | 2.6.3 | Loot affected by physics. |
| [x] 2.6.5 | Implement loot collection (player collision) | 2.6.4, 2.2.5 | Loot despawned, added to player `Gold`/`Cargo`. |

### Epic 2.7: Current Zones
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 2.7.1 | Define `CurrentZone` component | 1.1.3 | Component with `velocity: Vec2`, `bounds: Rect`. |
| [x] 2.7.2 | Create `CurrentSystem` | 2.7.1 | Queries all `RigidBody` in zone, applies `ExternalForce`. |
| [x] 2.7.3 | Spawn test current zone | 2.7.1, 2.7.2 | Visible zone that pushes entities. |
| [x] 2.7.4 | Visualize current zones (subtle overlay) | 2.7.3 | Directional arrows or flow lines. |

### Epic 2.8: Enemy Ships & AI
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 2.8.1 | Define `AI` marker component | 1.1.3 | Component exists. |
| [x] 2.8.2 | Define `Faction` component | 1.1.3 | Component with `FactionId`. |
| [x] 2.8.3 | Create `spawn_enemy_ship` function | 2.8.1, 2.8.2, 2.2.2 | Spawns enemy with `AI`, `Faction`, `Ship`, `Health`. |
| [x] 2.8.4 | Create enemy ship sprite (different color) | None | 64x64 PNG in `assets/sprites/ships/enemy.png`. |
| [x] 2.8.5 | Create `CombatAISystem` | 2.8.3, 2.3.1 | Enemy chases player. |
| [x] 2.8.6 | Implement AI pursuit behavior | 2.8.5 | Enemy rotates toward and moves toward player. |
| [x] 2.8.7 | Implement AI firing logic | 2.8.5, 2.4.4 | Enemy fires when in range and facing player. |
| [x] 2.8.8 | Implement AI flee behavior (low HP) | 2.8.5 | Enemy turns and flees when HP < 20%. |

### Epic 2.9: Combat Flow
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 2.9.1 | Define `CombatEndedEvent` | 1.1.3 | Event with `victory: bool`. |
| [x] 2.9.2 | Detect all enemies destroyed | 2.5.7 | System checks if no `AI` + `Ship` entities remain. |
| [x] 2.9.3 | Emit `CombatEndedEvent` on victory | 2.9.1, 2.9.2 | Event sent when all enemies dead. |
| [x] 2.9.4 | Detect player destroyed | 2.5.7 | System checks if `Player` + `Ship` is gone. |
| [x] 2.9.5 | Emit `PlayerDiedEvent` | 2.9.4 | Event sent when player dies. |
| [x] 2.9.6 | Transition to `GameOverState` on player death | 2.9.5, 1.2.2 | State changes to `GameOver`. |
| [x] 2.9.7 | Transition to `HighSeasState` on combat victory | 2.9.3, 1.2.2 | State changes to `HighSeas`. |

---

## Phase 3: High Seas Map
### Epic 3.1: Tilemap Setup
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 3.1.1 | Integrate `bevy_ecs_tilemap` | 1.1.2 | Plugin added, no errors. |
| [x] 3.1.2 | Create tileset image (water, land variants) | None | PNG in `assets/tilemaps/tileset.png`. |
| [x] 3.1.3 | Create `TilemapPlugin` | 3.1.1 | Plugin manages map loading/rendering. |
| [x] 3.1.4 | Define `MapData` resource (tile grid) | 3.1.3 | Resource holds 2D array of tile types. |
| [x] 3.1.5 | Spawn tilemap from `MapData` | 3.1.4, 3.1.2 | Tilemap renders water and land tiles. |

### Epic 3.2: Procedural Generation
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 3.2.1 | Integrate `noise` crate | 1.1.2 | Crate available. |
| [x] 3.2.2 | Create `generate_world_map` function | 3.2.1, 3.1.4 | Returns `MapData` with procedural land/water. |
| [x] 3.2.3 | Use Perlin/Simplex noise for landmasses | 3.2.2 | Noise-based threshold determines land vs water. |
| [x] 3.2.4 | Ensure starting area is navigable | 3.2.2 | Player spawn point is always on water. |
| [x] 3.2.5 | Place port locations procedurally | 3.2.2 | Ports spawn on coastlines. |
| [ ] 3.2.6 | Remove forced sea at spawn | 3.2.4 | Remove `ensure_spawn_navigable` that forces water at center. |
| [ ] 3.2.7 | Dynamic spawn location detection | 3.2.6 | Spiral search from center to find navigable open water; store as `SpawnLocation` resource. |

### Epic 3.3: Fog of War
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 3.3.1 | Define `FogOfWar` resource (explored tile set) | 3.1.4 | Resource holds `HashSet<IVec2>` of explored tiles. |
| [x] 3.3.2 | Create `FogOfWarSystem` | 3.3.1 | Updates explored tiles based on player position. |
| [x] 3.3.3 | Render fog overlay on unexplored tiles | 3.3.2 | Dark overlay on tiles not in `FogOfWar`. |
| [x] 3.3.4 | Define player vision radius | 3.3.2 | Tiles within radius of player are revealed. |
| 3.3.5 | Lookout companion increases vision radius | 3.3.4 | If Lookout present, radius increased. |

### Epic 3.4: Wind System
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 3.4.1 | Define `Wind` resource | 1.1.3 | Resource with `direction: Vec2`, `strength: f32`. |
| [x] 3.4.2 | Create `WindSystem` | 3.4.1 | Updates wind periodically (slowly shifts). |
| [x] 3.4.3 | Display wind direction on HUD (compass rose) | 3.4.1, 1.4.1 | UI shows arrow indicating wind direction. |
| [x] 3.4.4 | Apply wind to navigation speed | 3.4.1 | Traveling with wind = faster, against = slower. |
| [x] 3.4.5 | Apply wind to combat movement | 3.4.1, 2.3.1 | Ships move faster downwind, slower upwind in combat. |

### Epic 3.5: Navigation
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 3.5.1 | Define `Destination` component | 1.1.3 | Component with `target: Vec2`. |
| [x] 3.5.2 | Create `NavigationSystem` | 3.5.1, 3.4.4 | Moves player toward destination each tick. |
| [x] 3.5.3 | Implement click-to-navigate | 3.5.1, 1.3.5 | Clicking on map sets `Destination`. |
| [x] 3.5.4 | Implement A* pathfinding around land | 3.5.2, 3.1.4 | Path avoids land tiles. |
| [x] 3.5.5 | Visualize planned path | 3.5.4 | Dotted line shows route. |
| 3.5.6 | Navigator companion auto-routes | 3.5.4 | If Navigator present, path is optimized. |
| [x] 3.5.7 | Detect arrival at port | 3.5.2 | When player reaches port tile, trigger `PortState`. |

### Epic 3.6: Encounters
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 3.6.1 | Create `SpatialHash` utility | 1.1.3 | Utility for efficient proximity queries. |
| [x] 3.6.2 | Spawn AI ships on map (world simulation placeholder) | 2.8.3 | AI ships exist on world map. |
| [x] 3.6.3 | Create `EncounterSystem` | 3.6.1, 3.6.2 | Checks if player near AI ship. |
| [x] 3.6.4 | Determine hostility based on faction reputation | 3.6.3, 2.8.2 | Pirates always hostile, others based on rep. |
| [x] 3.6.5 | Emit `CombatTriggeredEvent` | 3.6.4 | Event sent when hostile encounter. |
| [x] 3.6.6 | Transition to `CombatState` on encounter | 3.6.5, 1.2.2 | State changes to `Combat`. |
| [x] 3.6.7 | Transfer relevant entities to combat scene | 3.6.6 | Player ship and encountered enemies spawn in combat. |

---

## Phase 4: Ports & Economy
### Epic 4.1: Port Entity
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 4.1.1 | Define `Port` marker component | 1.1.3 | Component exists. |
| [x] 4.1.2 | Define `Inventory` component | 1.1.3 | Component with goods, quantities, prices. |
| [x] 4.1.3 | Create `spawn_port` function | 4.1.1, 4.1.2 | Spawns port entity with inventory. |
| [x] 4.1.4 | Generate initial inventory for ports | 4.1.3, 3.2.5 | Each port has random starting goods. |

### Epic 4.2: Port UI
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 4.2.1 | Create Port View layout | 1.4.1, 1.2.2 | UI displays when in `PortState`. |
| [x] 4.2.2 | Implement Market panel | 4.2.1 | Shows goods, prices, buy/sell buttons. |
| [x] 4.2.3 | Implement Tavern panel | 4.2.1 | Shows intel options, rumors. |
| [x] 4.2.4 | Implement Docks panel | 4.2.1 | Shows ship HP, repair options. |
| [x] 4.2.5 | Implement Contracts panel | 4.2.1 | Shows available contracts. |
| [x] 4.2.6 | Implement Depart button | 4.2.1 | Transitions to `HighSeasState`. |

### Epic 4.3: Trading
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 4.3.1 | Create `MarketSystem` | 4.2.2, 2.2.3, 2.2.4 | Handles buy/sell logic. |
| [x] 4.3.2 | Implement buy goods | 4.3.1 | Player gold decreases, cargo increases. |
| [x] 4.3.3 | Implement sell goods | 4.3.1 | Player cargo decreases, gold increases. |
| [x] 4.3.4 | Implement cargo capacity check | 4.3.2 | Cannot buy if cargo full. |
| [x] 4.3.5 | Emit `TradeExecutedEvent` | 4.3.1 | Event for audio/logging. |

### Epic 4.4: Price Dynamics
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 4.4.1 | Create `PriceCalculationSystem` | 4.1.2 | Runs on world tick. |
| [x] 4.4.2 | Adjust prices based on supply | 4.4.1 | Low stock = higher price. |
| [x] 4.4.3 | Adjust prices based on demand (global) | 4.4.1 | High demand goods = higher price everywhere. |
| [x] 4.4.4 | Implement goods decay (perishables) | 4.4.1, 2.2.3 | Perishable goods lose value over time. |

### Epic 4.5: Contracts
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 4.5.1 | Define `Contract` entity and components | 1.1.3 | `Contract`, `ContractType`, `Origin`, `Destination`, `Reward`, `Expiry`. |
| [x] 4.5.2 | Create `ContractGenerationSystem` | 4.5.1 | Procedurally creates contracts per port. |
| [x] 4.5.3 | Implement contract acceptance | 4.5.1, 4.2.5 | Player accepts, contract added to active list. |
| [x] 4.5.4 | Implement contract tracking | 4.5.3 | Track progress (cargo delivered, area explored). |
| [x] 4.5.5 | Implement contract completion | 4.5.4 | On conditions met, reward paid, `ContractCompletedEvent`. |
| [x] 4.5.6 | Implement contract expiry | 4.5.1 | Contracts expire based on WorldClock. |

### Epic 4.6: Ship Repair
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 4.6.1 | Create `RepairSystem` | 4.2.4, 2.2.2, 2.2.4 | Repairs cost gold. |
| [x] 4.6.2 | Implement repair sails | 4.6.1 | Restore `sails` HP. |
| [x] 4.6.3 | Implement repair rudder | 4.6.1 | Restore `rudder` HP. |
| [x] 4.6.4 | Implement repair hull | 4.6.1 | Restore `hull` HP, remove `WaterIntake`. |
| [x] 4.6.5 | Display repair costs | 4.6.1, 4.2.4 | UI shows cost per component. |

---

## Phase 5: World Simulation & Orders
### Epic 5.1: World Tick
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 5.1.1 | Define `WorldClock` resource | 1.1.3 | Resource with `day`, `hour`, `tick`. |
| [x] 5.1.2 | Create `WorldTickSystem` | 5.1.1 | Runs on `FixedUpdate`, increments clock. |
| [x] 5.1.3 | Display time on HUD | 5.1.1, 1.4.1 | Shows "Day X, Hour Y". |

### Epic 5.2: Faction AI
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 5.2.1 | Define `FactionRegistry` resource | 1.1.3 | Holds state for each faction. |
| [x] 5.2.2 | Define `FactionState` struct | 5.2.1 | Gold, ships, reputation, trade routes. |
| [x] 5.2.3 | Create `FactionAISystem` | 5.2.1, 5.1.2 | Runs per world tick. |
| [x] 5.2.4 | Implement trade route generation | 5.2.3 | Faction AI creates routes between ports. |
| [x] 5.2.5 | Implement ship spawning by faction | 5.2.3, 2.8.3 | Faction spawns ships to fulfill routes. |
| [x] 5.2.6 | Implement threat response | 5.2.3 | Faction sends ships to combat player if hostile. |

### Epic 5.3: AI Ship Behavior
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 5.3.1 | Define `Order` enum | 1.1.3 | `TradeRoute`, `Patrol`, `Escort`, `Scout`. |
| [x] 5.3.2 | Define `OrderQueue` component | 5.3.1 | Queue of `Order`. |
| [x] 5.3.3 | Create `OrderExecutionSystem` | 5.3.2 | Reads orders, drives ship navigation. |
| [x] 5.3.4 | Implement `TradeRoute` order | 5.3.3 | Ship navigates from A to B, trades, repeats. |
| [x] 5.3.5 | Evaluate AI pathfinding with caching | 5.3.4 | Plan how AI ships use A* pathfinding with route caching (e.g., trade routes reused). |
| [x] 5.3.5a | Define `RouteCache` resource | 5.3.5 | Resource `HashMap<(IVec2, IVec2), Vec<IVec2>>`. |
| [x] 5.3.5b | Integrate `RouteCache` into `ai_pathfinding_system` | 5.3.5a | Use cache for AI navigation, fall back to Theta*. |
| [x] 5.3.5c | Register `RouteCache` in `WorldMapPlugin` | 5.3.5a | Initialize resource. |
| [x] 5.3.6 | Implement `Patrol` order | 5.3.3 | Ship moves around area, engages hostiles. |
| [x] 5.3.7 | Implement `Escort` order | 5.3.3 | Ship follows target entity. |
| [x] 5.3.8 | Implement `Scout` order | 5.3.3 | Ship explores area, reports intel. |

### Epic 5.4: Player Fleet Management
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 5.4.1 | Allow player to own multiple ships | 2.2.5 | Player has list of owned ships. |
| [x] 5.4.2 | Create Fleet Management UI | 5.4.1, 1.4.1 | Shows owned ships and their orders. |
| [x] 5.4.3 | Allow issuing orders to owned ships | 5.4.2, 5.3.2 | UI to assign `TradeRoute`, `Patrol`, etc. |
| [x] 5.4.4 | Implement subcontracting (delegate contract) | 5.4.3, 4.5.3 | Assign contract to owned ship for cut. |

### Epic 5.5: Scale Testing
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 5.5.1 | Spawn 1000 AI ships | 5.2.5 | 1000 ships exist in simulation. |
| [skip] 5.5.2 | Profile frame rate | 5.5.1 | Game runs > 30 FPS with 1000 ships. |
| [skip] 5.5.3 | Optimize with spatial hashing | 5.5.2, 3.6.1 | Reduce O(n²) checks. |
| [skip] 5.5.4 | Optimize with LOD (hide distant ships) | 5.5.2 | Only render ships near camera. |

---

## Phase 6: Intel & Companions
### Epic 6.1: Intel System
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 6.1.1 | Define `Intel` entity and components | 1.1.3 | `Intel`, `IntelType`, `MapData`, `Expiry`. |
| [x] 6.1.2 | Create `IntelAcquiredEvent` | 6.1.1 | Event with intel data. |
| [x] 6.1.3 | Create `IntelSystem` | 6.1.2, 3.3.1 | Adds revealed data to map on acquisition. |
| [x] 6.1.4 | Implement intel expiry | 6.1.1, 5.1.2 | Transient intel removed after TTL. |
| [x] 6.1.5 | Implement tavern intel purchase | 6.1.2, 4.2.3 | Player buys intel at tavern. |
| [x] 6.1.6 | Visualize intel on map (icons, routes) | 6.1.3 | Ship routes shown as lines, etc. |

### Epic 6.2: Companions
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 6.2.1 | Define `Companion` entity and components | 1.1.3 | `Companion`, `Role`, `Skills`, `AssignedTo`. |
| [x] 6.2.2 | Define `CompanionRole` enum | 6.2.1 | `Quartermaster`, `Navigator`, `Lookout`, `Gunner`, `Mystic`. |
| [x] 6.2.3 | Create `spawn_companion` function | 6.2.1, 6.2.2 | Creates companion entity. |
| [x] 6.2.4 | Implement companion recruitment (tavern) | 6.2.3, 4.2.3 | Player recruits at tavern for gold. |
| [x] 6.2.5 | Create companion roster UI | 6.2.4, 1.4.1 | Shows recruited companions. |
| [x] 6.2.6 | Implement Quartermaster ability | 6.2.2, 4.3.1 | Auto-trades based on market intel. |
| [x] 6.2.7 | Implement Navigator ability | 6.2.2, 3.5.6 | Auto-routes efficient paths. |
| [x] 6.2.8 | Implement Lookout ability | 6.2.2, 3.3.5 | Increases vision radius. |
| [x] 6.2.9 | Implement Gunner ability | 6.2.2, 2.4.3 | Reduces cannon cooldown. |

---

## Phase 7: Progression & Persistence
### Epic 7.1: Meta Profile
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 7.1.1 | Define `MetaProfile` resource | 1.1.3 | Stats, unlocks, legacy wrecks. |
| [x] 7.1.2 | Load `MetaProfile` on app start | 7.1.1 | Loaded from file or created fresh. |
| [x] 7.1.3 | Save `MetaProfile` on death/quit | 7.1.1 | Written to file. |
| [x] 7.1.4 | Define player stats (Charisma, Navigation, Logistics) | 7.1.1 | Stats affect game systems. |
| [x] 7.1.5 | Implement stat progression (XP or milestones) | 7.1.4 | Stats increase over runs. |

### Epic 7.2: Archetypes
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 7.2.1a | Define `Archetype` enum and `ArchetypeConfig` struct | 7.1.1 | Enum with 4 variants, config with gold/rep/ship fields. |
| [x] 7.2.1b | Create `ArchetypeRegistry` resource | 7.2.1a | Resource maps archetypes to configs, registered in plugin. |
| [x] 7.2.1c | Add `unlocked_archetypes` to `MetaProfile` | 7.2.1a | Vec persisted to profile.json, Default always unlocked. |
| [x] 7.2.2 | Implement `check_archetype_unlocks` system | 7.2.1c | Runs on profile load, checks lifetime stats for unlocks. |
| [x] 7.2.3 | Implement archetype selection UI | 7.2.1b, 1.2.2 | MainMenu shows unlocked archetypes, stores selection. |
| [x] 7.2.4 | Apply archetype bonuses on game start | 7.2.3 | `spawn_player_ship` reads selection, applies gold/rep/ship. |

### Epic 7.3: Legacy Wrecks
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 7.3.1 | Record wreck on player death | 2.9.5, 7.1.1 | Position and cargo saved to `MetaProfile`. |
| [x] 7.3.2 | Spawn legacy wrecks on new run | 7.3.1, 3.2.2 | Wrecks placed on map. |
| [x] 7.3.3 | Implement wreck exploration | 7.3.2 | Player can loot wreck for cargo/gold. |

### Epic 7.4: Save/Load
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 7.4.1 | Integrate `bevy_save` | 1.1.2 | Plugin added. |
| [x] 7.4.2 | Implement save game | 7.4.1 | All relevant entities/resources serialized. |
| [x] 7.4.3 | Implement load game | 7.4.1 | World reconstructed from save file. |
| [x] 7.4.4 | Implement autosave | 7.4.2 | Auto-triggers on state transitions. |
| [x] 7.4.5 | Add Save/Load to main menu | 7.4.2, 7.4.3, 4.2.1 | UI buttons work. |
| [x] 7.4.6 | Generate save presets to make feature testing easier | 7.4.2, 7.4.3 |

### Epic 7.5: Code Cleanup
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 7.5.1 | Fix all compiler warnings | 7.4.6 | `cargo check` produces zero warnings. |

---

## Phase 8: Audio & Polish
### Epic 8.1: Visual Polish
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 8.1.0 | Fix EguiUserTextures panic on startup | None | Game launches without panic. |
| [x] 8.1.1 | Create "Ink and Parchment" shader | 1.1.2 | WGSL shader in `assets/shaders/`. |
| [x] 8.1.2 | Apply shader as post-processing | 8.1.1 | Entire game has parchment tint. |
| [x] 8.1.3 | Create parchment texture for UI | None | PNG in `assets/sprites/ui/`. |
| [x] 8.1.4 | Style UI panels with parchment texture | 8.1.3, 1.4.1 | Egui panels have parchment bg. |
| [x] 8.1.5 | Add scroll/dagger decorations to UI | 8.1.4 | Decorative elements on panels. |
| [x] 8.1.6 | Add screen shake on cannon fire | 2.4.4 | Camera shakes briefly. |
| [x] 8.1.7 | Add hit flash on ship damage | 2.5.4 | Ship sprite flashes white. |
| [x] 8.1.8 | Add dynamic wake behind ships | 2.5.4 | Beautiful wake effect. |

### Epic 8.2: The Weathered Document (Paper Physicality)
> Make the screen feel like a physical piece of aged parchment.

| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 8.2.1 | Add paper texture to post-process shader | 8.1.2 | Shader samples `parchment.png` overlay. |
| [x] 8.2.2 | Implement vignette darkening | 8.2.1 | Screen edges darken like aged paper. |
| [x] 8.2.3 | Add procedural paper grain noise | 8.2.1 | Subtle fiber texture via FBM noise. |

### Epic 8.3: The Cartographer's Sketch (Edge Detection)
> Make everything look hand-drawn with quill and ink.

| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 8.3.1 | Implement Sobel edge detection | 8.1.2 | Shader extracts edges from scene. |
| [x] 8.3.2 | Render edges as ink strokes | 8.3.1 | Edges drawn in ink color, fill muted. |
| [x] 8.3.3 | Add line wobble displacement | 8.3.2 | Edges have hand-drawn imperfection. |
| [x] 8.3.4 | Implement variable line weight | 8.3.2 | Important edges thicker than details. |
| [x] 8.3.5 | Add crosshatch shading for shadows | 8.3.1 | Dark areas show crosshatch pattern. |

### Epic 8.4: Living Ink (Dynamic Fluid Effects)
> Make the ink feel alive and responsive.

| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 8.4.1 | Create `InkReveal` component | 3.3.1 | Tracks reveal animation progress. |
| [x] 8.4.2 | Implement fog reveal ink animation | 8.4.1 | New areas "draw in" with spreading ink. |
| [x] 8.4.3 | Add ship wake ink trails | 2.3.1 | Ships leave fading ink strokes. |
| [x] 8.4.4 | Implement damage ink splatter | 2.5.4 | Hits cause brief ink splatter VFX. |
| [x] 8.4.5 | Add water area ink wash effect | 8.2.1 | Ocean has watercolor bleeding at edges. |
| [x] 8.4.6 | Implement UI text write-on animation | 4.2.1 | Text appears stroke-by-stroke. |

### Epic 8.5: Historical Cartography
> Make the map feel like an authentic 18th-century nautical chart.

| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [x] 8.5.0 | Integrate `bevy_prototype_lyon` | 1.1.2 | Crate added (`v0.13`), `ShapePlugin` registered. |
| [x] 8.5.1 | Extract coastline polygons from tilemap | 3.1.5 | System converts land/water tiles to closed polygon paths. |
| [x] 8.5.2 | Render base coastline stroke (Lyon) | 8.5.0, 8.5.1 | Main coastal outline drawn; debug toggle in UI. |
| [x] 8.5.3 | Ensure coastline polygons are orientable | 8.5.2 | Polygons closed or hit map border; sea/land sides identifiable. |
| [x] 8.5.4 | Smooth coastline with Catmull-Rom splines | 8.5.3 | Raw tile edges become smooth curves. |
| [x] 8.5.5 | Add tunable noise/jitter to coastlines | 8.5.4 | Lines have organic irregularity (hand-drawn wobble). |
| [x] 8.5.6 | Implement waterlining effect (Lyon) | 8.5.4 | Three offset strokes, progressively lighter/thinner. |
| [x] 8.5.7 | Create `CompassRose` UI component | 1.4.1 | Decorative compass rose renders on screen. |
| [x] 8.5.7.1 | Make compass rose dynamic | 8.5.7 | Rose implementation uses Overlay Camera (RenderLayers). |
| [x] 8.5.8 | Create `ScaleBar` UI component | 1.4.1 | "Scale of Miles" bar renders on map. |
| [x] 8.5.9 | Make scale bar zoom-responsive | 8.5.8, 1.3.3 | Bar adjusts label/length based on camera zoom. |
| [x] 8.5.10 | Define `WaterDepth` tile attribute | 3.1.4 | Tilemap stores depth values for water tiles. |
| [x] 8.5.11 | Generate depth data from noise | 8.5.10, 3.2.1 | Depth decreases near coastlines (shallow) and increases offshore. |
| [x] 8.5.12 | Create stippling shader (Blue Noise) | 8.5.11 | Shader uses noise texture to render dots based on depth. |
| [x] 8.5.13 | Integrate stippling into map rendering | 8.5.12 | Shallow water shows dense dots, deep water sparse/none. |
| [x] 8.5.14 | Load Quintessential font | None | `assets/fonts/Quintessential-Regular.ttf` available. |
| [x] 8.5.15 | Create `LocationLabel` component | 4.1.1 | Component with name, importance rank, position. |
| [x] 8.5.16 | Calculate label perpendicular angle | 8.5.15, 8.5.4 | System computes angle perpendicular to nearest coastline. |
| [x] 8.5.17 | Render location labels with Quintessential font | 8.5.14, 8.5.16 | Labels drawn extending inland, perpendicular to coast. |
| [x] 8.5.18 | Scale label text by importance | 8.5.17 | Major ports large, minor locations smaller. |
| [x] 8.5.19 | Add decorative cartouche for map title | 8.5.14 | Ornate frame for map title/legend area. |

---

## Phase 9: Infrastructure & Performance
### Epic 9.1: Performance Profiling
> Enable measurement and optimization of game performance.

| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [ ] 9.1.1 | Add `bevy_diagnostics` FrameTimeDiagnostics | 1.1.2 | FPS and frame time logged to console. |
| [ ] 9.1.2 | Integrate Tracy profiler support | 9.1.1 | Compile with `--features bevy/trace_tracy`, connect Tracy. |
| [ ] 9.1.3 | Add diagnostic overlay (toggle with F4) | 9.1.1, 1.4.1 | Egui panel shows FPS, entity count, draw calls. |
| [ ] 9.1.4 | Document profiling workflow in AGENT.md | 9.1.2 | Instructions for running Tracy and interpreting results. |

### Epic 9.2: Scene Management & Entity Cleanup
> Track entity ownership per scene and despawn correctly on state transitions.

| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [ ] 9.2.1 | Create `SceneTag` marker components | 1.2.1 | `HighSeasEntity`, `CombatEntity`, `PortEntity` marker components. |
| [ ] 9.2.2 | Tag entities at spawn time | 9.2.1 | All spawned entities get appropriate `SceneTag`. |
| [ ] 9.2.3 | Create `despawn_scene_entities` generic system | 9.2.1 | System despawns all entities with given `SceneTag`. |
| [ ] 9.2.4 | Register despawn systems on `OnExit` for each state | 9.2.3 | Exiting HighSeas/Combat/Port despawns tagged entities. |
| [ ] 9.2.5 | Audit existing plugins for entity tagging | 9.2.2 | All worldmap, combat, port entities tagged. |
| [ ] 9.2.6 | Remove redundant per-plugin despawn systems | 9.2.4 | Consolidate to centralized cleanup. |

### Epic 9.3: Loading Screens
> Show progress indicator during scene transitions.

| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [ ] 9.3.1 | Create `Loading` state variant | 1.2.1 | `GameState::Loading` exists with target state field. |
| [ ] 9.3.2 | Create `LoadingScreen` plugin | 9.3.1 | Plugin spawns loading UI on `OnEnter(Loading)`. |
| [ ] 9.3.3 | Implement loading progress resource | 9.3.2 | `LoadingProgress { current: f32, message: String }`. |
| [ ] 9.3.4 | Add loading bar UI with progress text | 9.3.2, 9.3.3 | Animated bar and status message displayed. |
| [ ] 9.3.5 | Transition to target state on completion | 9.3.3 | When progress reaches 1.0, switch to target state. |
| [ ] 9.3.6 | Integrate loading screen into HighSeas entry | 9.3.5, 3.1.1 | Entering HighSeas shows loading during map gen. |
| [ ] 9.3.7 | Integrate loading screen into Combat entry | 9.3.5, 2.1.1 | Entering Combat shows brief loading. |