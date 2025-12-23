# Pirates (Working Title)

> A old-timey naval roguelike inspired by flash games and Our Flag Means Death built with Rust and Bevy.


## 1. Project Overview

### The Hook
Reminicient of old flash games, built on a modern, high-performance Entity-Component-System (ECS) architecture.

### The Vision
"Pirates" is a generative open-world naval roguelike. The player captains a single ship in a living, simulated archipelago. The goal is to build notoriety through plunder, trade, or naval dominance, eventually triggering a world-altering "Supernatural Shift" leading to an endgame against ghostly armadas.

The game prioritizes deep simulation over scripted narratives. Factions, economies, and weather systems interact deterministically to create emergent gameplay stories.

### The Vibe
* **Visuals:** Hand-drawn maps feel. UI looks like ink on parchment.
* **Audio:** creaking wood, snapping sails, roaring cannons, and a dynamic sea shanty soundtrack that shifts based on the current state (sailing, combat, port).
* **Game Feel:** Weighty naval combat. Ships have inertia; turning takes time; wind direction matters. 

## 2. Core Pillars

1.  **Deep Naval Combat:** Combat is about positioning, wind management, and broadside timing. Damage is granular (sails, rudder, hull, crew), not just a single HP bar.
2.  **Living World Simulation:** A procedurally generated archipelago with dynamic factions, fluctuating economies, and a day/night cycle. NPC ships have missions (patrol, trade, hunt) that they pursue actively.
3.  **Roguelike Progression:** Permadeath is real, but meta-progression (unlocking new captains, starting bonuses, and "legacy" wrecks from previous runs) softens the blow.
4.  **Deterministic Foundation:** The simulation runs on a fixed timestep. Given the same seed and inputs, the world evolves identically. This is crucial for debugging, replayability, and potential future multiplayer capabilities.

## 3. The Gameplay Loop

1.  **Prepare (Port):** Dock at neutral or friendly ports. Sell loot, buy cargo low to sell high elsewhere, repair damage, recruit crew, and accept contracts (bounties, deliveries, escorts).
2.  **Traverse (High Seas):** Navigate the open ocean using wind and charts. Manage crew morale and supplies. Encounter procedural events: storms, shipwrecks, floating loot, or other ships.
3.  **Engage (Combat):** Intercept merchant vessels for plunder or defend against pirate hunters and rival factions. Use tactical maneuvering to bring broadsides to bear while minimizing exposure. Board disabled ships for high-value loot.
4.  **Repeat & Evolve:** survive long enough, and your actions trigger the endgame "Supernatural Shift," introducing new enemy types and magic mechanics.

## 4. Key Features

### 4.1 Combat System
* **Broadsides:** Cannons are side-mounted. Players must turn their ship broadside to fire effectively.
* **Granular Damage:**
    * **Sails:** Hits reduce max speed and acceleration.
    * **Rudder:** Hits reduce turn rate.
    * **Hull:** Hits cause leaks (damage over time). If hull reaches zero, the ship sinks.
* **Ammo Types:** Chain shot (sails), grape shot (crew), round shot (hull).
* **Boarding:** Disable a ship's movement to initiate a crew-vs-crew auto-battler minigame for capture.

### 4.2 Navigation & Weather
* **Wind System:** Global wind direction changes slowly over time. Sailing into the wind is slow; sailing with it is fast. Tackling is necessary.
* **Fog of War:** The map is revealed as you explore. Buying charts in ports reveals distant areas.
* **Currents & Storms:** Environmental hazards that push ships or deal passive damage.

### 4.3 Economy & Factions
* **Dynamic Pricing:** Ports have supply and demand based on their biome and recent trade. Flooding a market with sugar will crash the price.
* **Reputation:** Actions (sinking ships, completing contracts) affect faction standing. Become a dreaded pirate, a privateer for the Crown, or a wealthy neutral merchant.

### 4.4 The World
* **Procedural Archipelago:** Islands, ports, and biomes (tropical, volcanic, rocky) generated via noise algorithms.
* **The "Supernatural Shift":** A mid-to-late game global event state change. The seas turn darker, ghost ships spawn, and players can acquire eldritch artifacts enabling magical abilities (e.g., summoning fog, momentary invulnerability).

## 5. Art Style Reference
* **Inspiration:** *Sea of Thieves* (vibes), *FTL* (readability), classic 16-bit RPGs (world maps).
* **Perspective:** Top-down 2D.
* **UI:** Diegetic where possible. Health is shown by ship visual damage; wind is a compass rose; inventory looks like a cargo hold manifest.

## 6. Tech Stack & Architecture

* **Engine:** Bevy (Latest stable Rust)
* **Architecture:** Pure ECS. Data-driven design.
* **Physics:** `avian2d` (Deterministic 2D physics)
* **Tilemaps:** `bevy_ecs_tilemap`
* **UI:** `bevy_egui` (for debug/tools) + Bevy UI (for in-game interface)
* **Input:** `leafwing-input-manager`

---

## 7. Developer Guide & Documentation

**This README is the high-level Game Design Document (GDD) and source of truth for the game's vision.**

For technical development, architectural constraints, and agent protocols, refer to the `docs/protocol/` directory.

### Key Documentation Files:

* **`docs/protocol/MANIFESTO.md` (The Soul):** The agent's core persona and command protocols.
* **`docs/protocol/INVARIANTS.md` (The Law):** Hard technical constraints, architectural rules, and "invisible knowledge" that must not be broken. **Read this before writing code.**
* **`docs/protocol/INDEX.md` (The Map):** A guide to the codebase structure and file responsibilities.
* **`WORK_PLAN.md` (The Queue):** The active task list for the current development phase.

### Setup & Running

1.  Install Rust (stable).
2.  Clone repository.
3.  Run: `cargo run --features bevy/dynamic_linking` (dynamic linking speeds up compile times during dev).

### Directory Structure Overview

A detailed index is available in `INDEX.md`.
