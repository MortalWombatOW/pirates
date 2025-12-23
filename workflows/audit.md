---
description: Comprehensive code review to ensure technical quality and product alignment. Run after /forge. Run before /accept (if passed) or /forge (if failed).
---

# Workflow: Audit

**Goal**: Comprehensive code review to ensure technical quality and product alignment.

## Protocol Steps

1.  **Product & Context Alignment**
    * Does the implementation strictly match the active task in `WORK_PLAN.md`?
    * Have changes been reflected in `WORK_LOG.md`?

2.  **Code Quality & Norms**
    * **Bevy Optimization**: Verify compliance with `AGENT.md` (Query filters, Command usage).
    * **Input Handling**: Verify no `.pressed()` on Axis inputs. Verify "sticky input" for FixedUpdate.
    * **Temporal Purity**: Grep for banned words (`Added`, `Fixed`, etc.) per `AGENT.md`.

3.  **Architectural Integrity**
    * Verify against `AGENT.md`.
    * Is physics logic strictly in `FixedUpdate`?
    * Are systems properly gated by `State`?

4.  **Invisible Knowledge Extraction**
    * Ask: "Did we make a decision here that isn't obvious?"
    * If yes, add a note to `AGENT.md`.

5.  **Handoff**
    * If **Failed**: Remind the user to run `/forge` to fix issues.
    * If **Passed**: Remind the user to run `/accept`.