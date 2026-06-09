# Track RE4: Plugin-ize `src/docs/generator.rs`

## Objective
Convert the monolithic report factory into a trait-based system where each export format (Mermaid, Markdown, JSON, etc.) is its own plugin.

## Requirements
- **Exporter Trait**: Define a `DocExporter` trait.
- **Export Separation**: Move logic for each of the 13+ formats into dedicated modules in `src/docs/exporters/`.
- **Registry**: Use a registry to map format strings to exporter implementations.

## Definition of Done (DoD)
- [ ] `src/docs/generator.rs` is reduced to < 300 lines (coordination and common utils only).
- [ ] Adding a new documentation format only requires adding a new file in `exporters/`.
- [ ] `changeguard index --export-docs` results are verified across all formats.
- [ ] Tests in `tests/doc_generation.rs` pass.
