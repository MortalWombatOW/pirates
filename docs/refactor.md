# Feature-Based Refactoring Plan

> Based on [Organizing Bevy Projects](https://vladbat00.github.io/blog/001-organising-bevy-projects/)

## Overview

This document outlines a refactoring plan to migrate from the current **layered organization** (plugins/systems/components/resources) to a **feature-based organization** where related code is colocated by game feature.

---

## Current Structure

```
src/
├── plugins/       (21 files) - Orchestration
├── systems/       (20 files) - Logic
├── components/    (22 files) - Data
├── resources/     (16 files) - Shared State
├── utils/         (4 files)  - Helpers
├── events/        (1 file)   - Event Definitions
├── lib.rs
└── main.rs
```

**Problems:**
1. Adding a feature requires touching 3-4 folders
2. No SystemSets - ordering via ad-hoc `.after()/.before()` chains
3. Some plugins are massive (worldmap.rs = 1,565 lines)
4. Cross-cutting concerns are hard to trace

---

## Target Structure

```
src/
├── features/
│   ├── combat/
│   │   ├── mod.rs           (CombatPlugin)
│   │   ├── components.rs    (Projectile, TargetComponent, Loot, etc.)
│   │   ├── systems.rs       (firing, collision, destruction, loot)
│   │   ├── resources.rs     (CannonState, CombatCooldowns)
│   │   ├── events.rs        (CannonFiredEvent, ShipDestroyedEvent)
│   │   └── ai.rs            (Combat AI behavior)
│   │
│   ├── navigation/
│   │   ├── mod.rs           (NavigationPlugin)
│   │   ├── components.rs    (Destination, NavigationPath, Vision)
│   │   ├── systems.rs       (click_to_navigate, pathfinding, arrival)
│   │   ├── resources.rs     (RouteCache, NavMeshResource)
│   │   └── pathfinding.rs   (Theta*, A*, utilities)
│   │
│   ├── worldmap/
│   │   ├── mod.rs           (WorldMapPlugin)
│   │   ├── components.rs    (HighSeasPlayer, HighSeasAI, WorldMapTile)
│   │   ├── systems.rs       (spawn, despawn, visibility)
│   │   ├── resources.rs     (MapData, FogOfWar, CoastlineData)
│   │   ├── tilemap.rs       (tileset, tilemap spawning)
│   │   ├── coastlines.rs    (extraction, rendering)
│   │   ├── fog.rs           (fog of war systems)
│   │   └── encounters.rs    (encounter detection, combat triggers)
│   │
│   ├── port/
│   │   ├── mod.rs           (PortPlugin)
│   │   ├── components.rs    (Port, Inventory, PortName)
│   │   ├── systems.rs       (arrival, departure)
│   │   ├── ui.rs            (market, tavern, docks panels)
│   │   └── trading.rs       (buy/sell logic)
│   │
│   ├── economy/
│   │   ├── mod.rs           (EconomyPlugin)
│   │   ├── components.rs    (Cargo, Gold, Contract)
│   │   ├── systems.rs       (prices, decay, contracts)
│   │   └── resources.rs     (GlobalDemand)
│   │
│   ├── ship/
│   │   ├── mod.rs           (ShipPlugin)
│   │   ├── components.rs    (Ship, Health, ShipType, WaterIntake)
│   │   ├── movement.rs      (thrust, turn, anchor, physics)
│   │   ├── damage.rs        (hit detection, destruction)
│   │   └── repair.rs        (repair systems)
│   │
│   ├── ai/
│   │   ├── mod.rs           (AIPlugin)
│   │   ├── components.rs    (AI, Faction, Order, OrderQueue)
│   │   ├── orders.rs        (TradeRoute, Patrol, Escort, Scout)
│   │   ├── faction.rs       (FactionRegistry, faction AI)
│   │   └── behavior.rs      (pursue, flee, fire decisions)
│   │
│   ├── companions/
│   │   ├── mod.rs           (CompanionPlugin)
│   │   ├── components.rs    (Companion, CompanionRole)
│   │   ├── systems.rs       (recruitment, abilities)
│   │   └── abilities.rs     (Gunner, Navigator, Lookout effects)
│   │
│   ├── progression/
│   │   ├── mod.rs           (ProgressionPlugin)
│   │   ├── resources.rs     (MetaProfile, ArchetypeRegistry)
│   │   ├── archetypes.rs    (unlock conditions, bonuses)
│   │   ├── wrecks.rs        (legacy wreck spawning/exploration)
│   │   └── persistence.rs   (save/load)
│   │
│   └── intel/
│       ├── mod.rs           (IntelPlugin)
│       ├── components.rs    (Intel, IntelType)
│       └── systems.rs       (acquisition, expiry, visualization)
│
├── core/
│   ├── mod.rs               (CorePlugin)
│   ├── state.rs             (GameState enum, transitions)
│   ├── camera.rs            (Camera2d, shake, follow)
│   ├── input.rs             (PlayerAction, input mapping)
│   └── time.rs              (WorldClock, Wind)
│
├── ui/
│   ├── mod.rs               (UiPlugin - aggregates all UI)
│   ├── theme.rs             (colors, fonts, styling)
│   ├── debug.rs             (debug panel, toggles)
│   ├── compass.rs           (CompassRosePlugin)
│   ├── scale_bar.rs         (ScaleBarPlugin)
│   ├── cartouche.rs         (CartouchePlugin)
│   ├── overlay.rs           (shared overlay utilities)
│   └── fade.rs              (FadeControllerPlugin)
│
├── graphics/
│   ├── mod.rs               (GraphicsPlugin)
│   ├── shaders.rs           (ink/parchment post-processing)
│   ├── effects.rs           (wake, splatter, hit flash)
│   └── stippling.rs         (StipplingMaterial)
│
├── shared/
│   ├── mod.rs
│   ├── geometry.rs          (vector math, polygons)
│   ├── spatial_hash.rs      (spatial indexing)
│   └── procgen.rs           (noise, map generation)
│
├── lib.rs
└── main.rs
```

---

## SystemSets

Define explicit system ordering to replace ad-hoc `.after()/.before()`:

```rust
// src/core/sets.rs

#[derive(SystemSet, Clone, Copy, Hash, Debug, Eq, PartialEq)]
pub enum GameSet {
    // Input processing
    Input,

    // Entity spawning/despawning
    Spawn,

    // AI decision making
    AI,

    // Physics and movement
    Physics,

    // Game logic (damage, collection, etc.)
    Logic,

    // State synchronization
    Sync,

    // Visual updates
    Visual,
}

// In CorePlugin::build():
app.configure_sets(
    Update,
    (
        GameSet::Input,
        GameSet::Spawn,
        GameSet::AI,
        GameSet::Physics,
        GameSet::Logic,
        GameSet::Sync,
        GameSet::Visual,
    ).chain(),
);
```

**Usage in feature plugins:**
```rust
// In CombatPlugin
app.add_systems(
    Update,
    (
        cannon_firing_system,
        consume_firing_input,
    )
        .chain()
        .in_set(GameSet::Logic)
        .run_if(in_state(GameState::Combat)),
);
```

---

## Migration Strategy

### Phase 1: Infrastructure (Low Risk)
1. Create `src/core/sets.rs` with SystemSets
2. Create `src/shared/` with utilities
3. Migrate utils/ to shared/

### Phase 2: Extract Small Features (Medium Risk)
4. Extract `features/companions/`
5. Extract `features/intel/`
6. Extract `features/economy/`

### Phase 3: Extract Large Features (Higher Risk)
7. Extract `features/combat/`
8. Extract `features/ship/`
9. Extract `features/ai/`

### Phase 4: Extract Core Features (Highest Risk)
10. Extract `features/worldmap/` (break up 1,565-line file)
11. Extract `features/port/`
12. Extract `features/navigation/`

### Phase 5: Consolidate
13. Extract `features/progression/`
14. Consolidate `ui/` plugins
15. Consolidate `graphics/` plugins
16. Delete empty old directories
17. Update lib.rs exports

---

## File Mapping

| Current Location | New Location |
|-----------------|--------------|
| `plugins/combat.rs` | `features/combat/mod.rs` |
| `systems/combat.rs` | `features/combat/systems.rs` |
| `components/combat.rs` | `features/combat/components.rs` |
| `components/loot.rs` | `features/combat/components.rs` |
| `systems/ai.rs` | `features/combat/ai.rs` |
| | |
| `plugins/worldmap.rs` | `features/worldmap/mod.rs` + split |
| `systems/worldmap.rs` | `features/worldmap/systems.rs` |
| `resources/map_data.rs` | `features/worldmap/resources.rs` |
| `resources/fog_of_war.rs` | `features/worldmap/fog.rs` |
| | |
| `plugins/companion.rs` | `features/companions/mod.rs` |
| `components/companion.rs` | `features/companions/components.rs` |
| `systems/companion.rs` | `features/companions/systems.rs` |
| | |
| `plugins/core.rs` | `core/mod.rs` |
| `plugins/input.rs` | `core/input.rs` |
| `systems/camera.rs` | `core/camera.rs` |
| `systems/world_tick.rs` | `core/time.rs` |
| `resources/wind.rs` | `core/time.rs` |
| | |
| `utils/pathfinding.rs` | `features/navigation/pathfinding.rs` |
| `utils/geometry.rs` | `shared/geometry.rs` |
| `utils/spatial_hash.rs` | `shared/spatial_hash.rs` |
| `utils/procgen.rs` | `shared/procgen.rs` |

---

## Benefits

1. **Locality**: All combat code in `features/combat/`
2. **Discoverability**: Easier to find related code
3. **Modularity**: Features can be enabled/disabled
4. **Team scaling**: Clear ownership boundaries
5. **Testing**: Feature-scoped integration tests
6. **SystemSets**: Explicit ordering, fewer bugs

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Breaking changes | Run `cargo check` after each file move |
| Import chaos | Update lib.rs exports incrementally |
| Lost functionality | Keep old code until new works |
| Merge conflicts | Do refactor in dedicated branch |

---

## Success Criteria

- [ ] All features in `features/` directory
- [ ] SystemSets defined and used throughout
- [ ] No file > 500 lines
- [ ] `cargo check` passes with zero warnings
- [ ] All tests pass
- [ ] Game runs correctly
