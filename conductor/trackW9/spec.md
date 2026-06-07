# Track W9 Spec: Test and Verification Mapping Confidence

## Background

Test and verification mapping currently scores 7/10. ChangeGuard has predictive verification and test mapping, but confidence, ownership, risk class, flakiness, and missing-test recommendations need refinement.

## Objective

Raise test mapping to 9/10 by modeling tests as graph nodes with target entities, confidence, risk classes, flakiness, last result, owners, and coverage imports.

## Proposed Design

1. Add durable test nodes with kind, owner, target entity, risk class, flakiness, last result, coverage confidence, and evidence.
2. Link tests to endpoints, handlers, symbols, data models, migrations, config keys, services, ADRs, dependencies, and security boundaries.
3. Add coverage import adapters for common coverage formats when present.
4. Add `verify explain --entity` and `changeguard tests for <entity>` query surfaces.
5. Add risk rules for high-impact changes without mapped tests or with stale/flaky mapped tests.

## Critical Files

| File | Expected work |
|---|---|
| `src/index/test_mapping.rs` | Extend test extraction and entity links |
| `src/verify/predict.rs` | Consume confidence and risk classes |
| `src/verify/predictor.rs` | Blend mapping and historical signals |
| `src/commands/verify.rs` | Add explain/query surface |
| `tests/` | Add focused mapping and prediction regression tests |

## Definition of Done

- Tests can be queried by entity, service, endpoint, risk class, and confidence.
- Predictive verification explains why tests were selected or omitted.
- Missing, stale, or flaky mapped tests raise actionable risk.
- Target score after completion: 9/10.
