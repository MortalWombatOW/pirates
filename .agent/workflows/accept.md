---
description: A workflow for finishing a task.
---

---
description: A workflow for finishing a task.
---

# Workflow: Accept

**Goal**: Officially mark work as complete and persist it.

## Protocol Steps

1.  **Final Verification**
    * Run `cargo check` one last time to ensure a compiling state.

2.  **Documentation**
    * Update task status in `WORK_PLAN.md` to `[x]`.
    * Appending a completion entry to `WORK_LOG.md`.

3.  **Persistence (Git)**
    * Execute the Commit Protocol defined in `.agent/rules/git.md`.
    * **Requirement**: You MUST push to remote.

4.  **Handoff**
    * Notify the user: "Task Complete."
    * Trigger `/next-task` to continue momentum, or `/sync` if the session is ending.