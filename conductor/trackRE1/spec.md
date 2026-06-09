# Track RE1: Decompose `src/commands/verify.rs`

## Objective
Reduce the cognitive complexity of `src/commands/verify.rs` (currently 224) by decomposing it into specialized components: `VerifyEngine`, `OutcomePredictor`, and `VerificationReporter`.

## Requirements
- **VerifyEngine**: Extract the core execution logic for running local tests and CI commands.
- **OutcomePredictor**: Move the Bayesian/Historical blending logic to `src/verify/predictor.rs`.
- **VerificationReporter**: Extract the TUI/CLI output formatting logic into `src/output/verification.rs`.
- **State Management**: Use a clean `VerificationContext` struct to pass state between components.

## Definition of Done (DoD)
- [ ] Cognitive complexity of `src/commands/verify.rs` is reduced below 50.
- [ ] No regression in `changeguard verify` behavior.
- [ ] All 897 tests (and any new ones) pass.
- [ ] The file `src/commands/verify.rs` primarily acts as a CLI entry point/orchestrator.
