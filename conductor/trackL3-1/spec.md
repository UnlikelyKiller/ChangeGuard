# Specification: Track L3-1 - Enforcement Data Model & Registration

## Objective
Implement the foundational data model, types, and CLI commands for Phase L3 (Tech Stack Enforcement & Validators) as outlined in `docs/Ledger-Incorp-plan.md`. This enables users to register and view tech stack rules and commit validators, preparing the system for active constraint enforcement at transaction start and commit times.

## Deliverables

### 1. `src/ledger/enforcement.rs`
Define the types and structures for the enforcement data model:

- `RuleType`: Enum for `TECH_STACK`, `VALIDATOR`, `MAPPING`, `WATCHER`.
- `TechStackRule`: Represents an entry in the `tech_stack` table.
- `ValidationLevel`: Enum `Error`, `Warning` (default `Error`).
- `CommitValidator`: Represents an entry in the `commit_validators` table.
- `CategoryMapping`: Represents an entry in the `category_stack_mappings` table.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuleType {
    TechStack,
    Validator,
    Mapping,
    Watcher,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStackRule {
    pub category: String, // e.g., DATABASE, BACKEND_LANG
    pub name: String,
    pub version_constraint: Option<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub locked: bool,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default = "default_entity_type")]
    pub entity_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitValidator {
    pub category: String, // e.g., FEATURE, ARCHITECTURE
    pub name: String,
    pub description: Option<String>,
    pub executable: String,
    pub args: Vec<String>,
    #[serde(default = "default_timeout")]
    pub timeout_ms: i32,
    pub glob: Option<String>,
    #[serde(default = "default_validation_level")]
    pub validation_level: ValidationLevel,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryMappingPayload {
    pub ledger_category: String,
    pub stack_category: String,
    pub glob: Option<String>,
    pub description: Option<String>,
}
```

### 2. `src/state/migrations.rs`
Add `M13` to `get_migrations()` containing the following table schemas:
- `tech_stack`
- `commit_validators`
- `category_stack_mappings`
- `watcher_patterns`

```sql
CREATE TABLE IF NOT EXISTS tech_stack (
    category           TEXT PRIMARY KEY,
    name               TEXT NOT NULL,
    version_constraint TEXT,
    rules              TEXT NOT NULL DEFAULT '[]',
    locked             INTEGER DEFAULT 0,
    status             TEXT DEFAULT 'ACTIVE',
    entity_type        TEXT DEFAULT 'FILE',
    registered_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS commit_validators (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    category           TEXT NOT NULL,
    name               TEXT NOT NULL,
    description        TEXT,
    executable         TEXT NOT NULL,
    args               TEXT NOT NULL,
    timeout_ms         INTEGER DEFAULT 30000,
    glob               TEXT,
    validation_level   TEXT DEFAULT 'ERROR',
    enabled            INTEGER DEFAULT 1
);

CREATE TABLE IF NOT EXISTS category_stack_mappings (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    ledger_category    TEXT NOT NULL,
    stack_category     TEXT NOT NULL REFERENCES tech_stack(category),
    glob               TEXT,
    description        TEXT
);

CREATE TABLE IF NOT EXISTS watcher_patterns (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    glob               TEXT NOT NULL,
    category           TEXT NOT NULL,
    source             TEXT NOT NULL DEFAULT 'CONFIG',
    description        TEXT
);
```

### 3. `src/ledger/db.rs`
Implement helper methods to handle inserting and fetching rules from the database:
- `pub fn insert_tech_stack_rule(&self, rule: &TechStackRule) -> Result<()>`
- `pub fn get_tech_stack_rules(&self, category: Option<&str>) -> Result<Vec<TechStackRule>>`
- `pub fn insert_commit_validator(&self, validator: &CommitValidator) -> Result<()>`
- `pub fn get_commit_validators(&self, category: Option<&str>) -> Result<Vec<CommitValidator>>`
- `pub fn insert_category_mapping(&self, mapping: &CategoryMappingPayload) -> Result<()>`

### 4. `src/cli.rs` and CLI commands
Extend `LedgerCommands` with:
- `Register`: Takes `--rule-type` (enum) and `--payload` (JSON string).
- `Stack`: Takes optional `--category` string to filter the output.

Add handlers for the CLI:
- `src/commands/ledger_register.rs`: Parses the JSON string into the appropriate struct based on the `RuleType`, validates it, and saves it via `ledger::db.rs`.
- `src/commands/ledger_stack.rs`: Fetches the rules via `ledger::db.rs` and formats them in a clear human-readable output or JSON.

### 5. Tests
- Update `tests/ledger_enforcement.rs` or unit tests inside `src/ledger/db.rs` and `src/commands/ledger_register.rs` following TDD.
- Verify basic schema instantiation, insertion of different JSON payloads, and reading back values.
