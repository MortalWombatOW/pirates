---
description: A workflow for finishing a task.
---

Follow these steps:
1. **Verification**: Run `cargo check` to ensure the project is in a compiling state.
2. **Clean State**: Ensure all edited files are clean and comments add useful context about why things were changed.
3. **Documentation**:
   - Update the task status in `WORK_PLAN.md` to `[x]`.
   - Add an entry to `WORK_LOG.md` explaining the task, what you did, and what we've learned.
   - Are there any updates or additions to the README.md that would be helpful for the future or for accuracy?
4. **Git Workflow**: Commit the changes with a descriptive message and push immediately. Refer to `.agent/rules/git.md`.
5. **Completion Signal**: Only after the push is successful, notify the user of completion.