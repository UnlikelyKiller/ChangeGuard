**Findings**
1. High: the `ai-brains recall` timeout is not a real process timeout. In [src/bridge/client/client_cli.rs](C:/dev/ChangeGuard/src/bridge/client/client_cli.rs:12), the code runs `cmd.output()` on a spawned thread and only times out the channel wait at line 19. If `ai-brains` hangs, the child process is not killed and the worker thread stays blocked, so requirement (1) is not actually satisfied. The existing test only checks overall command success, not child termination or elapsed time: [tests/bridge_query_tests.rs](C:/dev/ChangeGuard/tests/bridge_query_tests.rs:16).

2. High: named-pipe timeout handling still leaks blocked threads and has unbounded I/O paths. [src/bridge/ipc.rs](C:/dev/ChangeGuard/src/bridge/ipc.rs:16) wraps blocking `open()` in a detached thread and times out only the receiver at line 27; a stuck `open()` keeps running after the caller returns. On top of that, `receive_records()` explicitly notes it “might block” and has no timeout at [src/bridge/ipc.rs](C:/dev/ChangeGuard/src/bridge/ipc.rs:42), and `push_verify_results()` adds another fire-and-forget thread that can block in `send_record()` with no write deadline at [src/bridge/notify.rs](C:/dev/ChangeGuard/src/bridge/notify.rs:13). That fails (2) and leaves a real leak/hang vector.

3. High: the v0.2 schema is not strictly enforced. [src/bridge/model.rs](C:/dev/ChangeGuard/src/bridge/model.rs:52) warns on explicit version mismatch, but then still does best-effort parsing at line 64. Records with a missing `version` are accepted silently, and extra fields are ignored by Serde. That is “warn and continue,” not “strictly enforced with warnings,” so requirement (3) is only partially met. Tests cover only the happy-path `0.2` round trip: [tests/bridge_tests.rs](C:/dev/ChangeGuard/tests/bridge_tests.rs:4).

4. Medium: dual retrieval in `ask` is wired, but the blending/truncation logic is wrong. `ask` injects AI-Brains insights and relevant ADRs into `user_prompt` first at [src/commands/ask.rs](C:/dev/ChangeGuard/src/commands/ask.rs:202), then the local backend prepends a second packet-derived summary via `pruned_text` at lines 243-267, so decisions/contracts/observability can be duplicated. The local context budget only trims retrieved chunks in [src/local_model/context.rs](C:/dev/ChangeGuard/src/local_model/context.rs:20); it does not trim the duplicated packet/doc blocks. For Gemini, `truncate_for_context()` is called only after the prompt has already been built from the full packet at [src/commands/ask.rs](C:/dev/ChangeGuard/src/commands/ask.rs:302), so truncation does not actually reduce the submitted bridge/doc context.

5. Medium: the IPC recall path is effectively unimplemented. [src/bridge/client.rs](C:/dev/ChangeGuard/src/bridge/client.rs:9) probes the pipe, discards the client, and always falls back to CLI at line 19. So the system does not currently “prefer IPC” for query retrieval in any meaningful way, and the named-pipe robustness work mostly applies only to verify notifications.

**Assessment**
(1) Process timeouts: failed. The CLI call returns in 5s, but the subprocess is not actually bounded or cancelled.

(2) Named pipe robustness / no leaked threads: failed. Connect timeout and notify both rely on abandoned threads; read/write paths are still blocking.

(3) v0.2 schema strictness: failed. Version mismatch warns, but parsing remains best-effort and missing version is accepted.

(4) Dual retrieval blend/truncate: partial. Retrieval is integrated, but prompt assembly duplicates context and Gemini truncation is applied too late to matter.

(5) DTO decoupling: passed. `BridgeVerifyOutcome` is correctly defined in [src/bridge/model.rs](C:/dev/ChangeGuard/src/bridge/model.rs:26) and mapped from verify results in [src/commands/verify.rs](C:/dev/ChangeGuard/src/commands/verify.rs:415) before notify uses it in [src/bridge/notify.rs](C:/dev/ChangeGuard/src/bridge/notify.rs:6).

(6) Fail-open behavior: mostly passed, with caveats. Query/import/ask degrade to warnings or empty results, but the bridge still has silent-drop and resource-leak risks around IPC notify/write paths.

I could not run the test suite or `changeguard` verification commands in this sandbox, so this review is based on static inspection only. The highest-risk remaining gaps are the fake process timeout, orphaned IPC/open threads, blocking pipe I/O, and ineffective prompt truncation.