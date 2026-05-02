use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Struct,
    Enum,
    Trait,
    Interface,
    Type,
    Variable,
    Constant,
    Module,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub is_public: bool,
    pub cognitive_complexity: Option<i32>,
    pub cyclomatic_complexity: Option<i32>,
    #[serde(default)]
    pub line_start: Option<i32>,
    #[serde(default)]
    pub line_end: Option<i32>,
    #[serde(default)]
    pub qualified_name: Option<String>,
    #[serde(default)]
    pub byte_start: Option<i32>,
    #[serde(default)]
    pub byte_end: Option<i32>,
    #[serde(default)]
    pub entrypoint_kind: Option<String>,
}
