# ADR: Complexity Scoring Implementation

Date: 2026-04-20

## Decision

ChangeGuard will keep the native tree-sitter complexity implementation for Phase 2 instead of adopting `arborist-metrics` 0.1.2.

## Context

`docs/Plan-Phase2.md` identified `arborist-metrics` as a potentially useful crate, but also flagged high adoption risk: it was newly published, had minimal ecosystem signal, and could introduce tree-sitter version drift against ChangeGuard's existing parsers.

The Phase 2 implementation already has native Rust, TypeScript, and Python scoring behind `ComplexityScorer`. Keeping the native implementation avoids adding a young dependency while preserving deterministic local behavior.

## Consequences

- No new dependency is added for Phase 2 complexity scoring.
- Complexity behavior stays under ChangeGuard's tests and release cadence.
- Future adoption remains possible behind `ComplexityScorer` if the external crate matures and proves compatible.
