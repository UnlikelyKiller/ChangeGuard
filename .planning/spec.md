# Technical Specification: Phase 2 - Repo-Local State Layout and Init

## 1. Overview
This specification details the implementation of Phase 2 for Changeguard, focusing on repository-local state management, the `init` command, and `.gitignore` integration. The design strictly adheres to the principles of safe, idempotent file operations, deterministic behavior, and idiomatic Rust as defined in the architectural plans.

## 2. Objectives
- Establish the `.changeguard/` local directory layout.
- Scaffold the `init` CLI command.
- Integrate with `.gitignore` to prevent committing local state.
- Generate deterministic starter configuration files (`config.toml` and `rules.toml`).
- Handle failure modes gracefully using `miette` and `thiserror`.

## 3. Architecture & Module Boundaries

### 3.1 `src/cli.rs` & `src/commands/init.rs`
- **CLI Definition:** Add the `Init` variant to the CLI parser with an optional boolean flag `--no-gitignore`.
- **Command Handler:** The `init` handler acts as the orchestrator, invoking state layout creation, configuration writing, and `.gitignore` modification.
- **Responsibility:** Command parsing, user-facing output/diagnostics, and high-level flow.

### 3.2 `src/state/layout.rs`
- **Responsibility:** Encapsulate all knowledge of the `.changeguard/` directory structure.
- **API Contract:**
  - `pub fn ensure_state_dir(repo_root: &Path) -> Result<PathBuf, StateError>`
  - `pub fn initialize_subdirs(state_dir: &Path) -> Result<(), StateError>`
- **Directory Layout:**
  - `logs/`
  - `tmp/`
  - `reports/`
  - `state/`

### 3.3 `src/config/defaults.rs` & `src/config/mod.rs`
- **Responsibility:** Provide and persist starter TOML files.
- **API Contract:**
  - `pub fn write_starter_configs(state_dir: &Path) -> Result<(), ConfigError>`
- **Files:** `config.toml` and `rules.toml`.

### 3.4 `src/git/ignore.rs` (or similar utility)
- **Responsibility:** Idempotent modification of `.gitignore`.
- **API Contract:**
  - `pub fn append_to_gitignore(repo_root: &Path, entry: &str) -> Result<(), FsError>`
- **Behavior:** 
  - Reads `.gitignore` if it exists.
  - Checks if `.changeguard/` is already present.
  - Appends `.changeguard/` (with a newline) only if missing.
  - Gracefully ignores missing `.gitignore` by creating it, unless not in a git repo.

## 4. Engineering Principles & Constraints

### 4.1 Idiomatic Rust & Error Handling
- **No panics:** `unwrap()`, `expect()`, and `panic!()` are explicitly forbidden in production code. Use `?` for propagation.
- **Error Visibility:** 
  - Define explicit error enums using `thiserror::Error` for domain-specific failures (e.g., `StateError::Io`, `StateError::PermissionDenied`).
  - Command boundaries must return `miette::Result<()>` to provide rich, actionable diagnostics to the user.
  - Example Error: "Failed to create `.changeguard/`. Permission denied. Please check your directory permissions."

### 4.2 Single Responsibility Principle (SRP)
- `state/layout.rs` does not know about `.gitignore`.
- `commands/init.rs` does not manually concatenate paths.
- `config/` only deals with TOML defaults, not creating the parent directory.

### 4.3 Determinism and Edge Cases
- **Idempotency:** Running `changeguard init` multiple times must be safe and result in the exact same state without duplicating entries in `.gitignore` or corrupting configurations.
- **Cross-Platform Pathing:** Use `std::path::Path` and `PathBuf` for all operations to ensure correct behavior on Windows, WSL, and Linux.
- **File Encoding:** Write files strictly as UTF-8. Account for CRLF vs LF conceptually if needed, but defaults will be standard Rust string handling.

## 5. Testing Strategy (TDD)
- **Unit Tests:** Verify `state/layout.rs` path generation and `git/ignore.rs` line matching without touching the actual filesystem (or using temporary directories).
- **Fixture Tests:** Test `.gitignore` appending with various mocked file states (missing, empty, ends without newline, already contains entry).
- **Integration Tests:** Execute `init` command via CLI test harness in a temporary directory and assert the full directory structure and file contents.
