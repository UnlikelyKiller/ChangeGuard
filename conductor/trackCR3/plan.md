# Track CR3 Plan: Calibrate AI-Brains Timeout & Local Model Probe

## Phase 1: Implementation
- [x] Change AI-Brains CLI fallback timeout from 800ms to 2000ms in `src/bridge/client/client_cli.rs`.
- [x] Change local model TCP preflight probe from 150ms to 500ms in `src/local_model/client.rs`.
- [x] Change embedding model TCP preflight probe from 150ms to 500ms in `src/embed/client.rs`.

## Phase 2: Testing & Verification
- [x] `cargo test` passes — existing connectivity tests remain green.
