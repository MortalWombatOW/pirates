# Future Plans
Not to be implemented currently. Must be moved to WORK_PLAN.md when ready.


### Epic 8.2: Audio Integration
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [ ] 8.2.1 | Integrate `bevy_kira_audio` | 1.1.2 | Plugin added, no errors. |
| [ ] 8.2.2 | Create `AudioPlugin` | 8.2.1 | Manages music and SFX. |
| [ ] 8.2.3 | Implement scene-based music | 8.2.2, 1.2.2 | Different tracks for Port, High Seas, Combat. |
| [ ] 8.2.4 | Add placeholder music tracks | 8.2.3 | MP3/OGG files in `assets/audio/music/`. |
| [ ] 8.2.5 | Implement ambient sounds | 8.2.2 | Layered loops (waves, wind). |
| [ ] 8.2.6 | Implement SFX triggers | 8.2.2 | Cannon fire, hit, purchase, UI click. |
| [ ] 8.2.7 | Add placeholder SFX files | 8.2.6 | Files in `assets/audio/sfx/`. |

## Phase 9: Steam Integration
### Epic 9.1: Steamworks
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [ ] 9.1.1 | Integrate `steamworks` crate | 1.1.2 | Crate added, compiles. |
| [ ] 9.1.2 | Initialize Steam on app launch | 9.1.1 | Steam overlay available. |
| [ ] 9.1.3 | Define achievements | 9.1.2 | List in Steam partner site. |
| [ ] 9.1.4 | Implement achievement unlocking | 9.1.3 | Trigger on events (first win, etc.). |
| [ ] 9.1.5 | Implement cloud saves | 9.1.2, 7.4.2 | Saves sync via Steam Cloud. |
| [ ] 9.1.6 | Build and test Steam release | All above | Runs as Steam game. |

---

## Phase 10: Supernatural Shift
### Epic 10.1: Narrative Trigger
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [ ] 10.1.1 | Define success threshold | 7.1.1 | Gold earned, ships sunk, etc. |
| [ ] 10.1.2 | Create `SupernaturalShiftEvent` | 10.1.1 | Emitted when threshold reached. |
| [ ] 10.1.3 | Display narrative reveal | 10.1.2 | Cutscene or dialog. |

### Epic 10.2: Supernatural Enemies
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [ ] 10.2.1 | Define `Supernatural` faction | 2.8.2 | New `FactionId`. |
| [ ] 10.2.2 | Create undead ship sprites | 2.8.4 | Ghostly/skeletal ship variants. |
| [ ] 10.2.3 | Spawn supernatural ships post-shift | 10.1.2, 10.2.1, 10.2.2 | Undead ships appear on map. |
| [ ] 10.2.4 | Implement boss ships | 10.2.3 | Unique AI, high HP, special attacks. |
| [ ] 10.2.5 | Boss ships cannot be captured | 10.2.4 | Immune to boarding/capture. |

### Epic 10.3: Magic Abilities
| ID | Task | Dependencies | Acceptance Criteria |
|---|---|---|---|
| [ ] 10.3.1 | Define `MagicAbility` enum | 1.1.3 | `BurnSails`, `FreezeRudder`, `Invisibility`, `WindManipulation`. |
| [ ] 10.3.2 | Create `MagicSystem` | 10.3.1 | Handles ability activation and cooldowns. |
| [ ] 10.3.3 | Implement `BurnSails` | 10.3.2 | DoT on enemy sails. |
| [ ] 10.3.4 | Implement `FreezeRudder` | 10.3.2 | Lock enemy turn rate. |
| [ ] 10.3.5 | Implement `Invisibility` | 10.3.2 | Player ship hidden from AI. |
| [ ] 10.3.6 | Implement `WindManipulation` | 10.3.2, 3.4.1 | Change local wind direction. |
| [ ] 10.3.7 | Magic granted via Mystic companion | 10.3.2, 6.2.2 | Mystic enables magic abilities. |
| [ ] 10.3.8 | Magic granted via artifacts | 10.3.2 | Find artifacts in supernatural wrecks. |
