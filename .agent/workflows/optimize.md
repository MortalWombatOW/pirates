---
description: Optimize assistant instructions through structured analysis.
---

# Workflow: Optimize

**Goal**: Analyze and improve the project's internal documentation and rules.

## Protocol Steps

1.  **Analysis Phase**
    * Review `WORK_LOG.md` and recent Chat History.
    * Examine `.agent/rules/*` and `.agent/workflows/*`.
    * Look for:
        * Recurring errors (e.g., "I keep forgetting the Bevy input rule").
        * Friction in workflows (e.g., "Forge is doing too much").

2.  **Proposal**
    * Propose specific changes to the `.agent/` files.
    * Explain *why* this improves performance or consistency.

3.  **Implementation**
    * Upon approval, edit the files.
    * Log the optimization in `WORK_LOG.md`.