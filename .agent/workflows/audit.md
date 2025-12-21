---
description: 
---

# Workflow: Audit

**Goal**: Ensure code quality, documentation accuracy, and architectural integrity.

## Protocol Steps

1.  **Temporal Scan**
    * Review the code just generated.
    * Grep for "temporal" words: `Added`, `Updated`, `Changed`, `Fixed`, `Removed`.
    * If found, rewrite the comment to be timeless.

2.  **Invariant Check**
    * Verify against `docs/protocol/INVARIANTS.md`.
    * *Check*: Is Physics logic in `FixedUpdate`? Are input actions buffered? Are queries using `Changed<T>` where appropriate?

3.  **Invisible Knowledge Extraction**
    * Ask: "Did we make a decision here that isn't obvious from the code?"
    * *Example*: "We used a resource for the timer instead of a component because it's global state."
    * Action: If yes, add a note to `docs/protocol/INVARIANTS.md` under a relevant section.

4.  **Verification**
    * Confirm the implementation meets the "Acceptance Criteria" listed in `WORK_PLAN.md` for the active task.