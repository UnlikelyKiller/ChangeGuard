# Track GF4 Plan: Ledger Database Query Domain Split

## Phase 0: Baseline and Ledger Safety

- [ ] Confirm ledger state with `changeguard ledger status --compact`.
- [ ] Start the track transaction: `changeguard ledger start trackGF4 --category REFACTOR --message "Ledger DB query domain split"`.
- [ ] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [ ] Run `changeguard search "LedgerDb" --auto-index`.
- [ ] Run `cargo test ledger::db`.
- [ ] Run focused integration tests for ledger command surfaces.

Definition of done: The implementer has baseline ledger behavior and call-site inventory.

## Phase 1: Facade Pattern

- [ ] Create query-domain modules under `src/ledger/db/` or equivalent.
- [ ] Move one read-only query group first, such as ADR list or search mapping.
- [ ] Preserve the existing `LedgerDb` method signature.
- [ ] Run `cargo check --all-targets --all-features`.

Definition of done: The facade pattern is proven on a low-risk query group.

## Phase 2: Lifecycle and Drift Domains

- [ ] Extract transaction lifecycle queries.
- [ ] Extract drift/reconcile/adopt queries.
- [ ] Add tests for pending conflict, commit promotion, rollback, and drift status counts.
- [ ] Run ledger-focused tests.

Definition of done: High-risk lifecycle logic is separated and protected by tests.

## Phase 3: Remaining Domains

- [ ] Extract search and FTS queries.
- [ ] Extract ADR queries.
- [ ] Extract enforcement and validator queries.
- [ ] Extract federation queries.
- [ ] Extract provenance and graph-link queries.
- [ ] Run affected integration tests after each domain.

Definition of done: `db.rs` no longer mixes unrelated query domains.

## Phase 4: Final Verification

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo nextest run --lib --bins --workspace`.
- [ ] Run `cargo nextest run --test integration`.
- [ ] Run `changeguard verify`.
- [ ] Run `cargo install --path .`.
- [ ] Verify the git hooks still pass end-to-end: make a trivial commit in a temp clone or confirm `ledger status --compact --exit-code` returns 0, since pre-commit/pre-push depend on this DB layer.
- [ ] Commit the track transaction: `changeguard ledger commit <tx-id> --summary "Completed Track GF4" --reason "<why>"`. If the git pre-commit hook removed the sidecar and status still shows 1 pending after the git commit, run `ledger commit` immediately.
- [ ] Run `changeguard ledger status --compact` and confirm `0 pending, 0 unaudited drift`.

Definition of done: Full gates pass, installed binary matches source, hooks still work, and the ledger is clean.
