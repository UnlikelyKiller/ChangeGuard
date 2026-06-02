# Track U14 Plan: Semantic Indexing Concurrency Auto-Tuning

- [x] Task U14.1: Implement automatic thread-limit logic inside `src/commands/index.rs`. *(Done via the new `resolve_semantic_concurrency` helper in `src/semantic/concurrency.rs`, consumed at `src/commands/index.rs:630`.)*
- [x] Task U14.2: Parse system hardware details using dynamic boundaries to fallback safely under virtualized/containerized shells. *(Uses `std::thread::available_parallelism()` which respects cgroup/affinity limits; tested via `auto_falls_back_to_one_when_parallelism_unknown`.)*
- [x] Task U14.3: Assert that dynamic concurrency tuning keeps thread layouts optimal during heavy indexes. *(Eight unit tests on the resolver + two on the embed semaphore; the resolver returns `NonZeroUsize` so downstream callers cannot pass zero to rayon.)*
- [x] Task U14.4: Cap concurrent embed requests independently to avoid overwhelming the local ONNX server. *(Done via `EmbedSemaphore` + `DEFAULT_EMBED_CAP=4`; prevents the regression from commit `90da256`.)*
