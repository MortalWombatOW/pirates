---
description: Prepare a high-quality context handoff for the next AI agent. Run at the end of a session or when hitting a complex blocker.
---

# Workflow: Collaborate (Handoff)

**Goal**: Enable seamless context transfer between AI sessions by packaging the current mental state into a clear directive for the next agent.

## Protocol Steps

1.  **State Verification**
    *   Run `cargo check`.
    *   If it fails, fix it. **Constraint**: Never hand off a broken build without explicit documentation of *why* it is broken and how to fix it.

2.  **Context Synthesis**
    *   Update `WORK_LOG.md` with the latest changes.
    *   Identify the "Active Working Set": Which files are currently open or being modified?
    *   Identify "Invisible Context": What did you learn that isn't written down yet? (e.g., "The physics engine is unstable at high velocities"). Add this to `AGENT.md` (Invisible Knowledge) if permanent, or the Handoff Note if transient.

3.  **Handoff Note Generation**
    *   Create a specialized comment block or a temporary file (e.g., `NEXT_AGENT_INSTRUCTIONS.md`) containing:
        *   **Current Goal**: What is the immediate objective?
        *   **Status**: What is working? What is broken?
        *   **Next Command**: What specific command should the next agent run? (e.g., "Run `/forge` on task 5.2.1").
        *   **Pitfalls**: What should they avoid?

4.  **Communication**
    *   Notify the user with the Handoff Note.
    *   Explicitly state: "Session Handoff Ready. Please provide the note above to the next agent."

## Example Handoff Note

```markdown
# AI Handoff Note
**Current Task**: Epic 6.2 - Companion Routing
**Status**: Implementation complete, but tests failing on edge cases (island collision).
**Active Files**: `src/systems/navigation.rs`, `src/components/companion.rs`.
**Next Steps**:
1. Debug the raycasting logic in `navigation.rs`.
2. Run `cargo test navigation` to verify.
**Warning**: Do not increase the recursion depth of the pathfinder, it causes stack overflows.
```
