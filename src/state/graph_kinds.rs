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
    CiJob,
    Dependency,
    Test,
    ObservabilitySignal,
    Adr,
    LedgerTransaction,
    Hotspot,
    TemporalCoupling,
    SecurityBoundary,
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
    Authenticates,
    Authorizes,
    Consumes,
    TouchesSecret,
}
