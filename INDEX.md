# Codebase Index

> **Purpose**: A navigation map to locate relevant code triggers.

## Core Directories

| Path | Responsibility | When to Read/Edit |
| :--- | :--- | :--- |
| `src/main.rs` | App entry point, Plugin registration | Adding new high-level plugins or changing app config. |
| `src/plugins/` | Modular system containers | Registering new systems or resources. |
| `src/components/` | ECS Component definitions | Adding new data fields to entities. |
| `src/systems/` | Game logic implementations | Changing behavior, mechanics, or AI. |
| `src/resources/` | Global state (clocks, maps) | modifying shared data or configs. |

## Key Files

| File | Responsibility | When to Read/Edit |
| :--- | :--- | :--- |
| `src/plugins/core.rs` | GameState, Camera, Window setup | Changing states (`GameState`), camera logic. |
| `src/plugins/main_menu.rs` | Archetype selection UI, TypewriterText | Modifying starting character selection or title animation. |
| `src/plugins/physics.rs` | Avian2D config, Gravity | Tuning global physics settings. |
| `src/plugins/graphics.rs` | PostProcessPlugin, AestheticSettings | Managing shaders, post-processing pipelines. |
| `src/plugins/save.rs` | PersistencePlugin, bevy_save integration | Implementing save/load functionality. |
| `src/resources/meta_profile.rs` | MetaProfile, Archetypes, Unlocks | Changing progression or archetype configs. |
| `src/components/ship.rs` | `Ship`, `Player`, `AI`, `ShipType` | Modifying what defines a ship entity. |
| `src/components/cargo.rs` | `Cargo`, `Gold`, `GoodType` | Changing economy data structures. |
| `src/components/ink_reveal.rs` | `InkReveal` animation component | Fog reveal animation progress tracking. |
| `src/components/typewriter.rs` | `TypewriterText`, `TypewriterRegistry` | UI text write-on effects. |
| `src/systems/movement.rs` | Ship thrust, turn, drag logic | Tuning ship handling or "Keel Effect". |
| `src/systems/combat.rs` | Damage, Projectiles, Health | Balancing combat, hit detection. |
| `src/systems/navigation.rs` | Pathfinding (Theta*), Clicking | Fixing movement bugs or path smoothing. |
| `src/systems/ink_reveal.rs` | `spawn_ink_reveals`, `animate_ink_reveals` | Fog-of-war fade animation. |
| `src/systems/wake_effects.rs` | Ship wake particles, damage splatter | GPU particle effects (bevy_hanabi). |
| `src/resources/map_data.rs` | Tile grid, Navigability checks | Changing how the map is stored/accessed. |
| `src/utils/pathfinding.rs` | A*/Theta* algorithms | Optimizing pathfinding or LOS checks. |

## Assets

| Path | Description |
| :--- | :--- |
| `assets/sprites/` | Ship and UI sprites. |
| `assets/tilemaps/` | World map tilesets. |
| `assets/shaders/ink_parchment.wgsl` | Post-process shader (paper texture, edges, ink effects). |