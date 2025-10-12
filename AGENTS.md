# Repository Guidelines

## Project Structure & Module Organization
The Rust entrypoint lives in `src/main.rs`, with focused modules for configuration (`config.rs`), Maven orchestration (`maven.rs`, `project.rs`), terminal UI state (`tui.rs`) and reusable widgets under `src/ui/`. Shared helpers reside in `utils.rs`. Use the `demo-project/` Maven workspace as the canonical fixture for manual and automated tests; build outputs land in `target/` and should stay untracked.

## Build, Test, and Development Commands
- `cargo build` compiles the TUI binary; run with `--release` before distributing artifacts.
- `cargo run -- --project demo-project` boots the interface against the sample Maven repo.
- `cargo test` executes all unit and integration tests; run it before every push.
- `cargo fmt` and `cargo clippy -- -D warnings` keep formatting and linting aligned with CI expectations.

## Coding Style & Naming Conventions
Rely on stable `rustfmt` defaults (4-space indent, max width 100). Prefer `snake_case` for functions and modules, `PascalCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Keep modules focused on the domain hinted by their filename, and expose only what callsites need via `pub(crate)` when possible. Handle fallible paths with `Result` and clear error contexts instead of unchecked `unwrap`.

## Testing Guidelines
Unit tests live alongside their modules using `#[cfg(test)] mod tests`. For multi-module flows, add integration specs under `tests/` that stand up the `demo-project` fixture and cover command execution, profile selection, and log rendering. Name tests as `feature_under_test_expected_outcome` for clarity. Always run `cargo test` locally; add reproduction steps in the PR description if failures depend on Maven fixtures.

## Commit & Pull Request Guidelines
Recent history favors short, imperative commit titles (`fuzzy`, `profile`); follow that tone while keeping messages descriptive enough to scan. Group related changes into a single commit and mention linked issues via `Closes #ID` when relevant. Pull requests should outline the user-facing impact, list test commands executed, and include a screenshot or asciinema snippet when altering the TUI. Update docs or config examples (`demo-project/` or README) whenever behavior changes.
