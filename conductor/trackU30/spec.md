# Track U30: verify Warning Hygiene

**Status:** ✅ **Completed**
**Started:** 2026-06-02
**Owner:** Antigravity
**Priority:** P2 — UX / Output Noise

---

## Problem Statement

Running `changeguard verify` emits noisy `WARN` traces about empty diffs (`Semantic prediction: diff_text is empty; skipping outcome recording`) even on successful/green runs, which pollutes CI pipeline output logs.

## Acceptance Criteria

**AC1:** The semantic predictor skips emitting this warning when it is expected behavior (e.g., when there is no diff to verify).

## Design Notes

- In `src/verify/semantic_predictor.rs`, demote or skip logging warning messages when diff is naturally empty.

## Verification

- Run `changeguard verify` on a clean working directory and confirm no empty diff warnings are logged to stderr.
