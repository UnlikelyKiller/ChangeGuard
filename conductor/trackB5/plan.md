## Plan: Track B5 - Named Pipe IPC Integration
### Phase 1: Synchronous IPC Client
- [x] Task 1.1: Create `src/bridge/ipc.rs` defining the `IpcClient`.
- [x] Task 1.2: Implement `connect_with_timeout` utilizing `std::thread::spawn` and `mpsc::channel` to strictly bound `std::fs::File::open` hang times.
- [x] Task 1.3: Implement `send_record(&BridgeRecord)` and `receive_records()` via the acquired pipe.
- [x] Task 1.4: Integrate `IpcClient` into `src/bridge/client.rs` as the primary query mechanism before delegating to `query_external_cli`.
- [x] Task 1.5: Add tests for robust timeout behavior and fail-open downgrade logic.
