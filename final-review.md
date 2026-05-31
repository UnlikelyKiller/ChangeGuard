Reviewed current `main` at `420a2ee` against the pre-session baseline `3e898d6`, which covers the Q/R and resilience commits `0dd3202..420a2ee`. I did not modify files.

**Findings**

- High: The network fail-fast change breaks generation-only local-model configs. In [src/local_model/client.rs](/C:/dev/ChangeGuard/src/local_model/client.rs:116), `complete()` returns an error as soon as `base_url` is empty, before it considers `generation_url`, even though [ping_completions](/C:/dev/ChangeGuard/src/local_model/client.rs:42) correctly supports `generation_url`-only probing. The resulting error string also renders an empty URL. This is a real regression in the split-endpoint topology the session was hardening.

- Medium: The “strict trigram pre-filter” for ranked search is not actually strict. In [src/search/tantivy_engine.rs](/C:/dev/ChangeGuard/src/search/tantivy_engine.rs:143), the trigram clause is added as `Occur::Should` while the BM25 query is `Occur::Must`, so documents can still match without satisfying the trigram gate. That means the search-precision objective is not enforced; at best this tweaks scoring, it does not prune noisy matches as the comments claim.

- Medium: GPU discovery can still pick the wrong adapter on hybrid systems. In [src/platform/gpu.rs](/C:/dev/ChangeGuard/src/platform/gpu.rs:26), selection is based on the highest reported `current_usage` across adapters, nodes, and both `LOCAL` and `NON_LOCAL` memory groups. That can prefer an active iGPU or shared-memory path over the discrete adapter, so `doctor` may still report the wrong VRAM/budget on the exact class of systems this change was meant to fix.

- Medium: `ledger gc --orphans` can report success semantics too loosely. In [src/commands/ledger.rs](/C:/dev/ChangeGuard/src/commands/ledger.rs:488), per-transaction rollback failures are only logged via tracing and the command still returns `Ok(())`. If every candidate fails, the user gets neither a non-zero exit nor a clear failure summary. For an auditable cleanup path, that makes it too easy to believe GC completed when it did not.

- Low: Coverage is thin around the new ergonomics and audit paths. I found the new timestamp/signature fix unit test in [tests/ledger_signature_fix.rs](/C:/dev/ChangeGuard/tests/ledger_signature_fix.rs:16), but I did not find tests for `ledger gc --orphans`, for creation of `EntryType::Rollback`, or for the new `help/version` rewriting in [src/main.rs](/C:/dev/ChangeGuard/src/main.rs:22). The existing CLI help test still only checks top-level `--help` in [tests/cli_binary.rs](/C:/dev/ChangeGuard/tests/cli_binary.rs:4).

- Low: User-facing docs were not brought up to the same level as the implementation. [docs/Features.md](/C:/dev/ChangeGuard/docs/Features.md:9) still describes rollback generically and does not mention the new required rollback reason, auditable rollback entries, or `ledger gc --orphans`.

**What looked solid**

The signature timestamp preservation itself appears correctly fixed: [src/ledger/transaction.rs](/C:/dev/ChangeGuard/src/ledger/transaction.rs:230) now honors `committed_at`, the hook sidecar carries it through [src/commands/hook_commit_msg.rs](/C:/dev/ChangeGuard/src/commands/hook_commit_msg.rs:421) and [src/commands/hook_post_commit.rs](/C:/dev/ChangeGuard/src/commands/hook_post_commit.rs:74), and the regression test in [tests/ledger_signature_fix.rs](/C:/dev/ChangeGuard/tests/ledger_signature_fix.rs:16) covers the original drift bug.

**Residual risk**

I could not run fresh verification in this read-only sandbox. The latest recorded verify artifact, [latest-verify.json](/C:/dev/ChangeGuard/.changeguard/reports/latest-verify.json:1), shows only a failing placeholder command (`exit 1`), so there is no session-end successful verification record to rely on.