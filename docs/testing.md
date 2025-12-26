# Save-Based Feature Testing Plan

> **Purpose**: This document defines test saves and verification commands for all major features. Each test save captures a game state that exercises a specific feature, with log output proving correct behavior.

---

## Quick Reference

```bash
# Create a test save
cargo run -- --save-as test_<feature>
# (set up conditions in-game, press F5, exit)

# Verify a feature
cargo run -- --load test_<feature> 2>&1 | grep "<pattern>"
```

**Save Location**: `~/Library/Application Support/pirates/` (macOS)

---

## 1. Navigation & Pathfinding

### test_pathfinding_open_water
**Purpose**: Verify Theta* pathfinding finds direct routes across open water.

**Setup Conditions**:
1. Start new game, sail to open ocean (no nearby islands)
2. Click destination 200+ tiles away across open water
3. Save immediately after path is calculated

**Required Logs**:
```rust
info!("Pathfinding: route found with {} waypoints", waypoints.len());
info!("Pathfinding: path length {} tiles", path_length);
```

**Verification**:
```bash
cargo run -- --load test_pathfinding_open_water 2>&1 | grep "route found"
# Expected: "Pathfinding: route found with 2-5 waypoints" (direct path = few waypoints)
```

---

### test_pathfinding_obstacle
**Purpose**: Verify pathfinding routes around land obstacles.

**Setup Conditions**:
1. Position ship on one side of an island
2. Click destination on opposite side (path must go around)
3. Save after path is calculated

**Required Logs**:
```rust
info!("Pathfinding: routing around obstacle, {} waypoints", waypoints.len());
```

**Verification**:
```bash
cargo run -- --load test_pathfinding_obstacle 2>&1 | grep "routing around"
# Expected: Path has 5+ waypoints showing obstacle avoidance
```

---

### test_navmesh_ship_size
**Purpose**: Verify NavMesh respects ship size tiers (Sloop fits, Frigate doesn't).

**Setup Conditions**:
1. Find a narrow channel between islands
2. With Sloop: navigate through channel successfully
3. Save at channel entrance

**Required Logs**:
```rust
info!("NavMesh: ship tier {:?}, using archipelago {}", tier, archipelago_id);
info!("NavMesh: path valid through narrow passage");
// OR for larger ships:
info!("NavMesh: passage too narrow for ship tier {:?}", tier);
```

**Verification**:
```bash
cargo run -- --load test_navmesh_ship_size 2>&1 | grep "ship tier"
```

---

### test_coastline_avoidance
**Purpose**: Verify ships are pushed away from coastlines.

**Setup Conditions**:
1. Sail ship close to coastline (within 30 units)
2. Set destination that would clip through land
3. Save while ship is near coast

**Required Logs**:
```rust
info!("Coastline: within {} units, applying avoidance force", distance);
info!("Coastline: push direction ({:.2}, {:.2})", push.x, push.y);
```

**Verification**:
```bash
cargo run -- --load test_coastline_avoidance 2>&1 | grep "applying avoidance"
# Expected: Avoidance force is applied, ship doesn't beach
```

---

### test_click_navigation
**Purpose**: Verify click-to-navigate converts screen coords correctly.

**Setup Conditions**:
1. Zoom camera to non-default level
2. Pan camera off-center
3. Click to set destination
4. Save after destination is set

**Required Logs**:
```rust
info!("Navigation: click at screen ({}, {})", screen_x, screen_y);
info!("Navigation: destination set to world ({:.0}, {:.0})", world_x, world_y);
info!("Navigation: destination tile ({}, {})", tile_x, tile_y);
```

**Verification**:
```bash
cargo run -- --load test_click_navigation 2>&1 | grep "destination set"
```

---

## 2. Combat System

### test_cannon_firing
**Purpose**: Verify broadside cannon mechanics.

**Setup Conditions**:
1. Enter combat state with enemy nearby
2. Position ship broadside to enemy
3. Fire port cannon (Q)
4. Save immediately after firing

**Required Logs**:
```rust
info!("Combat: firing port broadside, {} projectiles", count);
info!("Combat: projectile spawned at ({:.0}, {:.0}) with velocity ({:.0}, {:.0})", x, y, vx, vy);
```

**Verification**:
```bash
cargo run -- --load test_cannon_firing 2>&1 | grep "firing.*broadside"
# Expected: 3 projectiles spawned with correct velocities
```

---

### test_cannon_cooldown
**Purpose**: Verify cannon cooldown is enforced.

**Setup Conditions**:
1. Fire cannon
2. Attempt to fire again immediately
3. Save during cooldown period

**Required Logs**:
```rust
info!("Combat: cannon on cooldown, {:.1}s remaining", remaining);
info!("Combat: cannon ready to fire");
```

**Verification**:
```bash
cargo run -- --load test_cannon_cooldown 2>&1 | grep "cooldown"
```

---

### test_projectile_collision
**Purpose**: Verify projectile-ship collision applies damage.

**Setup Conditions**:
1. Fire at enemy ship
2. Confirm hit registered
3. Save after hit

**Required Logs**:
```rust
info!("Combat: projectile hit {:?}, damage {} to {:?}", target, damage, component);
info!("Combat: {} health now {:.0}/{:.0}", component_name, current, max);
```

**Verification**:
```bash
cargo run -- --load test_projectile_collision 2>&1 | grep "projectile hit"
```

---

### test_ship_destruction
**Purpose**: Verify ship destruction triggers correctly at 0 hull HP.

**Setup Conditions**:
1. Damage enemy ship to near-zero hull
2. Final shot destroys ship
3. Save immediately after destruction

**Required Logs**:
```rust
info!("Combat: ship {:?} destroyed", entity);
info!("Combat: loot dropped at ({:.0}, {:.0})", x, y);
```

**Verification**:
```bash
cargo run -- --load test_ship_destruction 2>&1 | grep "ship.*destroyed"
```

---

### test_loot_collection
**Purpose**: Verify loot pickup adds to player cargo/gold.

**Setup Conditions**:
1. Destroy enemy ship (loot spawns)
2. Sail through loot
3. Save after collection

**Required Logs**:
```rust
info!("Loot: collected {} gold", gold_amount);
info!("Loot: collected {} units of {:?}", quantity, good_type);
info!("Loot: player gold now {}", new_total);
```

**Verification**:
```bash
cargo run -- --load test_loot_collection 2>&1 | grep "Loot: collected"
```

---

### test_water_intake
**Purpose**: Verify hull damage causes water intake leading to sinking.

**Setup Conditions**:
1. Take significant hull damage
2. Observe water accumulating
3. Save while taking on water

**Required Logs**:
```rust
info!("Damage: hull hit, adding water intake at rate {:.3}", rate);
info!("Sinking: water level {:.0}/{:.0}", current_water, max_capacity);
```

**Verification**:
```bash
cargo run -- --load test_water_intake 2>&1 | grep "water intake"
```

---

### test_hit_flash
**Purpose**: Verify visual hit feedback on damage.

**Setup Conditions**:
1. Get hit by enemy projectile
2. Save during flash animation

**Required Logs**:
```rust
info!("Visual: hit flash triggered on {:?}", entity);
info!("Visual: flash lerp progress {:.2}", progress);
```

**Verification**:
```bash
cargo run -- --load test_hit_flash 2>&1 | grep "hit flash"
```

---

## 3. Ship Movement & Physics

### test_ship_thrust
**Purpose**: Verify W key applies forward thrust.

**Setup Conditions**:
1. Start in HighSeas with ship stationary
2. Hold W to accelerate
3. Save while moving forward

**Required Logs**:
```rust
info!("Physics: applying thrust {:.0} in direction ({:.2}, {:.2})", force, dir.x, dir.y);
info!("Physics: ship velocity ({:.0}, {:.0}), speed {:.0}", vx, vy, speed);
```

**Verification**:
```bash
cargo run -- --load test_ship_thrust 2>&1 | grep "applying thrust"
```

---

### test_ship_turning
**Purpose**: Verify A/D keys apply torque for rotation.

**Setup Conditions**:
1. Hold A or D to turn
2. Save while turning

**Required Logs**:
```rust
info!("Physics: applying torque {:.0}", torque);
info!("Physics: angular velocity {:.2} rad/s", angular_vel);
```

**Verification**:
```bash
cargo run -- --load test_ship_turning 2>&1 | grep "applying torque"
```

---

### test_anchor
**Purpose**: Verify anchor (Shift) stops movement but allows rotation.

**Setup Conditions**:
1. Build up speed
2. Press Shift to anchor
3. Save while anchored

**Required Logs**:
```rust
info!("Physics: anchor engaged, zeroing velocity");
info!("Physics: anchored, velocity = ({:.2}, {:.2})", vx, vy);
```

**Verification**:
```bash
cargo run -- --load test_anchor 2>&1 | grep "anchor engaged"
```

---

### test_wind_effect
**Purpose**: Verify wind affects ship speed.

**Setup Conditions**:
1. Note wind direction
2. Sail with wind, observe speed boost
3. Sail against wind, observe speed reduction
4. Save while sailing

**Required Logs**:
```rust
info!("Wind: direction ({:.2}, {:.2}), strength {:.2}", wx, wy, strength);
info!("Wind: alignment factor {:.2}, speed multiplier {:.2}", alignment, multiplier);
```

**Verification**:
```bash
cargo run -- --load test_wind_effect 2>&1 | grep "Wind:.*multiplier"
```

---

## 4. Combat AI

### test_ai_circling
**Purpose**: Verify AI maintains broadside circling behavior.

**Setup Conditions**:
1. Engage enemy in combat
2. Let AI position for broadside
3. Save during circling

**Required Logs**:
```rust
info!("AI: state Circling, distance {:.0}, angle {:.2} rad", distance, angle);
info!("AI: adjusting to optimal range {:.0}", target_range);
```

**Verification**:
```bash
cargo run -- --load test_ai_circling 2>&1 | grep "AI: state Circling"
```

---

### test_ai_firing
**Purpose**: Verify AI fires when conditions are met.

**Setup Conditions**:
1. Let AI get into firing position
2. AI fires broadside
3. Save after AI fires

**Required Logs**:
```rust
info!("AI: firing conditions met - range {:.0}, angle {:.2}", range, angle);
info!("AI: broadside fired at player");
```

**Verification**:
```bash
cargo run -- --load test_ai_firing 2>&1 | grep "AI: firing conditions"
```

---

### test_ai_flee
**Purpose**: Verify AI flees when health is low.

**Setup Conditions**:
1. Damage enemy to <20% hull
2. Observe flee behavior
3. Save while AI is fleeing

**Required Logs**:
```rust
info!("AI: health critical ({:.0}%), switching to Flee", health_percent);
info!("AI: state Fleeing, retreating from player");
```

**Verification**:
```bash
cargo run -- --load test_ai_flee 2>&1 | grep "switching to Flee"
```

---

## 5. World & Environment

### test_fog_of_war
**Purpose**: Verify fog reveals as player explores.

**Setup Conditions**:
1. Start new game (most map is fog)
2. Sail to reveal new area
3. Save after revealing tiles

**Required Logs**:
```rust
info!("FogOfWar: revealed {} new tiles", count);
info!("FogOfWar: total explored {}/{} tiles", explored, total);
```

**Verification**:
```bash
cargo run -- --load test_fog_of_war 2>&1 | grep "revealed.*new tiles"
```

---

### test_vision_radius
**Purpose**: Verify Lookout companion increases vision.

**Setup Conditions**:
1. Recruit Lookout at tavern
2. Observe increased vision radius
3. Save with Lookout in crew

**Required Logs**:
```rust
info!("Vision: base radius {}, Lookout bonus +50%, effective radius {}", base, effective);
```

**Verification**:
```bash
cargo run -- --load test_vision_radius 2>&1 | grep "Lookout bonus"
```

---

### test_world_clock
**Purpose**: Verify time advances correctly.

**Setup Conditions**:
1. Note starting time
2. Wait for time to advance
3. Save after time change

**Required Logs**:
```rust
info!("WorldClock: Day {}, Hour {}, Tick {}", day, hour, tick);
info!("WorldClock: hour boundary crossed, now Day {} Hour {}", day, hour);
```

**Verification**:
```bash
cargo run -- --load test_world_clock 2>&1 | grep "WorldClock:"
```

---

### test_port_arrival
**Purpose**: Verify entering port triggers state transition.

**Setup Conditions**:
1. Sail toward a port
2. Enter port tile
3. Save happens automatically on Port entry

**Required Logs**:
```rust
info!("Port: arrived at '{}'", port_name);
info!("State: transitioning to Port");
```

**Verification**:
```bash
cargo run -- --load test_port_arrival 2>&1 | grep "arrived at"
```

---

## 6. Economy & Trading

### test_buy_goods
**Purpose**: Verify buying goods reduces gold, increases cargo.

**Setup Conditions**:
1. Enter port with gold
2. Buy goods from market
3. Save after purchase

**Required Logs**:
```rust
info!("Trade: bought {} units of {:?} for {} gold", quantity, good, cost);
info!("Trade: player gold {} -> {}", old_gold, new_gold);
info!("Trade: cargo now {}/{}", current, capacity);
```

**Verification**:
```bash
cargo run -- --load test_buy_goods 2>&1 | grep "Trade: bought"
```

---

### test_sell_goods
**Purpose**: Verify selling goods increases gold, reduces cargo.

**Setup Conditions**:
1. Enter port with cargo
2. Sell goods at market
3. Save after sale

**Required Logs**:
```rust
info!("Trade: sold {} units of {:?} for {} gold", quantity, good, revenue);
info!("Trade: player gold {} -> {}", old_gold, new_gold);
```

**Verification**:
```bash
cargo run -- --load test_sell_goods 2>&1 | grep "Trade: sold"
```

---

### test_price_dynamics
**Purpose**: Verify prices change based on supply/demand.

**Setup Conditions**:
1. Buy large quantity to reduce stock
2. Check price increase
3. Save after price change

**Required Logs**:
```rust
info!("Economy: {:?} price updated, supply={}, demand={}, price={}", good, supply, demand, price);
info!("Economy: price multiplier {:.2} (supply {:.2}, demand {:.2})", mult, supply_mult, demand_mult);
```

**Verification**:
```bash
cargo run -- --load test_price_dynamics 2>&1 | grep "price updated"
```

---

### test_ship_repair
**Purpose**: Verify repair costs gold and restores health.

**Setup Conditions**:
1. Take damage to ship
2. Enter port
3. Repair at docks
4. Save after repair

**Required Logs**:
```rust
info!("Repair: {} restored {:.0} HP for {} gold", component, hp_restored, cost);
info!("Repair: {} health now {:.0}/{:.0}", component, current, max);
```

**Verification**:
```bash
cargo run -- --load test_ship_repair 2>&1 | grep "Repair:.*restored"
```

---

### test_contract_accept
**Purpose**: Verify accepting contract adds to active list.

**Setup Conditions**:
1. Enter port with contracts available
2. Accept a contract
3. Save after acceptance

**Required Logs**:
```rust
info!("Contract: accepted '{}'", contract_name);
info!("Contract: objective - {:?}", objective);
info!("Contract: reward {} gold, expires Day {}", reward, expiry_day);
```

**Verification**:
```bash
cargo run -- --load test_contract_accept 2>&1 | grep "Contract: accepted"
```

---

### test_contract_complete
**Purpose**: Verify completing contract pays reward.

**Setup Conditions**:
1. Accept transport contract
2. Deliver goods to destination
3. Save after completion

**Required Logs**:
```rust
info!("Contract: '{}' completed!", contract_name);
info!("Contract: reward {} gold paid", reward);
info!("Contract: player gold now {}", new_total);
```

**Verification**:
```bash
cargo run -- --load test_contract_complete 2>&1 | grep "Contract:.*completed"
```

---

## 7. Progression & Persistence

### test_meta_profile_save
**Purpose**: Verify profile saves on game over.

**Setup Conditions**:
1. Play until death (or use debug)
2. Confirm profile updated
3. Check profile file

**Required Logs**:
```rust
info!("Profile: saving to disk...");
info!("Profile: lifetime gold {}, deaths {}, runs {}", gold, deaths, runs);
info!("Profile: saved successfully");
```

**Verification**:
```bash
cargo run -- --load test_meta_profile_save 2>&1 | grep "Profile: saved"
```

---

### test_archetype_unlock
**Purpose**: Verify archetypes unlock based on conditions.

**Setup Conditions**:
1. Meet unlock condition (e.g., 5 runs for RoyalNavyCaptain)
2. Return to main menu
3. Verify archetype is selectable

**Required Logs**:
```rust
info!("Archetype: checking unlock conditions...");
info!("Archetype: {:?} unlocked! (condition: {})", archetype, condition);
```

**Verification**:
```bash
cargo run -- --load test_archetype_unlock 2>&1 | grep "Archetype:.*unlocked"
```

---

### test_legacy_wreck
**Purpose**: Verify death creates explorable wreck.

**Setup Conditions**:
1. Die with gold/cargo
2. Start new game
3. Find wreck at death location
4. Save near wreck

**Required Logs**:
```rust
info!("Wreck: spawned legacy wreck at ({:.0}, {:.0})", x, y);
info!("Wreck: contains {} gold, {} cargo items", gold, cargo_count);
```

**Verification**:
```bash
cargo run -- --load test_legacy_wreck 2>&1 | grep "Wreck: spawned"
```

---

### test_save_load
**Purpose**: Verify save/load preserves game state.

**Setup Conditions**:
1. Play to specific state (gold, position, health)
2. Save with F5
3. Load with F9
4. Verify state matches

**Required Logs**:
```rust
info!("Save: game saved to '{}'", save_name);
info!("Load: game loaded from '{}'", save_name);
info!("Load: restored {} entities", entity_count);
```

**Verification**:
```bash
cargo run -- --load test_save_load 2>&1 | grep "Load: restored"
```

---

## 8. Companions

### test_companion_recruit
**Purpose**: Verify recruiting companion costs gold and adds to crew.

**Setup Conditions**:
1. Enter port tavern
2. Recruit a companion
3. Save after recruitment

**Required Logs**:
```rust
info!("Companion: recruited {:?} for {} gold", role, cost);
info!("Companion: crew size now {}", crew_count);
```

**Verification**:
```bash
cargo run -- --load test_companion_recruit 2>&1 | grep "Companion: recruited"
```

---

### test_gunner_cooldown
**Purpose**: Verify Gunner reduces cannon cooldown by 30%.

**Setup Conditions**:
1. Recruit Gunner
2. Enter combat
3. Fire cannon, observe cooldown
4. Save after firing

**Required Logs**:
```rust
info!("Combat: cannon cooldown {:.2}s (Gunner: -30%)", cooldown);
```

**Verification**:
```bash
cargo run -- --load test_gunner_cooldown 2>&1 | grep "Gunner: -30%"
```

---

### test_navigator_speed
**Purpose**: Verify Navigator increases navigation speed by 25%.

**Setup Conditions**:
1. Recruit Navigator
2. Set navigation destination
3. Observe speed bonus
4. Save while navigating

**Required Logs**:
```rust
info!("Navigation: speed multiplier {:.2} (Navigator: +25%)", multiplier);
```

**Verification**:
```bash
cargo run -- --load test_navigator_speed 2>&1 | grep "Navigator: +25%"
```

---

## 9. Faction AI

### test_faction_hourly
**Purpose**: Verify faction AI runs each hour.

**Setup Conditions**:
1. Wait for hour boundary
2. Observe faction updates
3. Save after faction tick

**Required Logs**:
```rust
info!("FactionAI: hourly tick at Day {} Hour {}", day, hour);
info!("FactionAI: {:?} earned {} gold from {} trade routes", faction, gold, routes);
```

**Verification**:
```bash
cargo run -- --load test_faction_hourly 2>&1 | grep "FactionAI: hourly tick"
```

---

### test_faction_reputation
**Purpose**: Verify reputation affects faction hostility.

**Setup Conditions**:
1. Attack faction ship (reputation decreases)
2. Observe faction turns hostile
3. Save after hostility change

**Required Logs**:
```rust
info!("Faction: {:?} reputation changed to {}", faction, new_rep);
info!("Faction: {:?} now hostile (reputation < 0)", faction);
```

**Verification**:
```bash
cargo run -- --load test_faction_reputation 2>&1 | grep "now hostile"
```

---

## 10. State Transitions

### test_state_main_menu
**Purpose**: Verify game starts in MainMenu state.

**Setup Conditions**:
1. Fresh game start
2. Save immediately (use debug)

**Required Logs**:
```rust
info!("State: entered MainMenu");
info!("State: profile loaded, {} archetypes unlocked", count);
```

**Verification**:
```bash
cargo run 2>&1 | grep "State: entered MainMenu"
```

---

### test_state_combat
**Purpose**: Verify combat state triggers correctly.

**Setup Conditions**:
1. Approach hostile ship
2. Enter combat range
3. Save after combat starts

**Required Logs**:
```rust
info!("State: transitioning MainMenu -> HighSeas");
info!("State: transitioning HighSeas -> Combat");
info!("Combat: initiated with {} enemies", enemy_count);
```

**Verification**:
```bash
cargo run -- --load test_state_combat 2>&1 | grep "transitioning.*Combat"
```

---

### test_state_gameover
**Purpose**: Verify death triggers GameOver state.

**Setup Conditions**:
1. Die in combat (hull = 0)
2. Save triggers automatically

**Required Logs**:
```rust
info!("State: player died, transitioning to GameOver");
info!("GameOver: creating legacy wreck");
info!("GameOver: saving profile");
```

**Verification**:
```bash
cargo run -- --load test_state_gameover 2>&1 | grep "transitioning to GameOver"
```

---

## Test Execution Checklist

### Phase 1: Core Systems (Must Pass)
- [ ] test_pathfinding_open_water
- [ ] test_coastline_avoidance
- [ ] test_ship_thrust
- [ ] test_cannon_firing
- [ ] test_projectile_collision
- [ ] test_save_load

### Phase 2: Gameplay Loop
- [ ] test_click_navigation
- [ ] test_pathfinding_obstacle
- [ ] test_ai_circling
- [ ] test_port_arrival
- [ ] test_buy_goods
- [ ] test_sell_goods

### Phase 3: Progression
- [ ] test_meta_profile_save
- [ ] test_legacy_wreck
- [ ] test_contract_accept
- [ ] test_contract_complete

### Phase 4: Polish
- [ ] test_fog_of_war
- [ ] test_wind_effect
- [ ] test_companion_recruit
- [ ] test_gunner_cooldown
- [ ] test_hit_flash

---

## Adding New Tests

When implementing a new feature, add a test entry to this document:

1. **Choose a descriptive name**: `test_<feature>_<scenario>`
2. **Document setup conditions**: What game state is needed?
3. **Add required logs**: What `info!()` output proves it works?
4. **Write verification command**: How to check the logs?
5. **Add to checklist**: Which phase does it belong to?

Example:
```markdown
### test_new_feature
**Purpose**: Verify new feature works correctly.

**Setup Conditions**:
1. [Steps to create test state]

**Required Logs**:
\`\`\`rust
info!("Feature: doing the thing with value {}", value);
\`\`\`

**Verification**:
\`\`\`bash
cargo run -- --load test_new_feature 2>&1 | grep "Feature: doing"
\`\`\`
```
