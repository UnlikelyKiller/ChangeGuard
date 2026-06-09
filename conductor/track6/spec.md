# Technical Specification: Watch Mode and Batch Debouncing

## 1. Overview
The watch mode continuously monitors supported source files for changes, ignoring build artifacts, temp files, and the `.changeguard/` directory itself. It relies on `notify-debouncer-full` (0.7.0) for file system events, implementing a robust debouncing strategy to merge related save events and gracefully handle rapid edits and renames, particularly on Windows.

## 2. Dependencies
- `notify-debouncer-full = "0.7.0"`
- `ignore = "0.4.25"` (or `globset` for custom path matching)
- `serde`, `serde_json` for batch persistence

## 3. Core Components
- **Watcher Initialization**: Setup `notify-debouncer-full` with an appropriate debounce timeout (e.g., 500ms) to merge rapid sequential events.
- **Event Filtering**: Discard events outside of supported files or inside ignored directories (e.g., `.changeguard/`, `target/`, `node_modules/`, editor temp files).
- **Batching Strategy**: Accumulate valid file events and deduplicate them. Handle rename storms, specifically Windows atomic saves which often involve a temporary file rename.
- **Batch Persistence**: Write the currently accumulated batch to `.changeguard/state/current-batch.json` atomically.

## 4. Edge Cases Handled
- **Rapid Edits**: Repeated saves on the same file are collapsed.
- **Atomic Saves (Windows)**: Editors that save via temp + rename (e.g., `file.tmp` -> `file.rs`) are correctly resolved to the final file.
- **Branch Churn**: Large bursts of changes from branch switching are batched efficiently without crashing the watcher.
- **File Deletions**: Files deleted after event capture but before batch processing are safely verified.

## 5. Persistence Schema
```json
{
  "timestamp": "2023-10-27T10:00:00Z",
  "events": [
    { "path": "src/main.rs", "kind": "Modify" },
    { "path": "src/watch/debounce.rs", "kind": "Create" }
  ]
}
```

## 6. Error Handling
- All internal watcher errors must be mapped to idiomatic `miette::Diagnostic` error types.
- The process should tolerate temporary unreachability of files or recoverable IO errors during rapid file manipulation.
