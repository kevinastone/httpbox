# Agent Guidelines for `httpbox`

This document outlines key development practices, conventions, and requirements for AI agents working in this codebase.

## Repository Overview

`httpbox` is an HTTP testing service inspired by `httpbin`, built in Rust on Hyper and Tokio. It is structured as a Cargo workspace with three member crates:
- `httpbox`: The main server binary and service definitions.
- `hyper_body`: HTTP body utilities and streaming helpers.
- `uri_path`: Route pattern-matching macro (`path!`) and URI segment extraction.

## Verification & Code Quality

Always verify workspace health before completing tasks or proposing changes. Run the following commands from the workspace root:

1. **Code Formatting (`cargo fmt`)**:
   Always run `cargo fmt` before staging commits:
   ```sh
   cargo fmt --all
   ```
   Ensure `cargo fmt --all --check` produces zero diffs.

2. **Linting (`cargo clippy`)**:
   Enforce zero warnings across all workspace members, targets, and features:
   ```sh
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   ```

3. **Testing (`cargo test`)**:
   Ensure all workspace tests pass:
   ```sh
   cargo test --workspace
   ```

## Commit Guidelines

- **Atomic Commits**: Keep changes focused and split by scope or crate (e.g. `feat(uri_path): ...`, `feat(service): ...`).
- **Conventional Commits**: Use conventional commit prefixes: `feat:`, `fix:`, `chore:`, `style:`, `test:`, `refactor:`.
- **Clean Git State**: Verify `git status` has no unintentional untracked files or unstaged changes before concluding.

## Architecture & Conventions

- **CLI Compatibility**: Be mindful not to introduce breaking changes to CLI arguments or short flags (e.g. `-h` is `--host`, `--help` is help).
- **Security & Limits**: Any endpoint accepting variable input lengths, byte counts, or memory allocation must have safe upper bounds or timeouts to prevent denial-of-service / memory exhaustion.
- **Async Runtime**: Prefer `tokio` native timing/concurrency primitives (e.g., `tokio::time::sleep`) rather than extraneous third-party crates.
- **Routing**: `path!` macro in `uri_path` supports:
  - Exact literals: `path!("api" / "v1")`
  - Dynamic parameters: `path!("status" / code)`
  - Regex-validated parameters: `path!("user" / [id ~ r"\d+"])`
  - Catch-all tail segments: `path!("assets" / *path)`
  - Regex catch-all tail segments: `path!("assets" / *[path ~ r".*\.(svg|png)"])`
  *(Catch-all segments must always be the final segment of the route).*
