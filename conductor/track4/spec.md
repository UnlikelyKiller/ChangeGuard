# Technical Specification: Git Scan Foundation (Phase 5)

## Overview
This specification outlines the architecture and technical requirements for the Track 4: Git Scan Foundation implementation of Changeguard. It focuses on implementing Phase 5 of the Implementation Plan v1, ensuring a robust, deterministic one-shot scan of a git repository, change classification, and state discovery using idiomatic Rust.

## Core Objectives
- **Repository Discovery**: Detect the root of a git repository given the current working directory.
- **Git Status Collection**: Collect unstaged and staged file paths and determine if the tree is clean or dirty.
- **Change Classification**: Map file changes accurately into `Added`, `Modified`, `Deleted`, and `Renamed` states.
- **Diff Summary & Metadata**: Extract the current HEAD commit hash, current branch name (or detached state), and a basic diff summary.
- **Pure Rust Implementation**: Leverage the `gix` (gitoxide) crate for cross-platform, dependency-free (no shell out) git operations wherever possible.

## Data Models

```rust
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed { old_path: PathBuf },
}

#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub is_staged: bool,
}

#[derive(Debug, Clone)]
pub struct RepoSnapshot {
    pub head_hash: Option<String>,
    pub branch_name: Option<String>,
    pub is_clean: bool,
    pub changes: Vec<FileChange>,
}
```

## Architecture Boundaries
The implementation must adhere to strict SRP guidelines, keeping git operations isolated from general platform code.
- `src/git/mod.rs`: The public interface for git operations.
- `src/git/repo.rs`: Repository initialization, discovery, and root directory resolution.
- `src/git/status.rs`: Traversal of the working tree and index to identify file states.
- `src/git/diff.rs`: Unstaged and staged diffing logic for change extraction.
- `src/git/classify.rs`: Core engine for assigning `ChangeType` based on `status`/`diff` results.
- `src/commands/scan.rs`: The CLI integration point executing `changeguard scan`.

## Failure Modes & Constraints
- **Unborn Branch / No Commits**: Handle empty/newly initialized repositories gracefully without panicking.
- **Detached HEAD**: `branch_name` should be correctly identified as `None`.
- **Not a Git Repo**: Return a user-friendly `miette::Diagnostic` stating the directory is not a git repository.
- **Deterministic Output**: Any output containing lists of files must be sorted deterministically to ensure stable tests.
- **Error Handling**: Use `anyhow::Result` for internal logic composition, and `miette::Result` for CLI entry points. **No `.unwrap()` or `.expect()` in production paths.**

## Edge Cases
- **Renames**: Ambiguous rename detection should not fail the scan. Ensure basic modified/deleted fallback if rename thresholds are not met or unsupported in `gix` status.
- **Ignored Files**: Exclude files matched by `.gitignore` unless explicitly requested (e.g., via a CLI flag later).
- **Submodules / Worktrees**: If `gix` encounters nested git boundaries, handle them safely or log a warning without crashing the top-level scan.