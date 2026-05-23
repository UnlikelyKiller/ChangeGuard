use crate::index::data_models::{ExtractedModel, ModelKind};
use crate::index::symbols::Symbol;
use miette::{IntoDiagnostic, Result};
use tree_sitter::{Parser, Node};

pub fn extract_data_models(content: &str, _path: &str, _symbols: &[Symbol]) -> Result<Vec<ExtractedModel>> {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse Rust content"))?;

    let mut models = Vec::new();
    collect_rust_models(tree.root_node(), content, &mut models);
    Ok(models)
}

fn collect_rust_models(node: Node, content: &str, models: &mut Vec<ExtractedModel>) {
    let kind = node.kind();

    if kind == "struct_item" || kind == "enum_item" {
        let text = node.utf8_text(content.as_bytes()).unwrap_or("");
        
        let mut is_model = false;
        let model_kind = ModelKind::Struct;

        // Check for common DB traits in derive or attributes
        if text.contains("Serialize") || text.contains("Deserialize") || 
           text.contains("FromRow") || text.contains("Entity") || 
           text.contains("Table") || text.contains("Queryable") ||
           text.contains("Insertable")
        {
            is_model = true;
        }

        if is_model {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node.utf8_text(content.as_bytes()).unwrap_or("").to_string();
                models.push(ExtractedModel {
                    model_name: name,
                    language: "Rust".to_string(),
                    model_kind: if kind == "enum_item" { ModelKind::Schema } else { model_kind },
                    confidence: 0.9,
                    evidence: "Detected based on derive traits and keywords".to_string(),
                });
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_rust_models(child, content, models);
    }
}
