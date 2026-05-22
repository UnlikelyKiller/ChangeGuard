# Track K12: Local Model Timeout and Readiness UX

## Status
Planned

## Milestone
K: Service Discovery & Storage Hardening

## Problem
`doctor` and local-model backed `ask` commands can take about a minute to fail when the configured completion endpoint is unavailable. The output is technically accurate but too slow for routine command checks.

The current `local_model.base_url` also assumes embeddings and completions live behind the same OpenAI-compatible server. The LLM2 sandbox splits these duties: Gemma chat runs on `http://127.0.0.1:8081`, while Nomic Embed v1.5 runs on `http://127.0.0.1:8083` using CPU ONNX. ChangeGuard needs first-class support for that topology.

## Objective
Make local model readiness checks fail fast, report embedding and completion readiness separately, support distinct embedding and generation base URLs, and provide actionable recovery guidance without blocking unrelated command diagnostics.

## Scope
- Add shorter default connection/read timeouts for liveness probes.
- Extend config with separate embedding and generation base URLs while preserving backward compatibility with `local_model.base_url`.
- Separate doctor probe timing from full inference request timing.
- Probe the embedding endpoint independently from the generation endpoint.
- Ensure `ask --backend local` fails quickly when completion is unreachable.
- Ensure semantic indexing/search uses the embedding endpoint, not the generation endpoint.
- Preserve configurable longer timeouts for real generation.

## Non-Goals
- Do not require users to run embeddings and completions in the same server.
- Do not remove `local_model.base_url` compatibility in this track.
- Do not make `doctor` perform a full long-form generation benchmark.

## Implementation Notes
- Treat `embedding_base_url` and `generation_base_url` as optional overrides; if either is absent, fall back to `base_url`.
- Keep model names independently configurable because Nomic, Gemma, BGE, and Qwen servers expose different identifiers.
- Liveness probes should use minimal payloads and short probe timeouts, while real generation keeps the configured generation timeout.
- The LLM2 acceptance fixture is Gemma chat on `127.0.0.1:8081` and Nomic Embed v1.5 on `127.0.0.1:8083`.

## Success Criteria
- [ ] `changeguard doctor` completes quickly when the completion endpoint is unavailable.
- [ ] `ask --backend local` fails fast with a concise endpoint and timeout diagnostic.
- [ ] Doctor output distinguishes embedding readiness, completion readiness, configured URLs, configured models, and timeout used.
- [ ] Config supports `embedding_base_url = "http://127.0.0.1:8083"` and `generation_base_url = "http://127.0.0.1:8081"` with `base_url` as a compatibility fallback.
- [ ] The LLM2 topology works: Nomic Embed v1.5 serves embeddings on CPU while Gemma serves completions on GPU.
- [ ] Config exposes clear knobs for probe timeout versus generation timeout.
- [ ] CI gate passes.

## Definition of Done
- [ ] Existing single-URL configs continue to work without migration.
- [ ] Split endpoint configs route embedding requests only to `embedding_base_url` and chat requests only to `generation_base_url`.
- [ ] `changeguard doctor` reports both LLM2 endpoints as healthy when they are running, and completes quickly when one is down.
- [ ] `changeguard ask --backend local "Say ok"` works against the generation endpoint in the LLM2 topology.
- [ ] `changeguard index --semantic` or an equivalent focused embedding smoke uses the embedding endpoint in the LLM2 topology.
- [ ] `changeguard verify` passes.
- [ ] `cargo install --path . --force` succeeds and installed-binary local-model smoke checks pass.
