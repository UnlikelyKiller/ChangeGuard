# Specification: AI-Brains Domain Schema & Cross-Domain Reachability (Track C2)

## Overview
Extend ChangeGuard's CozoDB schema with AI-Brains domain relations (`Turn`, `Session`, `Memory`, `Decision`) and define cross-domain reachability queries that link conversations to the AST nodes they modified.

## Architecture & SRP
- **Module**: `src/state/storage_cozo.rs`
- **Responsibility**: Own the CozoDB schema; this track adds AI-Brains domain tables and cross-domain query logic without modifying existing ChangeGuard relations.

## Requirements
- Define 4 new CozoDB relations: `Turn` (conversation turns), `Session` (grouped turns), `Memory` (derived memories), `Decision` (architectural decisions).
- Define cross-domain reachability queries that, given a Turn/Decision, traverse through `Memory` → modified symbols → `node`/`edge` in the existing KG to return the AST nodes affected by that conversation.
- Define reverse queries: given an AST node in the existing KG, find all conversations (Turns/Sessions) that discussed or modified it.
- All new relations must coexist with existing Cozo relations without breaking existing queries.
- Use Datalog rules (not imperative code) for cross-domain traversals so they are queryable via CozoDB's native engine.
- Schema must be idempotent — `setup_schema()` can be called multiple times safely.

## Dependencies
- Track C1 must be complete (needs structured BridgeRecord types for IPC transport of graph mutations from AI-Brains).
