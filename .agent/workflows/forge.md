---
description: 
---

# Workflow: Forge

**Goal**: Implement a single atomic task with high quality and adherence to Bevy ECS patterns.

## Protocol Steps

1.  **Task Acquisition**
    * Read the next available task in `WORK_PLAN.md` marked `[ ]`.
    * Read `docs/protocol/INDEX.md` to identify relevant files for this task.

2.  **Strategy Formulation**
    * Explicitly write out your plan in the chat:
        1.  **Plan**: What components/systems need changing.
        2.  **Implementation**: The specific code changes.
        3.  **Verification**: How we know it works (e.g., "The ship should stop moving when Space is held").

3.  **Implementation**
    * Generate the code.
    * **Strict Constraint**: Apply "Temporal Purity" to all comments. Do not mention "added", "updated", or "fixed". Describe the current behavior only.

4.  **Completion**
    * Mark the task as `[/]` (In Progress) if partial, or wait for `/audit` before marking `[x]`.