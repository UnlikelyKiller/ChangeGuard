# Technical Specification: Config and Rule Loading (Phase 4)

## Overview
This specification details the implementation for deterministic configuration and policy loading in Changeguard, fulfilling the Phase 4 objectives of the master plan. It establishes the foundational models, parsing, validation, and matching logic required to apply repository-specific rules.

## Core Components

### 1. Configuration & Policy Models
- **`src/config/model.rs`**: Defines the `Config` structure for `config.toml`. Covers application-wide settings like `core` (e.g., strict mode), `watch` (e.g., debounce intervals), and `gemini` integration parameters. Uses `serde` for serialization.
- **`src/policy/mode.rs`**: Defines the `Mode` enum representing operational strictness (`Analyze`, `Review`, `Suggest`, `Enforce`).
- **`src/policy/rules.rs`**: Defines the `Rules` structure for `rules.toml`. Contains global rules, a vector of `PathRule` objects for path-based overrides, and a list of `protected_paths`.

### 2. Default Values & File Loading
- **`src/config/defaults.rs`**: Provides built-in static default strings and structure defaults for configuration and rules to guarantee a fallback when local files are absent.
- **`src/config/load.rs` & `src/policy/load.rs`**: Handles discovering, reading, and parsing the `config.toml` and `rules.toml` files from the `.changeguard/` state directory. Uses `toml` crate to deserialize.

### 3. Validation Layer
- **`src/config/validate.rs`**: Validates the consistency of the parsed configurations.
- **`src/policy/validate.rs`**: Ensures `PathRule` objects contain valid `globset` patterns and that required verifications have no logical contradictions.

### 4. Rule Matcher
- **`src/policy/matching.rs`**: Evaluates which rules and modes apply to a specific changed file path. Supports path-based overrides by testing paths against the compiled `globset` from the `PathRule` definitions. Determines the union of `required_verifications`.

### 5. Protected Paths
- **`src/policy/protected_paths.rs`**: Evaluates if a given path falls under a configured `protected_path` pattern (e.g., `Cargo.toml`, `.github/workflows/`). Triggers risk escalation.

## Error Handling
- The entire module must use `miette` for rich, human-readable diagnostic errors.
- Custom error types defined in `src/config/error.rs` and `src/policy/error.rs` (using `thiserror`) will include `#[diagnostic]` annotations to point users exactly to malformed TOML structures or invalid glob patterns.

## Testing Strategy
- **T-D-D Focus**: Tests must be written before or alongside the implementation of each component.
- Uses `cargo test -j 1` to ensure deterministic execution of file-system related fixture tests.
