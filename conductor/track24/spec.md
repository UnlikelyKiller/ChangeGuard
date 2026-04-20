# Track 24 Spec: Complexity Indexing

## Objective
Measure cognitive and cyclomatic complexity for functions and structs to weight impact risks. This data will be combined with temporal data to identify hotspots.

## Deliverables
- `src/index/metrics.rs`: Complexity computation logic.
- `ComplexityScorer` trait to allow swappable implementations.
- Spike evaluation of `arborist-metrics` (0.1.2).
- Fallback native tree-sitter implementation if the spike fails.

## Functional Requirements
1. **Complexity Metrics**:
   - Cognitive Complexity (per SonarSource specification).
   - Cyclomatic Complexity (McCabe).
   - Source Lines of Code (SLOC).
2. **Language Support**:
   - Rust, TypeScript, Python (matching current parser coverage).
3. **Graceful Degradation**:
   - Files with syntax errors should return partial metrics with `ast_incomplete: true`.
   - Unsupported languages should return `Complexity::NotApplicable`.
4. **Determinism**:
   - Same source file must always produce the same complexity scores.
5. **Storage**:
   - Update `Symbol` metadata or create a new `ComplexityRecord` in SQLite to persist scores.

## Internal API
```rust
pub struct FileComplexity {
    pub path: Utf8PathBuf,
    pub functions: Vec<SymbolComplexity>,
    pub total_sloc: usize,
}

pub struct SymbolComplexity {
    pub symbol_name: String,
    pub cognitive: usize,
    pub cyclomatic: usize,
}

pub trait ComplexityScorer {
    fn score_file(&self, path: &Utf8Path, source: &str, language: Language) -> Result<FileComplexity>;
}
```

## Dependencies
- `tree-sitter` (0.26.8)
- `arborist-metrics = "0.1.2"` (Spike candidate)
- `miette`
