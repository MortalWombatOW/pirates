---
description: Convert requirements into atomic tasks. Run after /init. Run before /forge.
---

# Workflow: Architect

**Goal**: Convert requirements into atomic tasks.

## Protocol Steps

1.  **Context Loading**
    * Ensure context from `/init` is active. If not, read it from `workflows/init.md`.

2.  **Architectural Critique**
    * Analyze the request against `AGENT.md` (The Law).
    * Check for impacts on `FixedUpdate` loops or Event patterns.

3.  **Micro-Planning**
    * Draft a list of tasks.
    * **Constraint**: Each task must be approx. 1 file change or 1 system.

4.  **Verification Planning (Save-Based Testing)**
    * For each major feature or invariant, plan a corresponding test save:
        * **Save File**: `assets/saves/test_<feature_name>.sav` - A save state that sets up the conditions to demonstrate the feature.
        * **Verification Command**: `cargo run -- --load test_<feature_name>` (or equivalent CLI arg).
        * **Expected Logs**: Document what log output proves the feature works (e.g., `info!("Pathfinding: route found with {} waypoints", n)`).
    * Add a verification subtask to each feature task in `WORK_PLAN.md`:
        * `- [ ] Create test save for <feature>`
        * `- [ ] Verify via: cargo run -- --load test_<feature> | grep "<expected log>"`
    * **Constraint**: A feature is not complete until its test save exists and verification passes.

5.  **Plan Update**
    * Update `WORK_PLAN.md` with the new tasks.
    * **Rule**: Only `/architect` may add new feature tasks. `/forge` may add subtasks for bug fixes.

6.  **Handoff**
    * Remind the user to run `/forge` on the first new task.