---
description: Maintain documentation accuracy and improve agent performance through reflection. Run after /accept or when process friction is identified.
---

# Workflow: Refine

**Goal**: Maintain documentation accuracy and improve agent performance through reflection.

## Protocol Steps

1.  **Synchronization (The Map)**
    * Scan the `src/` directory for structural changes.
    * Update `docs/protocol/INDEX.md`:
        * Add new files.
        * Remove deleted files.
        * Update descriptions if responsibilities changed.
    * Ensure `WORK_LOG.md` reflects all recent activity.

2.  **Retrospective (The Mirror)**
    * Review recent work in `WORK_LOG.md` and Chat History.
    * Identify **Friction** (e.g., "The agent keeps forgetting to import the prelude").
    * Identify **Drift** (e.g., "We are ignoring the temporal purity rule").

3.  **Evolution (The Upgrade)**
    * Propose specific changes to:
        * **Rules**: `.ai/rules/*.md` or `docs/protocol/INVARIANTS.md`.
        * **Process**: `.ai/workflows/*.md`.
        * **Persona**: `docs/protocol/MANIFESTO.md`.
    * Explain *how* these changes prevent the friction identified in Step 2.

4.  **Implementation**
    * Upon user approval, apply the changes to the files.
    * Log the "Refinement" in `WORK_LOG.md`.

5.  **Handoff**
    * Wait for user input or remind them to run `/next-task` if appropriate.
