use crate::index::symbols::Symbol;

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
