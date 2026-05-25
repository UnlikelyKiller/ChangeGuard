# Track HP1: Fast Network Seams & Non-Blocking TCP Connect Probes

## Objective
Accelerate external/local network probes to prevent CLI latency when dependencies (local models, AI-Brains daemon) are offline or degraded.

## Requirements
- **Non-Blocking TCP Connect Checks**: Replace the 5-second HTTP connection timeouts for local model probing (port `8081` and `8083`) with a non-blocking TCP socket connection check that fails fast (e.g. <100ms timeout) before initiating any heavy HTTP client request.
- **Fast AI-Brains Connection Detection**: Optimize `query_external_cli` in `src/bridge/client/client_cli.rs` and IPC connection checks to fail fast when `ai-brains` is unreachable or uninstalled, reducing the 5-second blocking lag to <100ms when running command pipelines.

## Definition of Done (DoD)
- [ ] Local model check completes in under 100ms when the local model server is down.
- [ ] `changeguard ask` query returns immediately with a clear error without a 5-second lag when the AI-Brains daemon is unconfigured.
- [ ] All network probe checks are verified with unit tests using mocked endpoints.
