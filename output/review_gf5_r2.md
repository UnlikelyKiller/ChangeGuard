Findings:

1. [src/cli/mod.rs](/C:/dev/changeguard/src/cli/mod.rs:1) breaks the “facade-only / single stable entry point” contract by making both `args` and `dispatch` public. Existing callers of `changeguard::cli::run_with` still work, and [src/main.rs](/C:/dev/changeguard/src/main.rs:62) still uses that path, but `run_with` is now also public at `changeguard::cli::dispatch::run_with`. The same applies to all CLI types via `changeguard::cli::args::*`. That is not a behavioral regression, but it is an API-surface expansion and means `run_with` is no longer the sole public entry point.

2. The added tests in [src/cli/mod.rs](/C:/dev/changeguard/src/cli/mod.rs:178) are not sufficient to guard the hidden/internal contract. They prove `internal hook-*` still parses, but they do not assert that hidden commands remain absent from help output even though `Internal` and `SearchTrigrams` are explicitly hidden in [src/cli/args.rs](/C:/dev/changeguard/src/cli/args.rs:348). They also do not exercise the feature-gated `Daemon` / `VizServer` surfaces in [src/cli/args.rs](/C:/dev/changeguard/src/cli/args.rs:356), so a future attr regression there could compile and escape this test set.

Everything else I checked looks sound. `pub use args::*;` in [src/cli/mod.rs](/C:/dev/changeguard/src/cli/mod.rs:4) keeps the prior top-level `changeguard::cli::*` facade reachable, and I did not find any dropped clap aliases/defaults/`requires`/hidden attrs/feature gates when comparing `HEAD:src/cli.rs` to [src/cli/args.rs](/C:/dev/changeguard/src/cli/args.rs:7). There are no `env = ...` clap attrs in either version. `run_with` also remains the operational entry point used by the binary, just no longer the only public path.

On your specific questions: (1) yes, the former public CLI types are still reachable through the facade; (2) I found no accidental clap-attribute drift by inspection; (3) behaviorally yes, API-wise not strictly, because `dispatch` is public; (4) I did not find a missed re-export for current in-repo consumers; (5) the 15 tests are good baseline coverage but insufficient for hidden/help-surface and feature-gated regressions; (6) downstream break risk is low, but downstream API-sprawl risk is real because the new public submodules invite consumers to bind to non-facade paths.

I did not execute tests in this session because the workspace is read-only.

