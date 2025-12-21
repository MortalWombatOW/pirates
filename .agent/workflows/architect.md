---
description: 
---

# Workflow: Architect

**Goal**: Convert a vague requirement into a set of atomic, technically sound tasks.

## Protocol Steps

1.  **Context Loading**
    * Read `README.md` (Game Design Document).
    * Read `docs/protocol/INVARIANTS.md` (Technical Constraints).
    * Read `WORK_PLAN.md` (Current State).

2.  **Architectural Critique**
    * Analyze the request against the Invariants.
    * *Self-Correction*: Ask, "Does this feature require new Components? Does it impact the `FixedUpdate` physics loop? Does it conflict with existing Event patterns?"

3.  **Micro-Planning**
    * Draft a list of tasks.
    * **Constraint**: Each task must be small enough to complete in one `/forge` session (approx. 1 file change or 1 system implementation).

4.  **Update**
    * Present the plan to the user.
    * Upon approval, insert the new tasks into `WORK_PLAN.md`.