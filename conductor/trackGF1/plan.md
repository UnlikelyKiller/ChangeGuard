# Track GF1 Plan: Impact Packet Domain Type Split

## Phase 0: Baseline and Guardrails

- [x] Confirm ledger state with `changeguard ledger status --compact`.
- [x] Start the track transaction: `changeguard ledger start trackGF1 --category REFACTOR --message "Impact packet domain type split"`.
- [x] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [x] Run `changeguard search "ImpactPacket" --auto-index` and record major call-site groups.
- [x] Inventory every `pub struct`/`pub enum` in `packet.rs` and assign each a destination module from the spec.
- [x] Add the schema-stability golden test (plain `serde_json` assertion over a fully-populated packet) BEFORE moving any types.
- [x] Run `cargo test impact::packet` and record the baseline.
- [x] Run `cargo check --all-targets --all-features` and record the baseline.

Definition of done: The implementer knows the active packet call sites, baseline tests, and current risk signals, and a schema characterization test exists before moving code.

## Phase 1: Public Facade and Module Skeleton

- [x] Create the packet module tree.
- [x] Move one low-risk type group first, such as verification result types.
- [x] Re-export moved names from `src/impact/packet.rs`.
- [x] Run `cargo check --all-targets --all-features`.
- [x] Commit the mechanical move if the repo workflow requires frequent commits.

Definition of done: The first moved group proves the facade pattern without changing behavior.

## Phase 2: Domain Type Moves

- [x] Move core packet metadata and schema constants.
- [x] Move changed-file types and helpers.
- [x] Move risk and coupling types.
- [x] Move verification types.
- [x] Move coverage, observability, contracts, deploy, dependency, and security types.
- [x] After each group, run `cargo check --all-targets --all-features`.

Definition of done: `packet.rs` no longer carries unrelated domain type definitions.

## Phase 3: Test Relocation and Schema Protection

- [x] Relocate focused tests next to the domain modules.
- [x] Extend the Phase 0 schema-stability test if new representative fields surfaced during moves.
- [x] Add a compatibility test that imports key names through `crate::impact::packet::*`.
- [x] Run `cargo test impact::packet`.

Definition of done: Tests protect both the new module layout and the old public import path.

## Phase 4: Final Verification

- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo nextest run --lib --bins --workspace`.
- [x] Run `cargo nextest run --test integration`.
- [x] Run `changeguard verify`.
- [x] Run `cargo install --path .`.
- [x] Commit the track transaction: `changeguard ledger commit <tx-id> --summary "Completed Track GF1" --reason "<why>"`. If the git pre-commit hook removed the sidecar and status still shows 1 pending after the git commit, run `ledger commit` immediately.
- [x] Run `changeguard ledger status --compact` and confirm `0 pending, 0 unaudited drift`.

Definition of done: Full gates pass, installed binary matches source, and the ledger is clean.