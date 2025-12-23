# Claude Agent Configuration

## Initialization Protocol

You are an expert software engineer working on this project. To ensure consistency and quality, you must adhere to the following initialization protocol.

### 1. Context Loading
Immediately upon starting a session, you must read the following files to build your mental model:

**The Protocol (The Law)**
*   `docs/protocol/INDEX.md`: The map of the project structure.
*   `docs/protocol/INVARIANTS.md`: The immutable architectural rules.
*   `docs/protocol/MANIFESTO.md`: The project philosophy and your persona.

**The Product (The Goal)**
*   `README.md`: The game design document and overview.
*   `WORK_PLAN.md`: The current status of tasks.
*   `WORK_LOG.md`: The history of recent changes.

### 2. Capability Awareness
You have specific workflows defined to handle tasks safely.
*   Read `.ai/workflows/*.md` to understand your capabilities.
*   Review `.ai/config/claude/commands/*.toml` to understand the available slash commands (e.g., `/forge`, `/audit`).

### 3. Rules of Engagement
You must strictly follow the technical rules defined in:
*   `.ai/rules/development.md`: General coding standards.
*   `.ai/rules/git.md`: Version control protocols.
*   `.ai/rules/bevy.md`: Bevy-specific engine constraints.

### 4. Ready State
Once you have ingested this context, ask the user:
"Context loaded. What would you like to do?"
