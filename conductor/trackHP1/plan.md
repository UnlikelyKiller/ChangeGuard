# Plan: Track HP1 (Fast Network Seams & Non-Blocking TCP Connect Probes)

- [ ] 1. Implement non-blocking TCP socket connect checks in a new helper `src/util/network.rs`.
- [ ] 2. Update `ping_completions` in `src/local_model/client.rs` to run the TCP socket connect check first.
- [ ] 3. Update `query_external_cli` in `src/bridge/client/client_cli.rs` to fast-fail if the AI-Brains daemon port or CLI execution fails to respond immediately.
- [ ] 4. Write unit tests for TCP socket checking behavior.
