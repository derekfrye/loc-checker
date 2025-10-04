# Repository Guidelines

## Project Structure & Module Organization
`loc-checker` is a single-binary Rust crate. The entrypoint lives in `src/main.rs`; migrate shared logic into a future `src/lib.rs` so `src/main.rs` stays focused on wiring. Organize feature-specific modules under `src/<feature>/mod.rs`, and keep user-facing assets (sample inputs, templates) under `assets/` if they become necessary. Place integration tests in `tests/`, using subfolders such as `tests/cli/` when scenarios grow. Generated build artifacts land in `target/`; never check that directory into source control.

## Build, Test, and Development Commands
Use Cargo for all workflows:
- `cargo check` — fast validation of compilation errors without producing binaries.
- `cargo fmt` — run sparingly during scheduled formatting passes (do not auto-run after every change).
- `cargo clippy --all-targets --all-features -- -W clippy::pedantic` — lint with pedantic checks enabled.
- `cargo test` — execute unit and integration tests; add `-- --nocapture` when debugging output.
- `cargo run -- <args>` — run the binary locally, e.g. `cargo run -- ./samples/input.txt` once implemented.

### Post-change verification sequence
After making code changes, run the following commands in order:
1. `cargo check`
2. `cargo test --tests`
3. `cargo clippy -- -W clippy::pedantic`
4. `cargo run -- --path <target>`
5. `cargo run -- --path <target> --output-format json`

## Coding Style & Naming Conventions
Follow rustfmt defaults (4-space indent, max width from `.rustfmt.toml` when added). Use `snake_case` for modules, files, and functions; `UpperCamelCase` for types and traits; `SCREAMING_SNAKE_CASE` for constants. Keep functions small and prefer returning `Result<T>` for fallible paths. Target total LOC per file under 200 lines and total LOC per function under 50 lines where practical. Run `cargo clippy --all-targets --all-features -- -W clippy::pedantic` prior to opening a PR; formatting is handled separately during scheduled passes.

## Testing Guidelines
Use Rust’s built-in test harness. Unit tests live alongside code in `#[cfg(test)] mod tests` blocks with descriptive names such as `test_counts_blank_lines`. Create integration tests under `tests/` that exercise the binary end-to-end. When adding fixtures, store them in `tests/data/` and load with relative paths. Aim to cover new branches introduced by a change; highlight any intentional gaps in the PR description.

## Commit & Pull Request Guidelines
The repository has no history yet, so adopt Conventional Commits (e.g., `feat: add CLI parser`, `fix: handle unreadable files`). Keep the subject line imperative and under 72 characters. For pull requests, include: purpose, notable design choices, test command output, and links to related issues. Request review once `cargo fmt`, `cargo clippy`, and `cargo test` succeed locally, and attach screenshots or sample outputs when behavior changes.

## Security & Configuration Tips
Avoid embedding credentials or absolute paths in the codebase. Prefer reading configuration from environment variables and document defaults in the README when they appear. Audit third-party crates before adding them to `Cargo.toml`, and run `cargo deny` (add via dev-dependencies) if the dependency list grows.
