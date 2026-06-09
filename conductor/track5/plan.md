## Plan: Track 5 - Basic Impact Packet Shell

### Phase 1: Core Domain and Serialization
- [ ] Task 1.1: Create `src/impact/mod.rs` and `src/impact/packet.rs`. Add module to `src/lib.rs` or `src/main.rs`.
- [ ] Task 1.2: Define `RiskLevel`, `ChangedFile`, and `ImpactPacket` structs in `packet.rs` with `serde` derives.
- [ ] Task 1.3: Add a unit test in `packet.rs` that builds a sample `ImpactPacket` and asserts the serialized JSON structure matches expected schema conventions (camelCase, version included). Run `cargo test -j 1`.

### Phase 2: State Persistence Integration
- [ ] Task 2.1: Create `src/state/reports.rs`. Add module to `src/state/mod.rs`.
- [ ] Task 2.2: Implement `write_impact_report(layout: &Layout, packet: &ImpactPacket) -> miette::Result<()>` inside `reports.rs`.
- [ ] Task 2.3: Add a unit test in `reports.rs` using `tempfile` to verify that `latest-impact.json` is successfully written and contains the expected content. Run `cargo test -j 1`.

### Phase 3: CLI Command Wiring
- [ ] Task 3.1: Create `src/commands/impact.rs`. Add module to `src/commands/mod.rs`.
- [ ] Task 3.2: Implement `execute_impact()` which reads current git status, generates an `ImpactPacket`, and saves it using `write_impact_report`.
- [ ] Task 3.3: Wire the `Commands::Impact` enum variant in `src/cli.rs` to call `crate::commands::impact::execute_impact()`.
- [ ] Task 3.4: Verify the project builds via `cargo build`.

### Phase 4: Integration Testing
- [ ] Task 4.1: Create `tests/cli_impact.rs`.
- [ ] Task 4.2: Implement an integration test using a temporary git repo that creates a commit, modifies a file, and runs `changeguard impact`.
- [ ] Task 4.3: Assert that the command succeeds and `latest-impact.json` is generated correctly in `.changeguard/reports/`.
- [ ] Task 4.4: Ensure all tests pass with `cargo test -j 1`.

### Phase 5: Linting and Code Quality
- [ ] Task 5.1: Run `cargo clippy --all-targets --all-features` and address any warnings.
- [ ] Task 5.2: Run `cargo fmt --check` to verify code style.
