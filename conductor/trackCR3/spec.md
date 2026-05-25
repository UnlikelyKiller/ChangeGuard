# Track CR3: Calibrate AI-Brains Timeout & Local Model Probe

## Status
Planned

## Milestone
CR: Codex Review Remediation

## Problem
1. The AI-Brains CLI fallback timeout was set to 800ms with no backoff or configuration option. On loaded systems, this causes silent drops of RAG memory context.
2. The local model connectivity check uses a 150ms TCP preflight socket probe. This is extremely aggressive for localhost container/WSL systems and produces false negatives.

## Objective
Relax and calibrate connection/execution timeouts. Make the AI-Brains CLI timeout and LLM TCP preflight probes configurable, and increase their default safety margins to prevent false negatives.

## Scope
- Modify `src/bridge/client/client_cli.rs` to change the default CLI fallback timeout to a safer default (e.g., 2.0s) or make it configurable under `[bridge]`.
- Modify `src/embed/client.rs` and `src/local_model/client.rs` to relax the 150ms TCP socket preflight timeout to 500ms, or expose it as a configuration parameter.
- Verify connectivity robustness under loaded conditions.

## Success Criteria
- [ ] AI-Brains CLI fallback timeout default is increased to at least 2 seconds.
- [ ] Local model TCP socket preflight timeout default is relaxed to at least 500ms.
- [ ] Connect probes and fallbacks succeed reliably on slower or loaded local environments.

## Definition of Done
- [ ] Defaults adjusted in `src/bridge/client/client_cli.rs` and `src/util/network.rs` (or corresponding model clients).
- [ ] Configuration options added to `Config` struct if necessary.
- [ ] Checked against slow endpoint simulation tests.
- [ ] `cargo test` passes.
