# Specification: CozoDB-Redux Dependency Upgrade (Track KD1)

## Overview
Upgrade ChangeGuard's `cozo` and `cozo-sys` dependencies to target the updated `CozoDB-redux` fork repository. Ensure the build, doctor commands, and test suites are fully compatible and run cleanly under the new engine version.

## Architecture & SRP
- **Module**: `Cargo.toml`
- **Responsibility**: Maintain third-party library dependency declarations.

## Requirements
- Update `cozo` dependency in `Cargo.toml` to point to the `UnlikelyKiller/cozo-redux` fork on GitHub.
- Run `cargo clean`, rebuild, and resolve any compile-time version mismatches.
- Verify path confinement and Windows filesystem locking behaves correctly during cold start.
- Ensure `changeguard doctor` successfully initializes the CozoDB engine and verifies existing tables.
- All integration and unit tests must run and pass under the new engine.

## Dependencies
- Sled -> Fjall transition does not affect SQLite/Mem backend, but check that SQLite and Mem drivers compile without issue.
