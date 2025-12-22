---
description: 
---

# Workflow: Forge

**Goal**: Implement a single atomic task with high quality.

## Protocol Steps

1.  **Strategy Formulation**
    * **Scope Safety**: Check `WORK_PLAN.md`. If the requirement is missing, **ABORT** and trigger `/architect`. Do not invent tasks.
    * **Bevy Rule Injection**: Read `.agent/rules/bevy.md`. Quote the specific section of the rules that applies to this task (e.g., "Input Handling", "ECS Optimization") in your plan.
    * **Plan**: List components/systems to modify.

2.  **Implementation**
    * Generate the code.
    * **Constraint**: Apply the "Temporal Purity Standard" from `.agent/rules/development.md`. Describe current behavior only.

3.  **Verification**
    * Run `cargo check`.
    * Verify the implementation meets the criteria in `WORK_PLAN.md`.

4.  **Handoff**
    * Trigger `/audit` to review the code.