---
description: Picking up the next task from the work plan.
---

Follow these steps to proceed with the next task in the project:

1. **Context Loading & Research**:
   - Read [README.md](file:///Users/andrewgleeson/Documents/code/pirates/README.md) to understand the full Game Design Document and Technical Specification.
   - Read [WORK_PLAN.md](file:///Users/andrewgleeson/Documents/code/pirates/WORK_PLAN.md) to find the current project status.
   - **Research**: Carefully research the task requirements. Before writing New code, check Bevy documentation and the existing codebase/libraries (e.g., Avian, Egui, Tilemap, Leafwing) to identify features and patterns that can be re-used.

2. **Task Selection**:
   - Identify the next available task in `WORK_PLAN.md`. 
   - An available task is one marked with `[ ]` whose dependencies (if any) are already marked as `[x]`.
   - Priority should be given to lower IDs within the same Phase and higher priority (`P0` > `P1` > `P2`).

3. **Status Update**:
   - Mark the selected task as in-progress by changing `[ ]` to `[/]` in `WORK_PLAN.md`.

4. **Design & Implementation**:
   - **Architectural Integrity**: Design your implementation to reduce technical debt. Ensure clean boundaries between systems and maintain consistent practices as defined in the technical specification (e.g., proper use of Components vs Resources, standard Event patterns).
   - Perform the task as described in its "Task" and "Acceptance Criteria" columns.
   - Refer back to the `README.md` for specific technical details, component definitions, or architectural patterns.

5. **Completion & Documentation**:
   - Once the task and its acceptance criteria are met, update the task in `WORK_PLAN.md` by changing `[/]` to `[x]`.
   - Update `README.md` if the task revealed any necessary changes to the design or technical specification.
   - Summarize the work done in the `WORK_LOG.md`.

6. **Cleanup**:
   - If the task was part of a larger Epic or Phase that is now complete, update the status indicators accordingly.
