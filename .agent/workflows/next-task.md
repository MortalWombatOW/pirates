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

> [!IMPORTANT]
> **Task Order Review**: Complete tasks in numerical order within each Epic. If you believe a task has incorrect dependencies or is logically out of sequence:
> 1. **Stop** before starting any out-of-order work.
> 2. **Propose** a corrected task order to the user with your reasoning.
> 3. **Wait** for approval before proceeding.
>
> Do not silently reorder or batch tasks together.

3. **Status Update**:
   - Mark the selected task as in-progress by changing `[ ]` to `[/]` in `WORK_PLAN.md`.

4. **Design & Implementation**:
   - **Architectural Integrity**: Design your implementation to reduce technical debt. Ensure clean boundaries between systems and maintain consistent practices as defined in the technical specification (e.g., proper use of Components vs Resources, standard Event patterns).
   - Perform the task as described in its "Task" and "Acceptance Criteria" columns.
   - Refer back to the `README.md` for specific technical details, component definitions, or architectural patterns.
   - **API Verification**: Before writing code that uses external crates, verify the exact API by checking the crate's source or examples. Use `grep` on the cargo registry or run `cargo doc --open` to confirm method signatures.

5. **Completion & Documentation**:
   - Once the task and its acceptance criteria are met, update the task in `WORK_PLAN.md` by changing `[/]` to `[x]`.
   - Update `README.md` if the task revealed any necessary changes to the design or technical specification.
   - Summarize the work done in the `WORK_LOG.md`.
   - **Git Workflow**: Immediately commit and push your changes. Refer to `.agent/rules/git.md`.

6. **Cleanup**:
   - If the task was part of a larger Epic or Phase that is now complete, update the status indicators accordingly.
