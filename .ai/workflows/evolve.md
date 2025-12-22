---
description: Improve the agent's behavior and the project's rules based on friction or errors. Run when needed.
---

# Workflow: Evolve

**Goal**: Improve the agent's behavior and the project's rules based on friction or errors.

## Protocol Steps

1.  **Reflection**
    * Identify the friction point (e.g., "The agent keeps forgetting to import the `prelude`").

2.  **Mutation**
    * Determine where the fix belongs:
        * **Persona/Behavior**: Update `docs/protocol/MANIFESTO.md`.
        * **Technical Rule**: Update `docs/protocol/INVARIANTS.md`.
        * **Process Step**: Update a specific workflow file (e.g., `docs/protocol/workflows/forge.md`).

3.  **Application**
    * Apply the change to the file.
    * Log the "Evolution" in `WORK_LOG.md`.