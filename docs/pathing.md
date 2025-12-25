# Pathfinding System

## Overview

The Pirates game uses **bevy_landmass** (v0.8.0) for velocity-based navigation steering. This provides smooth, obstacle-avoiding movement for all ships.

## Architecture

### Core Components

1. **Archipelago2d** - Central navigation manager entity
   - Three archipelagos for different ship size tiers (Small/Medium/Large)
   - Each has different agent radius for shore buffer

2. **Island2dBundle** - Navigation mesh container
   - Contains the validated `NavigationMesh2d`
   - Linked to archipelago via `ArchipelagoRef2d`

3. **Agent2dBundle** - Ship navigation components
   - `Agent` marker component
   - `AgentSettings` (radius, desired_speed, max_speed)
   - `AgentDesiredVelocity2d` - output velocity from landmass

### Shore Buffer Tiers

| Tier | Ships | Buffer (world units) | Agent Radius |
|------|-------|---------------------|--------------|
| Small | Sloop, Raft | 32.0 | 16.0 |
| Medium | Schooner | 64.0 | 32.0 |
| Large | Frigate | 96.0 | 48.0 |

### Navigation Flow

1. **Click/Order** sets `Destination` component
2. **sync_destination_to_agent_target** converts to `AgentTarget2d::Point`
3. **Landmass plugin** calculates steering velocity
4. **Movement systems** apply `AgentDesiredVelocity2d` to Transform
5. **arrival_detection_system** removes components at destination

## Key Files

| File | Purpose |
|------|---------|
| `src/resources/landmass.rs` | LandmassArchipelagos, ShoreBufferTier, mesh building |
| `src/systems/landmass_movement.rs` | Velocity-based movement systems |
| `src/utils/geometry.rs` | NavMesh generation via Delaunay triangulation |
| `src/plugins/worldmap.rs` | Plugin integration, archipelago/island spawning |

## NavMesh Generation

Uses `spade` crate for Delaunay triangulation:

1. Extract coastline polygons from map (CCW winding, land on left)
2. Filter out border polygons (they enclose entire ocean and break collision tests)
3. Offset remaining land polygons outward by shore buffer distance
4. Create Delaunay triangulation of map bounds
5. Filter triangles: keep only those with centroids in water (not inside any land polygon)
6. Convert to `NavigationMesh2d` format for landmass

### Agent Configuration

Archipelagos use tuned `AgentOptions` for smooth navigation:
- `node_sample_distance`: 0.5 * radius (larger = fewer waypoints, smoother paths)
- `obstacle_avoidance_time_horizon`: 0.3 (lower = less "sticky" obstacles)

## Movement Systems

Ships move forward in their facing direction and rotate toward the desired
direction at a rate limited by ship type. This creates realistic boat physics
where larger ships turn more slowly.

### Ship Turn Rates

| Ship Type | Turn Rate (rad/s) | Degrees/sec |
|-----------|-------------------|-------------|
| Sloop | 2.5 | ~143° |
| Raft | 2.0 | ~115° |
| Schooner | 1.5 | ~86° |
| Frigate | 0.8 | ~46° |

### Player Movement (`landmass_player_movement_system`)
- Reads desired direction from `AgentDesiredVelocity2d`
- Rotates toward desired direction at ship-type-limited rate
- Moves forward in **facing** direction (not velocity direction)
- Applies Navigator companion bonus (+25%)
- Applies Navigation stat scaling
- Applies wind effects (±50% based on facing direction)

### AI Movement (`landmass_ai_movement_system`)
- Same turning physics as player
- Uses ship-type-based turn rates
- AI ships move at 50% of base speed

## Path Visualization

Draws dotted line from ship to destination:
- Brown/sepia dots along path
- Red X with circle at destination

## Debug Visualization

Press **F3** to toggle NavMesh debug visualization:
- Blue lines: boundary edges
- Light blue lines: connectivity edges
- Green spheres: agent positions
- Yellow spheres: target positions

## Dependencies

- `bevy_landmass` v0.8.0 (Bevy 0.15 compatible)
- `landmass` v0.7.0 (core navigation library)
- `spade` v2.15 (Delaunay triangulation)
