## Plan: Config and Rule Loading

### Phase 1: Models and Error Handling
- [ ] Task 1.1: Define `miette`-based error types for config and policy loading in `src/config/error.rs` and `src/policy/error.rs` using `thiserror`.
- [ ] Task 1.2: Implement the `Config` struct and its nested types (`CoreConfig`, `WatchConfig`, etc.) in `src/config/model.rs` using `serde`.
- [ ] Task 1.3: Implement the `Mode` enum and the `Rules`, `PathRule` structs in `src/policy/mode.rs` and `src/policy/rules.rs`.
- [ ] Task 1.4: Write unit tests verifying default serialization/deserialization logic.
- [ ] Task 1.5: Verify Phase 1: `cargo test -j 1 --package changeguard -- test_models` (or run relevant model tests).

### Phase 2: Defaults and Loader Implementation
- [ ] Task 2.1: Implement default fallback instances and string constants in `src/config/defaults.rs` and `src/policy/defaults.rs`.
- [ ] Task 2.2: Implement `load_config()` in `src/config/load.rs` to discover and parse `config.toml` from `.changeguard/`, falling back to defaults if absent.
- [ ] Task 2.3: Implement `load_rules()` in `src/policy/load.rs` to read `rules.toml`.
- [ ] Task 2.4: Write T-D-D fixture tests simulating missing configurations and malformed TOML files to ensure `miette` error propagation works correctly.
- [ ] Task 2.5: Verify Phase 2: `cargo test -j 1 -- test_loading` or relevant loading tests.

### Phase 3: Validation Logic
- [ ] Task 3.1: Implement `validate_config()` in `src/config/validate.rs`.
- [ ] Task 3.2: Implement `validate_rules()` in `src/policy/validate.rs`. Ensure that required verifications and mode definitions don't conflict, and validate all glob patterns.
- [ ] Task 3.3: Write unit tests feeding valid and invalid configurations (e.g., an invalid glob pattern in a PathRule) to the validators.
- [ ] Task 3.4: Verify Phase 3: `cargo test -j 1 -- test_validation` or relevant validation tests.

### Phase 4: Rule Matcher and Protected Paths
- [ ] Task 4.1: Implement `RuleMatcher` in `src/policy/matching.rs`. Use the `globset` crate to match given file paths against `PathRule` patterns.
- [ ] Task 4.2: Add logic to `RuleMatcher` to determine the active `Mode` and merge `required_verifications` for a specific changed file.
- [ ] Task 4.3: Implement protected paths checking in `src/policy/protected_paths.rs` to escalate risk when protected paths are touched.
- [ ] Task 4.4: Write test cases testing path-based rule overrides, ensuring precedence is respected and protected paths are flagged correctly.
- [ ] Task 4.5: Verify Phase 4: `cargo test -j 1 -- test_matching` or relevant matching tests.
