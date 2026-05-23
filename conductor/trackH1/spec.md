# Track H1: Semantic Engine Audit

## Objective
Fix the mathematical instability in semantic search resulting in `NaN` distances and calibrate BM25 scoring for small corpora.

## Requirements
- **NaN Fix**: Identify the root cause of `NaN` distances in `search --semantic`. This is likely a division-by-zero in cosine similarity when a vector has zero magnitude or an issue with the Nomic embedding model's normalization.
- **BM25 Calibration**: Adjust the BM25 parameters (k1, b) or the result normalization to ensure that exact matches in small projects (like the test repos) return meaningful, non-zero scores.
- **Stability Tests**: Add unit tests in `src/semantic/` that specifically test similarity math with edge-case vectors.

## Definition of Done (DoD)
- [ ] `changeguard search --semantic` returns valid floating-point distances for all matches.
- [ ] Keyword search scores for exact matches are positive and properly ranked.
- [ ] New unit tests for vector similarity math pass.
