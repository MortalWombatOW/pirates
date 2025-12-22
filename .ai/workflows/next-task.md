---
description: Identify and lock the next unit of work. Run after /init or /accept. Run before /forge.
---

# Workflow: Next Task

**Goal**: Identify and lock the next unit of work.

## Protocol Steps

1.  **Context Check**
    * Ensure you have executed the `/init` workflow to load the project context.

2.  **Task Selection**
    * Identify the next available task in `WORK_PLAN.md`.
    * Criteria: Marked `[ ]`, dependencies are `[x]`.
    * **Constraint**: Strict numerical order within Epics. Do not skip.

3.  **State Transition**
    * Mark the selected task as in-progress `[/]` in `WORK_PLAN.md`.

4.  **Handoff**
    * Remind the user to run `/forge` to begin implementation.