# Track CR3 Plan: Calibrate AI-Brains Timeout & Local Model Probe

## Phase 1: AI-Brains CLI Timeout Calibration
- [ ] In `src/bridge/client/client_cli.rs`, increase the `Duration::from_millis(800)` timeout to `Duration::from_millis(2000)`.
- [ ] Add `bridge_timeout_ms` (default 2000) to the configuration structure so it can be customized.

## Phase 2: Local Model TCP Preflight Relaxation
- [ ] In `src/embed/client.rs` and `src/local_model/client.rs` (or where the TCP connect probe is executed), increase the 150ms timeout to 500ms to allow slower containers/WSL interfaces to respond.
- [ ] Ensure that preflight connection failures log descriptive warnings instead of hard failing when request-level fallbacks are available.

## Phase 3: Verification
- [ ] Run `changeguard doctor` and verify that the embedding and completion model server probes are robust.
- [ ] Add mock integration tests representing slow-connecting endpoints to verify timeout behaviors.
