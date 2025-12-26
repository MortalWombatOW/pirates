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
    * For each major feature or invariant, plan how to test it:
        * **Test Save Name**: `test_<feature_name>`
        * **Setup Conditions**: What game state demonstrates the feature? (e.g., "Ship near coastline with destination set")
        * **Expected Logs**: What `info!()` output proves it works? (e.g., `"Pathfinding: route found with {} waypoints"`)
    * Add verification subtasks to `WORK_PLAN.md`:
        ```
        - [ ] Add info!() logs to prove feature behavior
        - [ ] Create test save: `cargo run -- --save-as test_<feature>`, set up conditions, F5
        - [ ] Verify: `cargo run -- --load test_<feature> 2>&1 | grep "<pattern>"`
        ```
    * **Constraint**: A feature is not complete until its test save exists and verification passes.
    * See `AGENT.md` (Save-Based Feature Verification) for detailed instructions.

5.  **Plan Update**
    * Update `WORK_PLAN.md` with the new tasks.
    * **Rule**: Only `/architect` may add new feature tasks. `/forge` may add subtasks for bug fixes.

6.  **Handoff**
    * Remind the user to run `/forge` on the first new task.