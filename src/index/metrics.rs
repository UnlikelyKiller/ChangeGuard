use crate::index::languages::Language;
use camino::Utf8Path;
use miette::Result;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FileComplexity {
    pub total_sloc: usize,
    pub functions: Vec<SymbolComplexity>,
    pub ast_incomplete: bool,
    pub complexity_capped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolComplexity {
    pub name: String,
    pub cognitive: usize,
    pub cyclomatic: usize,
}

pub trait ComplexityScorer {
    fn score_file(
        &self,
        path: &Utf8Path,
        source: &str,
        language: Language,
    ) -> Result<FileComplexity>;
}

pub struct NativeComplexityScorer;

impl Default for NativeComplexityScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeComplexityScorer {
    pub fn new() -> Self {
        Self
    }

    fn calculate_cyclomatic(&self, node: Node) -> usize {
        let mut complexity = 1; // Base complexity
        let mut cursor = node.walk();
        let mut stack = vec![node];

        while let Some(current) = stack.pop() {
            let kind = current.kind();

            // Branching points that increase cyclomatic complexity
            if matches!(
                kind,
                "if_statement"
                    | "if_expression"
                    | "for_statement"
                    | "for_expression"
                    | "while_statement"
                    | "while_expression"
                    | "loop_expression"
                    | "match_arm"
                    | "case_item"
                    | "&&"
                    | "||"
                    | "and"
                    | "or"
                    | "ternary_expression"
                    | "conditional_expression"
                    | "binary_expression" if matches!(current.child_by_field_name("operator").map(|n| n.kind()), Some("&&" | "||"))
            ) {
                complexity += 1;
            }

            for child in current.children(&mut cursor) {
                stack.push(child);
            }
        }

        complexity
    }

    fn calculate_cognitive(&self, node: Node) -> usize {
        self.calculate_cognitive_recursive(node, 0).0
    }

    fn calculate_cognitive_recursive(&self, node: Node, nesting: usize) -> (usize, usize) {
        let mut score = 0;
        let kind = node.kind();
        let mut current_nesting = nesting;

        let is_nesting_increment = matches!(
            kind,
            "if_statement"
                | "if_expression"
                | "for_statement"
                | "for_expression"
                | "while_statement"
                | "while_expression"
                | "loop_expression"
                | "match_expression"
                | "switch_statement"
                | "catch_clause"
        );

        if is_nesting_increment {
            score += 1 + nesting;
            current_nesting += 1;
        } else if matches!(kind, "match_arm" | "case_item") {
            // Incremented by nesting but doesn't increment nesting itself usually
            score += nesting;
        } else if matches!(kind, "&&" | "||" | "and" | "or") {
            score += 1;
        } else if kind == "binary_expression" {
             if let Some(op) = node.child_by_field_name("operator") {
                 if matches!(op.kind(), "&&" | "||") {
                     score += 1;
                 }
             }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let (child_score, _) = self.calculate_cognitive_recursive(child, current_nesting);
            score += child_score;
        }

        (score, current_nesting)
    }
}

impl ComplexityScorer for NativeComplexityScorer {
    fn score_file(
        &self,
        _path: &Utf8Path,
        source: &str,
        language: Language,
    ) -> Result<FileComplexity> {
        let total_sloc = source.lines().count();
        let complexity_capped = total_sloc > 10_000;
        
        if complexity_capped {
            return Ok(FileComplexity {
                total_sloc,
                functions: Vec::new(),
                ast_incomplete: false,
                complexity_capped: true,
            });
        }

        let mut parser = tree_sitter::Parser::new();
        let ts_language = match language {
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Language::Python => tree_sitter_python::LANGUAGE.into(),
        };
        parser
            .set_language(&ts_language)
            .map_err(|e| miette::miette!("TS language error: {e}"))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| miette::miette!("Failed to parse source"))?;
        let root = tree.root_node();
        let ast_incomplete = root.has_error();

        let mut functions = Vec::new();
        let mut cursor = root.walk();
        let mut stack = vec![root];

        while let Some(node) = stack.pop() {
            let kind = node.kind();
            if matches!(
                kind,
                "function_item" 
                | "function_definition" 
                | "method_declaration" 
                | "method_definition"
                | "arrow_function"
                | "function_declaration"
                | "generator_function_declaration"
            ) {
                let name = node
                    .child_by_field_name("name")
                    .map(|n| {
                        n.utf8_text(source.as_bytes())
                            .unwrap_or("anonymous")
                            .to_string()
                    })
                    .unwrap_or_else(|| "anonymous".to_string());

                functions.push(SymbolComplexity {
                    name,
                    cognitive: self.calculate_cognitive(node),
                    cyclomatic: self.calculate_cyclomatic(node),
                });
            }

            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }

        Ok(FileComplexity {
            total_sloc,
            functions,
            ast_incomplete,
            complexity_capped,
        })
    }
}
