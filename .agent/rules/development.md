# Development Workflow

To maintain high development velocity and avoid long idle times during compilation:

## Verification Priority
1. **Always run `cargo check` first** after making changes or adding dependencies. It is significantly faster than a full build/run as it skips code generation.
2. If `cargo check` fails, resolve the errors before attempting to build.
3. Only use `cargo build` or `cargo run` when verification of runtime behavior or visual elements is required.

## Performance Profiling
- If compilation feels unusually slow, use `cargo build --timings` to identify bottleneck crates.
- Bevy's `dynamic_linking` feature is enabled in `Cargo.toml` for development; Ensure it remains active to speed up incremental builds.

## Final Check
- Before considering a task "complete", ensure the project is in a compiling state (`cargo check` passes).
- **Run the Git Workflow**: Immediately commit and push your changes after every task. Refer to `.agent/rules/git.md` for details.
