# Specification: Declarative Logical & Security Checks in Datalog (Track KD3)

## Overview
Move static-analysis verification checks—specifically security boundary authorization checks and entrypoint-to-sink/unreachable code detection—from imperative Rust code loops into declarative Datalog rules.

## Architecture & SRP
- **Modules**: `src/commands/security.rs`, `src/impact/analysis/dead_code/evidence.rs`
- **Responsibility**: Isolate policy verification and call path correctness checking logic inside the database query layer.

## Requirements
- Define custom Datalog rules for security boundary checks (e.g., matching principal, action, resource, and service bindings to detect unauthorized access paths).
- Migrate the imperative checking loop in `security.rs` to query the Datalog rule results directly.
- Define a unified Datalog rule for reachable/unreachable entrypoint routes and sinks.
- The Rust application should act as a runner that triggers the Datalog scripts and formats the output, leaving the complex recursive graph matching to CozoDB.

## Dependencies
- Track KD2 must be completed.
