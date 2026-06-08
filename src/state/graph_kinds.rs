use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum NodeKind {
    Symbol,
    Endpoint,
    Service,
    DataModel,
    Migration,
    ConfigKey,
    DeploySurface,
    CiWorkflow,
    CiJob,
    CiEnvironment,
    CiArtifact,
    Dependency,
    Test,
    ObservabilitySignal,
    Adr,
    LedgerTransaction,
    Hotspot,
    TemporalCoupling,
    SecurityBoundary,
    Table,
    Column,
    File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum EdgeKind {
    Owns,
    Handles,
    Calls,
    Covers,
    Governs,
    Supersedes,
    Deploys,
    DependsOn,
    Emits,
    AlertsOn,
    ChangedWith,
    Validates,
    Verifies,
    Authenticates,
    Authorizes,
    Consumes,
    TouchesSecret,
    Migrates,
    MapsTo,
}
