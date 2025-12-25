---
description: Implement a single atomic task with high quality. Run after /architect or /next-task. Run before /audit.
---

# Workflow: Forge

**Goal**: Implement a single atomic task with high quality.

## Protocol Steps

1.  **Strategy Formulation & Design**
    * **Scope Safety**: Check `WORK_PLAN.md`. If the requirement is missing, **ABORT** and trigger `/architect`. Do not invent tasks.
    * **Bevy Rule Injection**: Read `AGENT.md`. Quote the specific section of the rules that applies to this task (e.g., "Input Handling", "ECS Optimization") in your plan.
    * **Detailed Design**: List components/systems to modify. Describe the proposed logic and structural changes.
    * **User Review**: Present this plan to the user. **STOP** and wait for user approval or feedback before implementation.

2.  **Implementation**
    * Generate the code.
    * **Constraint**: Apply the "Temporal Purity Standard" from `AGENT.md`. Describe current behavior only.

3.  **Verification**
    * Run `cargo check`.
    * **Warning Resolution**: If warnings exist, apply `AGENT.md` (Warning Resolution):
        * For unused variables: Determine intent—remove if truly unused, integrate if implementation is incomplete.
        * For dead code: Determine if caller is missing (fix it) or code is obsolete (delete it).
        * **NEVER** prefix with `_` just to silence a warning without understanding the root cause.
    * **No-Fallback Check**: If you are replacing existing functionality, verify you are NOT reverting to the old implementation. If the new implementation isn't working, debug it systematically per `AGENT.md` (No-Fallback Rule).
    * Verify the implementation meets the criteria in `WORK_PLAN.md`.
    * **Requirement**: Zero warnings before proceeding to `/audit`.

## Interrupt Protocol: Bug/Error Discovery

**Trigger**: User identifies a bug or error during implementation.

1.  **Documentation**
    *   Add a subtask to `WORK_PLAN.md` (indented under the current task) describing the error (e.g., `- [ ] FIX: Panic on collision`).

2.  **Resolution**
    *   Diagnose and fix the issue immediately.

3.  **Verification**
    *   Ask the user: "Is this resolved?"
    *   **Wait** for confirmation.

4.  **Resumption**
    *   Mark the subtask as `[x]`.
    *   Resume the original task.

4.  **Handoff**
    * Remind the user to run `/audit` to review the code.