# Specification: Unified Retrieval in `ask` (Track B6)

## Overview
Enhance `changeguard ask` by automatically blending internal CozoDB/Tantivy knowledge with external AI-Brains memories during context assembly.

## Architecture & SRP
- **Module**: `src/commands/ask.rs` and `src/gemini/mod.rs`
- **Responsibility**: Safely orchestrate dual-retrieval and cleanly construct the unified Gemini prompt.

## Requirements
- When `ask` is invoked, concurrently (or sequentially with timeouts) execute internal queries and `bridge::client::query_unified()`.
- Collate CozoDB architecture responses alongside returned `Insight` records.
- Format external `Insight` contents into a dedicated `### External AI-Brains Context` Markdown block within the final prompt payload.
- Ensure the combined context respects token truncation limits.
- **Fail-open**: If the external `bridge` query fails, log it and proceed using purely local CozoDB context.
