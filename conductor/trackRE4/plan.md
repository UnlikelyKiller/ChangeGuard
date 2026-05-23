# Plan: Track RE4 (Plugin-ize `src/docs/generator.rs`)

- [ ] 1. Define the `DocExporter` trait in `src/docs/mod.rs` or `generator.rs`.
- [ ] 2. Create the directory `src/docs/exporters/`.
- [ ] 3. Create sub-modules for Mermaid, Markdown, JSON, and other formats.
- [ ] 4. Move the specific report generation logic and Datalog query templates into these modules.
- [ ] 5. Implement an `ExporterRegistry` to manage the lifecycle of these plugins.
- [ ] 6. Refactor the main `execute_export` function to use the registry.
- [ ] 7. Update integration tests to verify all export paths.
