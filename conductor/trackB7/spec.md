# Specification: Verification Feedback Loop (Track B7)

## Overview
Push test and verification outcomes back to AI-Brains, permitting the memory vault to record structurally flaky modules and risk predictions.

## Architecture & SRP
- **Module**: `src/verify/mod.rs` and `src/bridge/notify.rs`
- **Responsibility**: Dispatch verification events cleanly post-execution without interfering with the CLI’s standard exit code behavior.

## Requirements
- After `verify.commands` runs tests/checks, map any failures into a `BridgeRecord::VerifyOutcome`.
- Fire `IpcClient::send_record` to emit the update.
- **Fail-open**: The transmission must be entirely fire-and-forget. If the pipe rejects it, trap the error using `tracing::debug!` and silently discard. It must never fail the underlying verification.
