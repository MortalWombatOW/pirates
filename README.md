# Pirates - Complete Game Design Document & Technical Specification

> **This document is the single source of truth for the Pirates game.**
> It contains all design decisions, technical architecture, and implementation details.
> AI agents will use this document to generate implementation tasks.

---

## Table of Contents
1. [Project Overview](#1-project-overview)
2. [Core Gameplay Loop](#2-core-gameplay-loop)
3. [Game States & Views](#3-game-states--views)
4. [Game Systems](#4-game-systems)
5. [Entity & Component Definitions](#5-entity--component-definitions)
6. [Technical Architecture](#6-technical-architecture)
7. [Plugin Structure](#7-plugin-structure)
8. [Dependencies & Crates](#8-dependencies--crates)
9. [File Structure](#9-file-structure)
10. [Implementation Phases](#10-implementation-phases)

---

## Quick Start

```bash
# First-time setup: run check to compile dependencies (faster than build)
cargo check

# Run the game
cargo run

# Build with timing info (useful for profiling slow builds)
cargo build --timings
```

> [!TIP]
> Always run `cargo check` first when pulling new changes or updating dependencies. It's faster than `cargo build` because it skips code generation.

> [!IMPORTANT]
> **Commit and push after completing every task.** This ensures progress is persisted and visible to the team/other agents.

---


## 1. Project Overview

### 1.1 High Concept
**Pirates** is a 2D top-down roguelike merchant/strategy/combat game. Players captain a ship, trade goods, gather intelligence, and engage in physics-based naval combat. The world is simulated with 1000+ AI-controlled ships, procedural economies, and faction politics.

### 1.2 Genre & Pillars
- **Genre**: Top-down Strategy / Merchant Sim / Tactical Combat / Roguelike
- **Pillars**:
  1. **Trade**: Buy low, sell high. Manage cargo and contracts.
  2. **Explore**: Uncover a fog-of-war map through intelligence gathering.
  3. **Fight**: Physics-based naval combat with targetable ship components.
  4. **Survive**: Roguelike progression with meta-unlocks.

### 1.3 Target Platform
- **Platform**: Desktop (Windows, macOS, Linux)
- **Distribution**: Steam
- **Controls**: Keyboard (WASD) + Mouse

### 1.4 Aesthetic
- **Visual Style**: "Ink and Parchment" — 2D graphics rendered to look like quill and ink on aged parchment.
- **UI Style**: Diegetic design using scrolls, daggers, and parchment textures.
- **Audio**: Scene-specific ambience (creaking hulls, tavern chatter) + orchestral/folk soundtrack.

---

## 2. Core Gameplay Loop

```
┌─────────────────────────────────────────────────────────────────┐
│                         CORE LOOP                               │
├─────────────────────────────────────────────────────────────────┤
│  1. PORT PHASE                                                  │
│     - Trade goods at the Market                                 │
│     - Gather intel at the Tavern                                │
│     - Recruit companions                                        │
│     - Repair/upgrade ship at the Docks                          │
│     - Accept contracts                                          │
│                                                                 │
│  2. NAVIGATION PHASE (High Seas View)                           │
│     - Set course on the map                                     │
│     - Wind and weather affect travel                            │
│     - Encounter events (storms, other ships)                    │
│     - Fog of War reveals as you explore                         │
│                                                                 │
│  3. COMBAT PHASE (if engaged)                                   │
│     - Real-time physics-based combat                            │
│     - WASD movement, Spacebar to fire                           │
│     - Target enemy components (Sails, Rudder, Hull)             │
│     - Loot ejected into water with currents                     │
│                                                                 │
│  4. PROFIT & GROWTH                                             │
│     - Sell goods at destination                                 │
│     - Upgrade ship and crew                                     │
│     - Expand fleet                                              │
│     - Repeat                                                    │
│                                                                 │
│  [ON DEATH] → Meta-progression applied → New run begins         │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Game States & Views

The game uses a state machine to manage transitions between views.

### 3.1 State Definitions

| State | Description | Entry Condition | Exit Condition |
|---|---|---|---|
| `MainMenuState` | Title screen, new game, load game, options. | App launch. | User selects New Game or Load. |
| `PortState` | Player is docked at a port. | Arriving at a port. | Player selects "Depart". |
| `HighSeasState` | Player is navigating the world map. | Departing a port. | Arriving at port OR encounter triggered. |
| `CombatState` | Real-time tactical combat. | Encounter with hostile ship. | All enemies defeated OR player flees/dies. |
| `GameOverState` | Player's ship is destroyed. | Player ship HP <= 0. | User returns to Main Menu. |

### 3.2 State Transitions

```
MainMenuState ──[New Game]──> PortState (starting port)
MainMenuState ──[Load Game]──> (restored state)

PortState ──[Depart]──> HighSeasState
HighSeasState ──[Arrive at Port]──> PortState
HighSeasState ──[Encounter Hostile]──> CombatState
CombatState ──[Victory/Flee]──> HighSeasState
CombatState ──[Player Dies]──> GameOverState

GameOverState ──[Continue]──> MainMenuState (meta-progression saved)
```

### 3.3 View Details

#### A. Port View
- **Camera**: Static, centered on port illustration.
- **UI Elements**:
  - Market panel (buy/sell goods)
  - Tavern panel (intel, rumors, recruitment)
  - Docks panel (ship repair, upgrades)
  - Contracts panel (available jobs)
  - Depart button
- **Systems Active**: `MarketSystem`, `RecruitmentSystem`, `ContractSystem`, `ShipRepairSystem`.

#### B. High Seas View
- **Camera**: Pan/zoom on world map, centered on player ship.
- **Rendering**:
  - Tilemap for land/water.
  - Fog of War overlay (unexplored tiles are obscured).
  - Wind direction indicator (compass rose).
  - Intel markers (ship routes, storms, treasure).
- **Controls**:
  - Click to set destination.
  - Scroll to zoom.
  - Drag to pan (optional).
- **Systems Active**: `NavigationSystem`, `WindSystem`, `FogOfWarSystem`, `EncounterSystem`, `WorldSimulationSystem`.

#### C. Combat View
- **Camera**: Follows player ship, with some zoom-out to show arena.
- **Rendering**:
  - Ships as sprites with rotation.
  - Cannonball projectiles.
  - Floating loot.
  - Current zones (visualized as subtle directional overlay).
  - Land obstacles (if near coast).
- **Controls**:
  - WASD: Thrust/turn.
  - Spacebar: Fire cannons.
  - Tab: Cycle target component (Sails/Rudder/Hull).
  - Shift: Drop anchor (for whip turns).
- **Systems Active**: `ShipMovementSystem`, `CannonSystem`, `ProjectileSystem`, `DamageSystem`, `LootSystem`, `CurrentSystem`, `CombatAISystem`.

---

## 4. Game Systems

Each system is a Bevy plugin with defined responsibilities.

### 4.1 World Simulation System
**Purpose**: Simulate the world at a macro level (1000 ships, faction economies).

| Responsibility | Details |
|---|---|
| Tick-based update | Runs on `FixedUpdate` schedule (e.g., 1 tick/second for world sim). |
| Faction AI | Each of the 3 nations + pirates has an AI that sets trade routes, responds to threats. |
| Ship Spawning | AI ships are spawned/destroyed based on faction needs. |
| Event Scheduling | Queue world events (storms, convoys) to trigger at specific times. |

**Resources**:
- `WorldClock`: Tracks in-game time (day, hour).
- `FactionRegistry`: Holds state for each faction (gold, reputation, active ships).

---

### 4.2 Navigation System
**Purpose**: Handle player movement on the High Seas map.

| Responsibility | Details |
|---|---|
| Pathfinding | Calculate route from current position to destination, avoiding land. |
| Wind Effect | Adjust travel speed based on wind direction relative to heading. |
| Travel Progress | Move player icon along path each tick. |
| Arrival Detection | Trigger `PortState` when arriving at a port. |

**Components Used**: `Position`, `Destination`, `Speed`, `WindSailModifier`.

---

### 4.3 Encounter System
**Purpose**: Determine when the player encounters events on the High Seas.

| Responsibility | Details |
|---|---|
| Proximity Check | Use spatial hashing to find AI ships near player. |
| Hostility Check | If nearby ship is hostile (based on faction reputation), trigger combat. |
| Event Triggers | Random events (storms, floating wreckage, intel opportunities). |

**Events Emitted**: `CombatTriggeredEvent`, `StormEncounterEvent`, `DiscoveryEvent`.

---

### 4.4 Combat Systems

#### 4.4.1 Ship Movement System
- Read `InputState` (WASD).
- Apply forces to `RigidBody` via Rapier.
- Account for wind direction (upwind = slower, downwind = faster).
- Handle anchor drop (lock position, enable whip turn).

#### 4.4.2 Cannon System
- On Spacebar press, check cooldown timer.
- Spawn `Projectile` entity aimed at targeted component.
- Apply recoil force to firing ship.

#### 4.4.3 Projectile System
- Move projectiles each frame.
- On collision with ship, emit `ShipHitEvent`.
- Despawn on collision or timeout.

#### 4.4.4 Damage System
- Listen for `ShipHitEvent`.
- Reduce HP of targeted component (Sails, Rudder, or Hull).
- Apply debuffs:
  - Sails damage: reduce `MaxSpeed`.
  - Rudder damage: reduce `TurnRate`.
  - Hull damage: reduce both slightly, add `WaterIntake` component.
- Spawn `Loot` entities at impact point.

#### 4.4.5 Loot System
- Loot entities have `RigidBody` and are affected by currents.
- Player can collect by colliding with loot.
- Add to player `Cargo` or `Gold`.

#### 4.4.6 Current System
- Define zones with `CurrentZone` component (velocity vector).
- Apply force to all `RigidBody` entities within zone each frame.

#### 4.4.7 Combat AI System
- Enemy ships use steering behaviors:
  - Pursue player.
  - Flee if HP low.
  - Flank for broadside.
- Fire cannons when in range and facing target.

---

### 4.5 Economy System
**Purpose**: Manage goods, prices, and trade.

| Responsibility | Details |
|---|---|
| Port Inventory | Each port has `Inventory` component with goods and quantities. |
| Price Calculation | Prices based on supply (local) and demand (global). |
| Trade Execution | Transfer goods between player `Cargo` and port `Inventory`. |
| Goods Decay | Perishable goods lose value over time. |

**Components Used**: `Inventory`, `Cargo`, `Gold`, `GoodsTrait` (Perishable, Heavy, Illegal).

---

### 4.6 Contract System
**Purpose**: Generate and manage jobs/quests.

| Responsibility | Details |
|---|---|
| Contract Generation | Procedurally create contracts (transport, explore, escort). |
| Contract Tracking | Track progress (cargo delivered, area explored). |
| Reward Payout | On completion, add gold/reputation to player. |
| Subcontracting | Player can delegate contracts to owned ships or NPCs. |

**Entity**: `Contract` with components `ContractType`, `Origin`, `Destination`, `Reward`, `Expiry`.

---

### 4.7 Orders System
**Purpose**: Command hierarchy for fleets.

| Responsibility | Details |
|---|---|
| Order Queue | Each ship has an `OrderQueue` component (list of `Order`). |
| Order Types | `TradeRoute(A, B)`, `Patrol(Area)`, `Escort(Target)`, `Scout(Area)`. |
| Order Execution | Systems read orders and drive navigation/behavior. |
| Player Fleet Management | UI to assign orders to owned ships. |

---

### 4.8 Intelligence System
**Purpose**: Information as currency.

| Responsibility | Details |
|---|---|
| Intel Acquisition | Gained from taverns, purchased, or generated by companions. |
| Intel Types | Permanent (island location) vs. Transient (ship route, storm forecast). |
| Map Injection | When acquired, intel reveals data on the player's map. |
| Expiry | Transient intel has a `TimeToLive` and decays. |

**Entity**: `Intel` with components `IntelType`, `MapData`, `Expiry`.

---

### 4.9 Companion System
**Purpose**: Manage officers and their abilities.

| Responsibility | Details |
|---|---|
| Roster | Player has a list of recruited companions. |
| Assignment | Companions are assigned to ships or roles. |
| Abilities | Each companion has skills that activate automatically. |

**Companion Roles**:
| Role | Ability |
|---|---|
| Quartermaster | Auto-trades goods based on market intel. |
| Navigator | Auto-plots efficient routes considering wind/dangers. |
| Lookout | Passive chance to reveal distant threats/features. |

**Late-Game Magic Abilities** (unlocked via supernatural shift):
- Burn Sails: DoT on enemy sails.
- Freeze Rudder: Lock enemy turn rate.
- Invisibility: Temporary stealth.
- Wind Manipulation: Change local wind direction.

---

### 4.10 Progression System
**Purpose**: Roguelike meta-progression.

| Responsibility | Details |
|---|---|
| Run Tracking | Record stats for current run (gold earned, ships sunk, distance traveled). |
| Meta-Stats | Persistent D&D-style attributes that grow over runs: Charisma, Navigation, Logistics, etc. |
| Archetype Unlocks | Unlock new starting roles (Royal Navy Captain, Smuggler, Castaway). |
| Legacy Wrecks | On death, record ship location/cargo for future runs to find. |

**Resources**: `MetaProfile` (loaded at startup, saved on death/quit).

---

### 4.11 Save/Load System
**Purpose**: Persist game state.

| Responsibility | Details |
|---|---|
| Save Game | Serialize world state, player state, and meta-profile. |
| Load Game | Deserialize and reconstruct ECS world. |
| Autosave | Trigger on state transitions (entering port, after combat). |

**Implementation**: Use `bevy_save` crate.

---

### 4.12 Audio System
**Purpose**: Manage music and sound effects.

| Responsibility | Details |
|---|---|
| Scene Music | Play different tracks for Port, High Seas, Combat. |
| Ambience | Layer ambient sounds (waves, creaking, wind). |
| SFX | Trigger sounds on events (cannon fire, hit, purchase, UI click). |

**Implementation**: Use `bevy_kira_audio` crate.

---

### 4.13 Rendering System
**Purpose**: Draw the game.

| Responsibility | Details |
|---|---|
| Sprite Rendering | Draw ships, ports, loot, UI elements. |
| Tilemap Rendering | Draw High Seas map with `bevy_ecs_tilemap`. |
| Shader Effects | Apply "Ink and Parchment" post-processing shader. |
| UI Rendering | Draw HUD and menus with `bevy_egui`. |

---

## 5. Entity & Component Definitions

### 5.1 Core Entities

| Entity | Purpose | Key Components |
|---|---|---|
| `PlayerShip` | The player's controlled vessel. | `Ship`, `Player`, `RigidBody`, `Position`, `Health`, `Cargo`, `OrderQueue`. |
| `AIShip` | NPC-controlled vessel. | `Ship`, `AI`, `Faction`, `RigidBody`, `Position`, `Health`, `Cargo`, `OrderQueue`. |
| `Port` | A location for trading and services. | `Port`, `Position`, `Inventory`, `Faction`. |
| `Projectile` | A fired cannonball. | `Projectile`, `RigidBody`, `Velocity`, `Damage`, `TargetComponent`. |
| `Loot` | Floating resource in water. | `Loot`, `RigidBody`, `Value`, `GoodType`. |
| `Intel` | A piece of information. | `Intel`, `IntelType`, `MapData`, `Expiry`. |
| `Contract` | A job/quest. | `Contract`, `ContractType`, `Origin`, `Destination`, `Reward`, `Expiry`. |
| `Companion` | An officer/crew member. | `Companion`, `Role`, `Skills`, `AssignedTo`. |
| `CurrentZone` | An area affecting physics. | `CurrentZone`, `Velocity`, `Bounds`. |

### 5.2 Component Definitions

```rust
// === Markers ===
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct AI;

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct Port;

#[derive(Component)]
pub struct Projectile;

#[derive(Component)]
pub struct Loot;

#[derive(Component)]
pub struct Intel;

#[derive(Component)]
pub struct Contract;

#[derive(Component)]
pub struct Companion;

#[derive(Component)]
pub struct CurrentZone;

// === Data Components ===
#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Health {
    pub sails: f32,
    pub sails_max: f32,
    pub rudder: f32,
    pub rudder_max: f32,
    pub hull: f32,
    pub hull_max: f32,
}

#[derive(Component)]
pub struct Cargo {
    pub goods: HashMap<GoodType, u32>,
    pub capacity: u32,
}

#[derive(Component)]
pub struct Gold(pub u32);

#[derive(Component)]
pub struct Faction(pub FactionId);

#[derive(Component)]
pub struct Inventory {
    pub goods: HashMap<GoodType, (u32, f32)>, // (quantity, price)
}

#[derive(Component)]
pub struct OrderQueue {
    pub orders: VecDeque<Order>,
}

#[derive(Component)]
pub struct Destination {
    pub target: Vec2,
}

#[derive(Component)]
pub struct Expiry {
    pub remaining_ticks: u32,
}

#[derive(Component)]
pub struct Damage(pub f32);

#[derive(Component)]
pub struct TargetComponent(pub ShipComponent);

#[derive(Component)]
pub struct WaterIntake {
    pub rate: f32,
    pub current: f32,
}
```

### 5.3 Enums

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum GoodType {
    Rum,
    Sugar,
    Spices,
    Timber,
    Cloth,
    Weapons,
    // ... more
}

#[derive(Clone, Copy)]
pub enum GoodsTrait {
    Perishable,
    Heavy,
    Illegal,
}

#[derive(Clone, Copy)]
pub enum ShipComponent {
    Sails,
    Rudder,
    Hull,
}

#[derive(Clone, Copy)]
pub enum FactionId {
    NationA,
    NationB,
    NationC,
    Pirates,
}

#[derive(Clone)]
pub enum Order {
    TradeRoute { from: Entity, to: Entity },
    Patrol { area: Rect },
    Escort { target: Entity },
    Scout { area: Rect },
}

#[derive(Clone)]
pub enum IntelType {
    PermanentLand { tiles: Vec<IVec2> },
    ShipRoute { ship: Entity, path: Vec<Vec2>, expiry: u32 },
    StormForecast { area: Rect, start_tick: u32, duration: u32 },
    TreasureRumor { location: Vec2 },
}

#[derive(Clone, Copy)]
pub enum ContractType {
    Transport,
    Explore,
    Escort,
    Hunt,
}

#[derive(Clone, Copy)]
pub enum CompanionRole {
    Quartermaster,
    Navigator,
    Lookout,
    Gunner,
    Mystic, // late-game magic
}
```

---

## 6. Technical Architecture

### 6.1 Engine & Language
- **Engine**: Bevy 0.15 (Rust)
- **Language**: Rust (2021 edition)
- **Build Tool**: Cargo

### 6.2 ECS Pattern
Bevy's native ECS is used throughout:
- **Entities**: Game objects (ships, ports, projectiles).
- **Components**: Data attached to entities.
- **Systems**: Logic that queries and mutates components.
- **Resources**: Global singletons (clock, faction registry).
- **Events**: Pub/Sub for decoupling (e.g., `ShipHitEvent`).

### 6.3 State Management
Use Bevy `States` for game state machine:
```rust
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    Port,
    HighSeas,
    Combat,
    GameOver,
}
```

Systems are gated by state using `.run_if(in_state(GameState::Combat))`.

### 6.4 Schedules
- `Update`: Runs every frame (rendering, input).
- `FixedUpdate`: Runs at fixed timestep (physics, world simulation).

### 6.5 Physics
Use `avian2d` (ECS-native physics, successor to bevy_xpbd):
- Ships are `RigidBody::Dynamic`.
- Land is `Collider` (static).
- Projectiles are `Sensor` (trigger on overlap).
- Currents apply forces via `ExternalForce`.

### 6.6 Input Management
Use `leafwing-input-manager` for action-based input:
```rust
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    Thrust,
    TurnLeft,
    TurnRight,
    Fire,
    Anchor,
    CycleTarget,
}
```

### 6.7 Best Practices

**Change Detection**: Use `Changed<T>` and `Added<T>` filters in queries to avoid unnecessary work:
```rust
fn update_damaged_ships(query: Query<&Health, Changed<Health>>) {
    for health in &query {
        // Only runs when Health component changed
    }
}
```

**Minimal Commands**: Prefer direct component mutation over `Commands` when possible. Use `Commands` only for entity spawn/despawn or adding/removing components.

**Plugin Pattern**: Each game system is encapsulated in a Bevy `Plugin` for modularity and testability.

---

## 7. Plugin Structure

The game is organized into modular plugins:

```
src/
├── main.rs                 # App setup, add plugins
├── lib.rs                  # Re-exports
├── plugins/
│   ├── mod.rs
│   ├── core.rs             # CorePlugin: states, input, camera
│   ├── world_sim.rs        # WorldSimulationPlugin
│   ├── navigation.rs       # NavigationPlugin
│   ├── combat.rs           # CombatPlugin
│   ├── economy.rs          # EconomyPlugin
│   ├── contracts.rs        # ContractPlugin
│   ├── orders.rs           # OrdersPlugin
│   ├── intel.rs            # IntelPlugin
│   ├── companions.rs       # CompanionPlugin
│   ├── progression.rs      # ProgressionPlugin
│   ├── save_load.rs        # SaveLoadPlugin
│   ├── audio.rs            # AudioPlugin
│   └── rendering.rs        # RenderingPlugin
├── components/
│   ├── mod.rs
│   ├── ship.rs
│   ├── cargo.rs
│   ├── health.rs
│   └── ... (one file per component group)
├── systems/
│   ├── mod.rs
│   └── ... (organized by plugin)
├── resources/
│   ├── mod.rs
│   ├── world_clock.rs
│   ├── faction_registry.rs
│   └── meta_profile.rs
├── events/
│   ├── mod.rs
│   ├── combat_events.rs
│   └── world_events.rs
└── utils/
    ├── mod.rs
    ├── spatial_hash.rs
    └── pathfinding.rs
```

---

## 8. Dependencies & Crates

### 8.1 Required Crates

| Crate | Version | Purpose |
|---|---|---|
| `bevy` | `0.15` | Core engine. |
| `avian2d` | `0.2` | ECS-native 2D physics (successor to bevy_xpbd). |
| `leafwing-input-manager` | `0.16` | Action-based input handling. |
| `bevy_ecs_tilemap` | `0.15` | Tilemap rendering for world map. |
| `bevy_egui` | `0.31` | Immediate-mode UI. |
| `bevy_kira_audio` | `0.21` | Advanced audio. |
| `bevy_save` | `0.16` | Save/Load functionality. |
| `steamworks` | `0.11` | Steamworks SDK bindings. |
| `noise` | `0.9` | Procedural generation (maps, wind). |
| `rand` | `0.8` | Random number generation. |
| `serde` | `1.0` | Serialization (for save files). |

### 8.2 Cargo.toml

```toml
[package]
name = "pirates"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = { version = "0.15", features = ["dynamic_linking"] }
avian2d = "0.2"
leafwing-input-manager = "0.16"
bevy_ecs_tilemap = "0.15"
bevy_egui = "0.31"
bevy_kira_audio = "0.21"
bevy_save = "0.16"
steamworks = "0.11"
noise = "0.9"
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3
```

---

## 9. File Structure

```
pirates/
├── Cargo.toml
├── Cargo.lock
├── README.md                    # This file (source of truth)
├── WORK_LOG.md                  # Development log
├── assets/
│   ├── sprites/
│   │   ├── ships/
│   │   ├── ports/
│   │   ├── loot/
│   │   └── ui/
│   ├── tilemaps/
│   │   └── world.tmx
│   ├── audio/
│   │   ├── music/
│   │   ├── ambience/
│   │   └── sfx/
│   ├── fonts/
│   └── shaders/
│       └── ink_parchment.wgsl
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── plugins/
│   ├── components/
│   ├── systems/
│   ├── resources/
│   ├── events/
│   └── utils/
└── saves/
    └── (generated at runtime)
```

---

## 10. Implementation Phases

### Phase 1: Foundations (Est. 2 weeks)
- [ ] Initialize Bevy project with `Cargo.toml`.
- [ ] Implement `GameState` enum and state transitions.
- [ ] Set up 2D camera with pan/zoom.
- [ ] Integrate `bevy_egui` for debug UI.
- [ ] Create placeholder sprites for ships.
- [ ] Implement basic input handling (WASD, mouse click).

### Phase 2: Combat MVP (Est. 2 weeks)
- [ ] Integrate `avian2d` physics.
- [ ] Create `Ship` entity with `RigidBody`, `Health`, `Velocity`.
- [ ] Implement `ShipMovementSystem` (WASD thrust/turn).
- [ ] Implement `CannonSystem` (Spacebar fire, cooldown).
- [ ] Implement `ProjectileSystem` (spawn, move, collision).
- [ ] Implement `DamageSystem` (component damage, debuffs).
- [ ] Implement `LootSystem` (spawn on damage, collect on collision).
- [ ] Implement `CurrentZone` (areas that apply force).
- [ ] Basic enemy AI (chase, fire).
- [ ] Win/lose conditions (all enemies dead / player dead).

### Phase 3: High Seas Map (Est. 2 weeks)
- [ ] Integrate `bevy_ecs_tilemap`.
- [ ] Procedural island/landmass generation with `noise` crate.
- [ ] Implement `FogOfWarSystem` (track explored tiles).
- [ ] Implement `WindSystem` (global wind direction, affects speed).
- [ ] Implement `NavigationSystem` (click to set destination, pathfinding).
- [ ] Implement `EncounterSystem` (proximity triggers combat).
- [ ] Transition between `HighSeasState` and `CombatState`.

### Phase 4: Ports & Economy (Est. 2 weeks)
- [ ] Create `Port` entities with `Inventory`.
- [ ] Implement `MarketSystem` (buy/sell goods).
- [ ] Implement `PriceCalculationSystem` (supply/demand).
- [ ] Implement `GoodsDecaySystem` (perishable items lose value).
- [ ] Build Port UI with `bevy_egui` (Market, Tavern, Docks).
- [ ] Implement `ContractSystem` (generate, track, reward).

### Phase 5: World Simulation & AI (Est. 2 weeks)
- [ ] Implement `WorldSimulationPlugin` (tick-based loop).
- [ ] Faction AI (set trade routes, respond to threats).
- [ ] AI ship pathfinding (A*).
- [ ] `OrdersSystem` for player and AI fleets.
- [ ] Spatial hashing for efficient proximity queries.
- [ ] Stress test with 1000 simulated ships.

### Phase 6: Companions & Intel (Est. 1 week)
- [ ] Implement `Companion` entity and `CompanionPlugin`.
- [ ] Companion abilities (Quartermaster, Navigator, Lookout).
- [ ] Implement `IntelPlugin` (acquire, store, expire, render on map).

### Phase 7: Progression & Persistence (Est. 1 week)
- [ ] Implement `MetaProfile` resource.
- [ ] D&D-style stat progression.
- [ ] Archetype unlocks.
- [ ] Legacy wreckage spawning.
- [ ] Integrate `bevy_save` for save/load.
- [ ] Autosave on state transitions.

### Phase 8: Audio & Polish (Est. 1 week)
- [ ] Integrate `bevy_kira_audio`.
- [ ] Scene-based music (Port, High Seas, Combat).
- [ ] Ambient sounds (waves, wind, creaking).
- [ ] SFX (cannon fire, hit, purchase, UI).
- [ ] Implement "Ink and Parchment" shader.
- [ ] UI polish (parchment textures, scroll animations).

### Phase 9: Steam Integration (Est. 1 week)
- [ ] Integrate `steamworks` crate.
- [ ] Steam initialization on app launch.
- [ ] Achievements (first combat win, first million gold, etc.).
- [ ] Cloud saves.
- [ ] Build and test Steam release.

### Phase 10: Supernatural Shift (Est. 2 weeks)
- [ ] Narrative trigger (success threshold).
- [ ] Late-game enemy faction (Undead/Spirits).
- [ ] Spectral ship reskins.
- [ ] Boss ships (unique AI, immune to capture).
- [ ] Magic abilities (Burn Sails, Freeze Rudder, Invisibility, Wind Manipulation).
- [ ] Magic-granting companions (Mystic role).

---

## Appendix: Events

| Event | Trigger | Handlers |
|---|---|---|
| `ShipHitEvent` | Projectile collides with ship. | `DamageSystem`, `LootSystem`, `AudioSystem`. |
| `ShipDestroyedEvent` | Ship HP (hull) <= 0. | `CombatSystem` (remove entity), `ProgressionSystem` (track stats). |
| `CombatTriggeredEvent` | Player near hostile AI ship. | State transition to `CombatState`. |
| `CombatEndedEvent` | All enemies dead or player fled. | State transition to `HighSeasState`. |
| `PlayerDiedEvent` | Player ship destroyed. | State transition to `GameOverState`, save meta-profile. |
| `IntelAcquiredEvent` | Player gains intel. | `IntelSystem` (add to map), `AudioSystem`. |
| `ContractCompletedEvent` | Contract conditions met. | `ContractSystem` (payout), `ProgressionSystem`. |
| `TradeExecutedEvent` | Player buys/sells goods. | `EconomySystem` (update inventories), `AudioSystem`. |
| `OrderIssuedEvent` | Player/AI issues order to a ship. | `OrdersSystem` (add to queue). |

---

**End of Document**