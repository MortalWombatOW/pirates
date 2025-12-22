---
description: Convert requirements into atomic tasks. Run after /init. Run before /forge.
---

# Workflow: Architect

**Goal**: Convert requirements into atomic tasks.

## Protocol Steps

1.  **Context Loading**
    * Ensure context from `/init` is active.

2.  **Architectural Critique**
    * Analyze the request against `docs/protocol/INVARIANTS.md`.
    * Check for impacts on `FixedUpdate` loops or Event patterns.

3.  **Micro-Planning**
    * Draft a list of tasks.
    * **Constraint**: Each task must be approx. 1 file change or 1 system.

4.  **Plan Update**
    * Update `WORK_PLAN.md` with the new tasks.
    * **Rule**: Only `/architect` may add new feature tasks. `/forge` may add subtasks for bug fixes.

5.  **Handoff**
    * Remind the user to run `/forge` on the first new task.