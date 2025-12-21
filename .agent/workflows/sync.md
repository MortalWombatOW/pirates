---
description: 
---

# Workflow: Sync

**Goal**: Ensure the documentation ("The Map") accurately reflects the codebase ("The Territory").

## Protocol Steps

1.  **Index Update**
    * Scan the `src/` directory.
    * Update `docs/protocol/INDEX.md`:
        * Add new files.
        * Remove deleted files.
        * Update descriptions if a file's responsibility has changed.

2.  **Work Log**
    * Summarize the session's achievements.
    * Append to `WORK_LOG.md` using the efficient cat syntax:
        ```bash
        cat >> WORK_LOG.md <<'EOF'
        ## [Date] - [Epic/Task Name]
        - [Completed Item 1]
        - [Completed Item 2]
        EOF
        ```

3.  **Plan Cleanup**
    * Update `WORK_PLAN.md` to mark completed tasks as `[x]`.