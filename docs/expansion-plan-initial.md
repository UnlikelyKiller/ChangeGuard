# ChangeGuard Expansion Plan: System-Wide Architectural Intelligence

This document outlines the roadmap for expanding ChangeGuard from a **Transactional Change Intelligence** tool (v1) into a comprehensive **System-Wide Architectural Intelligence** engine. 

The goal is to bridge the gap between "what code changed" and "how the system functions," allowing ChangeGuard to serve as a persistent "World Model" for both human developers and AI agents.

---

## 1. Vision: From Transactional Delta to System World Model

The core intention of this expansion is to solve the **"Context Cold Start"** problem. Currently, ChangeGuard can tell you *what* changed at a code level, but it cannot tell you *why* that change matters to the overall architecture or how it will behave at runtime.

By building a multi-layered System Intelligence engine, we move ChangeGuard from a "Safety Gate" to a **"Digital Twin"** of the codebase. This allows:
- **Agents** to onboard to unfamiliar repositories in seconds by querying the "Structural Layer."
- **Developers** to understand the "Blast Radius" of a change across APIs and Data Models (the "Behavioral Layer").
- **Operations** to see if a code change maintains production visibility (the "Observability Layer").
- **Teams** to ensure that every change adheres to the project's actual deployment rules (the "Safety Layer").

---

## 2. The Four-Layer Intelligence Model

### 2.1 Phase E1: The Structural Layer (Bearings)
**Intent**: *Eliminate the "Blind Dive."* When an agent or new developer enters a repo, they usually spend the first 30 minutes reading READMEs and clicking folders. This phase automates that "orientation" by building a deterministic map of where things are and what they are for.
- **Track E1-1: README & Documentation Ingestion**: Parse `README.md` to ground technical analysis in the project's stated mission.
- **Track E1-2: Directory & Module Topology**: Label folders (e.g., "This is the Infrastructure layer") based on contents and naming conventions.
- **Track E1-3: Dependency & Tech-Stack Mapping**: Identify the "Gravity" of the project—which libraries dictate the architecture.
- **Track E1-4: Entry Point Identification**: Trace the "First Breath" of the application to understand its initialization sequence.

### 2.2 Phase E2: The Behavioral Layer (Mechanics)
**Intent**: *Understand the "Soul" of the Logic.* Code isn't just lines; it's a series of handlers and data transformations. This layer maps the "Surface Area" (APIs) to the "Internal Organs" (Business Logic).
- **Track E2-1: Framework-Aware Routing**: Map external entry points (URLs/Handlers) to internal functions.
- **Track E2-2: Data Model/Entity Extraction**: Track the "Source of Truth" for data and how it flows through the system.
- **Track E2-3: Critical Path Analysis**: Use centrality metrics to identify "Hot" functions that, if broken, take down the entire system.

### 2.3 Phase E3: The Observability Layer (Visibility)
**Intent**: *Ensure the System is "Talkative."* A common failure in modern dev is changing logic but forgetting to update the logs or traces. This layer treats Observability as a first-class architectural citizen.
- **Track E3-1: Logging & Event Topology**: Ensure every "Critical Path" has an associated "Voice" (logging).
- **Track E3-2: Error Handling Patterns**: Map the "Safety Nets" of the system.
- **Track E3-3: Telemetry & Trace Wiring**: Link static code paths to their runtime "breadcrumbs."

### 2.4 Phase E4: The Safety Layer (Guardrails)
**Intent**: *Codify the "Definition of Done."* Verification shouldn't be a suggestion; it should be a requirement based on the project's own CI/CD and test rules.
- **Track E4-1: Test-Implementation Mapping**: Automate "Targeted Verification" by knowing exactly which test guards which symbol.
- **Track E4-2: CI/CD Workflow Awareness**: Ingest the project's "Laws" from GitHub Actions/Jenkins.
- **Track E4-3: Environment & Secret Schema**: Flag changes that create new infrastructure dependencies (e.g., a new env var).
- **Track E4-3: Environment & Secret Schema**: Extract requirements from `.env.example` or config schemas to flag changes that impact infrastructure requirements.

---

## 2. Technical Architecture Updates

To support these four layers, the following internal changes are required:

### 2.1 The Relational-Graph Schema
The current `ledger.db` schema must be expanded from a flat transaction list to a **Relational Graph**.
- **Nodes**: Symbols, Files, Transactions, Decisions, Routes, Tests, CI Gates.
- **Edges**: `DEFINES`, `CALLS`, `TESTS`, `LOGS`, `IMPLEMENTS`, `DEPENDS_ON`.

### 2.2 Extended Tree-Sitter Queries
Current symbol extraction is generic. Phase E requires **Intent-Specific Queries**:
- `(call_expression ...) @logging` where function name matches known logging crates.
- `(attribute ...) @route` where attribute matches routing macros.
- `(struct_item ...) @model` where struct is in a directory labeled as "entities."

### 2.3 Local-First Semantic Indexing
While maintaining the "No Required Python/Cloud" principle, we will introduce lightweight embedding support (via a Rust-native library like `candle` or `ort`) to allow semantic linking between documentation (README) and implementation (Code).

---

## 3. Implementation Tracks

| ID | Title | Description | Dependency |
|---|---|---|---|
| **T32** | Graph Schema Migration | Update `ledger.db` to support node/edge relationships. | L7 |
| **T33** | Intent-Aware Extraction | Update `index/symbols.rs` with multi-layer TS queries. | T32 |
| **T34** | Doc-Code Alignment | Implement the E1-1 README/Doc ingestion logic. | T33 |
| **T35** | Behavioral Mapping | Implement Route and Data Model identification. | T34 |
| **T36** | Observability Mapping | Implement Logging and Error pattern indexing. | T35 |
| **T37** | Safety Context Ingestion | Integrate CI/CD and Env Var awareness. | T36 |

---

## 4. Non-Negotiables for Expansion

1. **Local-First Always**: All graph analysis and indexing must happen on the user's machine.
2. **Deterministic Over Speculative**: If a Route cannot be identified with high confidence, label it as "POTENTIAL_ROUTE" rather than guessing.
3. **Rust CLI Core**: The expansion remains within the single `changeguard` binary.
4. **Graceful Degradation**: If a repository lacks a README or Tests, ChangeGuard must still function for the remaining layers.
