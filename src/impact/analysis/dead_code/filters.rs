use crate::index::symbols::{Symbol, SymbolKind};

pub(super) fn is_entrypoint(symbol: &Symbol) -> bool {
    matches!(
        symbol.entrypoint_kind.as_deref(),
        Some("ENTRYPOINT")
            | Some("HANDLER")
            | Some("PUBLIC_API")
            | Some("TEST")
            | Some("FFI")
            | Some("MACRO")
    )
}

/// Standard Rust traits that are often used implicitly (via derive macros, blanket
/// impls, or trait objects) and therefore produce frequent false positives in dead-code
/// analysis.
const STANDARD_TRAIT_NAMES: &[&str] = &[
    "Eq",
    "PartialEq",
    "Ord",
    "PartialOrd",
    "Default",
    "Clone",
    "Copy",
    "Debug",
    "Display",
    "Hash",
    "Send",
    "Sync",
    "Serialize",
    "Deserialize",
    "From",
    "Into",
    "AsRef",
    "AsMut",
    "Iterator",
    "IntoIterator",
    "Error",
    "ToString",
    "Sized",
    "Drop",
];

/// Returns `true` when a symbol is a standard trait implementation that should be
/// suppressed by default (i.e. when `--include-traits` is not set).
///
/// The Rust AST extractor records `impl Eq for MyType {}` as `SymbolKind::Type`
/// with the trait name (`"Eq"`) as the symbol name.  `SymbolKind::Trait` is only
/// used for *definitions* of traits (`trait Eq { … }`), which never appear in user
/// code for stdlib traits.  Checking `SymbolKind::Type` therefore correctly catches
/// explicit trait-impl blocks while leaving user-defined structs with the same name
/// untouched (they carry `SymbolKind::Struct`).
pub(super) fn is_standard_trait(symbol: &Symbol) -> bool {
    matches!(symbol.kind, SymbolKind::Type) && STANDARD_TRAIT_NAMES.contains(&symbol.name.as_str())
}

/// Name-based confidence penalty (-0.20) applied to symbols whose names match
/// common patterns for dynamically dispatched or serialized types.
///
/// Such types are typically invoked through trait objects, serde, or dependency
/// injection frameworks and therefore produce false positives when no static
/// call edges exist.
const NAME_PENALTY_SUFFIXES: &[&str] = &["Provider", "Result", "Chunk", "Record"];
const NAME_PENALTY: f64 = 0.20;

/// Returns the penalty to subtract from a symbol's confidence score based on its name.
pub(super) fn name_penalty(symbol_name: &str) -> f64 {
    for suffix in NAME_PENALTY_SUFFIXES {
        if symbol_name.ends_with(suffix) {
            return NAME_PENALTY;
        }
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::symbols::{Symbol, SymbolKind};
    use std::collections::BTreeMap;

    fn make_symbol(name: &str, kind: SymbolKind) -> Symbol {
        Symbol {
            name: name.to_string(),
            kind,
            is_public: false,
            cognitive_complexity: None,
            cyclomatic_complexity: None,
            line_start: None,
            line_end: None,
            qualified_name: None,
            byte_start: None,
            byte_end: None,
            entrypoint_kind: None,
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn test_is_standard_trait_matches_impl_type_kind() {
        // The Rust extractor stores `impl Debug for MyType {}` as
        // (name="Debug", kind=SymbolKind::Type). These are the symbols that
        // actually appear in the dead-code index for explicit trait impls.
        let eq = make_symbol("Eq", SymbolKind::Type);
        assert!(is_standard_trait(&eq));

        let ord = make_symbol("Ord", SymbolKind::Type);
        assert!(is_standard_trait(&ord));

        let debug = make_symbol("Debug", SymbolKind::Type);
        assert!(is_standard_trait(&debug));

        let serialize = make_symbol("Serialize", SymbolKind::Type);
        assert!(is_standard_trait(&serialize));
    }

    #[test]
    fn test_is_standard_trait_requires_type_kind() {
        // Same name but wrong kind must NOT be filtered — structs named Eq are valid
        // user-defined types and must not be suppressed.
        let eq_struct = make_symbol("Eq", SymbolKind::Struct);
        assert!(!is_standard_trait(&eq_struct));

        let eq_fn = make_symbol("Eq", SymbolKind::Function);
        assert!(!is_standard_trait(&eq_fn));

        // SymbolKind::Trait is used for trait *definitions* (trait Eq { }), which are
        // stdlib-only and never appear in user-code indices — the filter doesn't need
        // to handle them.
        let eq_trait_def = make_symbol("Eq", SymbolKind::Trait);
        assert!(!is_standard_trait(&eq_trait_def));
    }

    #[test]
    fn test_is_standard_trait_user_defined_trait_not_filtered() {
        let custom = make_symbol("MyCustomTrait", SymbolKind::Type);
        assert!(!is_standard_trait(&custom));
    }

    #[test]
    fn test_name_penalty_provider_suffix() {
        assert!((name_penalty("CIPredictorProvider") - NAME_PENALTY).abs() < f64::EPSILON);
        assert!((name_penalty("SomeProvider") - NAME_PENALTY).abs() < f64::EPSILON);
    }

    #[test]
    fn test_name_penalty_chunk_suffix() {
        assert!((name_penalty("RetrievedChunk") - NAME_PENALTY).abs() < f64::EPSILON);
    }

    #[test]
    fn test_name_penalty_record_suffix() {
        assert!((name_penalty("BridgeRecord") - NAME_PENALTY).abs() < f64::EPSILON);
    }

    #[test]
    fn test_name_penalty_result_suffix() {
        assert!((name_penalty("SearchResult") - NAME_PENALTY).abs() < f64::EPSILON);
    }

    #[test]
    fn test_name_penalty_no_match() {
        assert_eq!(name_penalty("execute_scan"), 0.0);
        assert_eq!(name_penalty("Config"), 0.0);
        assert_eq!(name_penalty("MyStruct"), 0.0);
    }
}
