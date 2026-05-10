use camino::{Utf8Path, Utf8PathBuf};
use cozo::{DataValue, Num};
use miette::{Diagnostic, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use thiserror::Error;
use tracing::warn;

use crate::state::storage_cozo::CozoStorage;

#[derive(Debug, Error, Diagnostic)]
pub enum DocGenerationError {
    #[error("CozoDB query failed: {0}")]
    QueryFailed(String),
    #[error("I/O error writing doc output")]
    IoFailed(#[from] std::io::Error),
    #[error("Knowledge Graph unavailable")]
    CozoUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileDependency {
    pub source_file: String,
    pub target_file: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolRow {
    pub qualified_name: String,
    pub symbol_name: String,
    pub symbol_kind: String,
    pub file_path: String,
    pub line_start: i64,
    pub line_end: i64,
    pub is_public: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleGroup {
    pub dir: String,
    pub files: Vec<String>,
}

pub trait DocTemplate: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf>;
}

pub struct DocRegistry {
    templates: Vec<Box<dyn DocTemplate>>,
}

impl DocRegistry {
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
        }
    }

    pub fn register(&mut self, template: Box<dyn DocTemplate>) {
        self.templates.push(template);
    }

    pub fn default_registry() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(DependencyGraphTemplate));
        registry.register(Box::new(SymbolTableTemplate));
        registry.register(Box::new(ModuleSummaryTemplate));
        registry.register(Box::new(ModuleMapTemplate));
        registry.register(Box::new(SymbolIndexTemplate));
        registry.register(Box::new(ServiceBoundaryTemplate));
        registry.register(Box::new(ChangeHotspotReportTemplate));
        registry.register(Box::new(SemanticNeighborIndexTemplate));
        registry.register(Box::new(DataFlowDiagramTemplate));
        registry.register(Box::new(TestCoverageGapTemplate));
        registry.register(Box::new(ApiContractIndexTemplate));
        registry.register(Box::new(AdrStalenessReportTemplate));
        registry.register(Box::new(CiPipelineMapTemplate));
        registry.register(Box::new(TokenProvenanceMapTemplate));
        registry.register(Box::new(FederationSummaryTemplate));
        registry.register(Box::new(DependencyHealthTemplate));
        registry.register(Box::new(ObservabilitySignalSnapshotTemplate));
        registry.register(Box::new(CallGraphDetailTemplate));
        registry
    }

    pub fn resolve(&self, name: &str) -> Option<&dyn DocTemplate> {
        self.templates
            .iter()
            .find(|t| t.name() == name)
            .map(|t| t.as_ref())
    }

    pub fn run_all(
        &self,
        storage: &CozoStorage,
        output_dir: &Utf8Path,
    ) -> Result<Vec<Utf8PathBuf>> {
        let mut paths = Vec::new();
        for template in &self.templates {
            let name = template.name();
            match template.generate(storage, output_dir) {
                Ok(path) => {
                    paths.push(path);
                }
                Err(err) => {
                    warn!("Template '{}' failed: {:#}", name, err);
                }
            }
        }
        Ok(paths)
    }

    pub fn run_filtered(
        &self,
        names: &[String],
        storage: &CozoStorage,
        output_dir: &Utf8Path,
    ) -> Result<Vec<Utf8PathBuf>> {
        let mut paths = Vec::new();
        for name in names {
            match self.resolve(name) {
                Some(template) => match template.generate(storage, output_dir) {
                    Ok(path) => paths.push(path),
                    Err(err) => warn!("Template '{}' failed: {:#}", name, err),
                },
                None => warn!("Template '{}' not found in registry", name),
            }
        }
        Ok(paths)
    }
}

impl Default for DocRegistry {
    fn default() -> Self {
        Self::default_registry()
    }
}

// --- Query functions ---

pub fn query_file_dependencies(cozo: &CozoStorage) -> Result<Vec<FileDependency>> {
    let script = r#"
        ?[source_file, target_file] := 
            *edge{source: src_sym, target: tgt_sym},
            *project_symbol{qualified_name: src_sym, file_path: source_file},
            *project_symbol{qualified_name: tgt_sym, file_path: target_file},
            source_file != target_file
    "#;
    let res = cozo
        .run_script(script)
        .map_err(|e| DocGenerationError::QueryFailed(e.to_string()))?;

    let mut dedup = BTreeSet::new();
    for row in res.rows {
        if let (Some(DataValue::Str(source)), Some(DataValue::Str(target))) =
            (row.first(), row.get(1))
        {
            let source_norm = source.replace('\\', "/");
            let target_norm = target.replace('\\', "/");
            if source_norm != target_norm {
                dedup.insert((source_norm, target_norm));
            }
        }
    }
    Ok(dedup
        .into_iter()
        .map(|(s, t)| FileDependency {
            source_file: s,
            target_file: t,
        })
        .collect())
}

pub fn query_symbol_table(cozo: &CozoStorage) -> Result<Vec<SymbolRow>> {
    let script = r#"
        ?[qualified_name, symbol_name, symbol_kind, file_path, line_start, line_end, is_public] := 
            *project_symbol{id, qualified_name, symbol_name, symbol_kind, file_path, line_start, line_end, is_public}
    "#;
    let res = cozo
        .run_script(script)
        .map_err(|e| DocGenerationError::QueryFailed(e.to_string()))?;

    let mut rows = Vec::new();
    for row in res.rows {
        if let (
            Some(DataValue::Str(qualified_name)),
            Some(DataValue::Str(symbol_name)),
            Some(DataValue::Str(symbol_kind)),
            Some(DataValue::Str(file_path)),
            Some(DataValue::Num(Num::Int(line_start))),
            Some(DataValue::Num(Num::Int(line_end))),
            Some(DataValue::Bool(is_public)),
        ) = (
            row.first(),
            row.get(1),
            row.get(2),
            row.get(3),
            row.get(4),
            row.get(5),
            row.get(6),
        ) {
            rows.push(SymbolRow {
                qualified_name: qualified_name.to_string(),
                symbol_name: symbol_name.to_string(),
                symbol_kind: symbol_kind.to_string(),
                file_path: file_path.replace('\\', "/"),
                line_start: *line_start,
                line_end: *line_end,
                is_public: *is_public,
            });
        }
    }
    rows.sort_by(|a, b| {
        a.file_path
            .cmp(&b.file_path)
            .then_with(|| a.line_start.cmp(&b.line_start))
            .then_with(|| a.qualified_name.cmp(&b.qualified_name))
    });
    Ok(rows)
}

pub fn query_module_groups(cozo: &CozoStorage) -> Result<Vec<ModuleGroup>> {
    let script = r#"
        ?[file_path, label] := *node{id: file_path, label, category: 'file'}
    "#;
    let res = cozo
        .run_script(script)
        .map_err(|e| DocGenerationError::QueryFailed(e.to_string()))?;

    let mut groups: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for row in res.rows {
        if let (Some(DataValue::Str(file_path)), Some(DataValue::Str(_label))) =
            (row.first(), row.get(1))
        {
            let path_norm = file_path.replace('\\', "/");
            let dir = std::path::Path::new(&path_norm)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string());
            groups.entry(dir).or_default().insert(path_norm);
        }
    }

    let mut result = Vec::new();
    for (dir, files) in groups {
        let mut files_vec: Vec<String> = files.into_iter().collect();
        files_vec.sort();
        result.push(ModuleGroup {
            dir,
            files: files_vec,
        });
    }
    Ok(result)
}

#[derive(Debug, Clone, PartialEq)]
pub struct RiskyNode {
    pub id: String,
    pub label: String,
    pub category: String,
    pub risk_score: f64,
}

pub fn query_risky_nodes(cozo: &CozoStorage) -> Result<Vec<RiskyNode>> {
    let script = r#"
        ?[id, label, category, risk_score] := 
            *node{id, label, category, risk_score}, risk_score > 0
    "#;
    let res = cozo
        .run_script(script)
        .map_err(|e| DocGenerationError::QueryFailed(e.to_string()))?;

    let mut rows = Vec::new();
    for row in res.rows {
        if let (
            Some(DataValue::Str(id)),
            Some(DataValue::Str(label)),
            Some(DataValue::Str(category)),
            Some(DataValue::Num(Num::Float(risk_score))),
        ) = (row.first(), row.get(1), row.get(2), row.get(3))
        {
            rows.push(RiskyNode {
                id: id.to_string(),
                label: label.to_string(),
                category: category.to_string(),
                risk_score: *risk_score,
            });
        }
    }
    rows.sort_by(|a, b| {
        b.risk_score
            .partial_cmp(&a.risk_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(rows)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NeighborEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
}

pub fn query_all_edges(cozo: &CozoStorage) -> Result<Vec<NeighborEdge>> {
    let script = r#"
        ?[source, target, relation] := *edge{source, target, relation}
    "#;
    let res = cozo
        .run_script(script)
        .map_err(|e| DocGenerationError::QueryFailed(e.to_string()))?;

    let mut dedup = BTreeSet::new();
    for row in res.rows {
        if let (
            Some(DataValue::Str(source)),
            Some(DataValue::Str(target)),
            Some(DataValue::Str(relation)),
        ) = (row.first(), row.get(1), row.get(2))
        {
            dedup.insert((source.to_string(), target.to_string(), relation.to_string()));
        }
    }
    Ok(dedup
        .into_iter()
        .map(|(s, t, r)| NeighborEdge {
            source: s,
            target: t,
            relation: r,
        })
        .collect())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolWithKind {
    pub qualified_name: String,
    pub symbol_name: String,
    pub symbol_kind: String,
    pub file_path: String,
}

pub fn query_symbols_with_kinds(cozo: &CozoStorage) -> Result<Vec<SymbolWithKind>> {
    let script = r#"
        ?[qualified_name, symbol_name, symbol_kind, file_path] := 
            *project_symbol{id, qualified_name, symbol_name, symbol_kind, file_path}
    "#;
    let res = cozo
        .run_script(script)
        .map_err(|e| DocGenerationError::QueryFailed(e.to_string()))?;

    let mut rows = Vec::new();
    for row in res.rows {
        if let (
            Some(DataValue::Str(qualified_name)),
            Some(DataValue::Str(symbol_name)),
            Some(DataValue::Str(symbol_kind)),
            Some(DataValue::Str(file_path)),
        ) = (row.first(), row.get(1), row.get(2), row.get(3))
        {
            rows.push(SymbolWithKind {
                qualified_name: qualified_name.to_string(),
                symbol_name: symbol_name.to_string(),
                symbol_kind: symbol_kind.to_string(),
                file_path: file_path.replace('\\', "/"),
            });
        }
    }
    rows.sort_by(|a, b| {
        a.file_path
            .cmp(&b.file_path)
            .then_with(|| a.qualified_name.cmp(&b.qualified_name))
    });
    Ok(rows)
}

// --- Templates ---

pub struct DependencyGraphTemplate;
pub struct SymbolTableTemplate;
pub struct ModuleSummaryTemplate;
pub struct ModuleMapTemplate;
pub struct SymbolIndexTemplate;
pub struct ServiceBoundaryTemplate;
pub struct ChangeHotspotReportTemplate;
pub struct SemanticNeighborIndexTemplate;
pub struct DataFlowDiagramTemplate;
pub struct TestCoverageGapTemplate;

impl DocTemplate for DependencyGraphTemplate {
    fn name(&self) -> &'static str {
        "dependency_graph"
    }

    fn description(&self) -> &'static str {
        "Mermaid diagram of file-level dependencies"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let deps = query_file_dependencies(storage)?;
        let mut lines = Vec::new();
        lines.push("graph TD".to_string());

        let mut nodes = BTreeSet::new();
        for dep in &deps {
            nodes.insert(dep.source_file.clone());
            nodes.insert(dep.target_file.clone());
        }

        for node in &nodes {
            let id = mermaid_id(node);
            lines.push(format!("    {id}[\"{node}\"]"));
        }

        for dep in &deps {
            let src = mermaid_id(&dep.source_file);
            let tgt = mermaid_id(&dep.target_file);
            lines.push(format!("    {src} --> {tgt}"));
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("dependency_graph.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

impl DocTemplate for SymbolTableTemplate {
    fn name(&self) -> &'static str {
        "symbol_table"
    }

    fn description(&self) -> &'static str {
        "Markdown table of indexed symbols"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let rows = query_symbol_table(storage)?;
        let mut lines = Vec::new();
        lines.push("# Symbol Table".to_string());
        lines.push(String::new());

        let mut current_file: Option<&str> = None;
        const MAX_ROWS: usize = 10_000;

        for (count, row) in rows.iter().enumerate() {
            if count >= MAX_ROWS {
                break;
            }

            let file = row.file_path.as_str();
            if current_file != Some(file) {
                current_file = Some(file);
                lines.push(format!("### {file}"));
                lines.push(String::new());
                lines.push(
                    "| Qualified Name | Symbol Name | Kind | Line Range | Public |".to_string(),
                );
                lines.push("|---|---|---|---|---|".to_string());
            }

            let line_range = format!("{}-{}", row.line_start, row.line_end);
            let public_str = if row.is_public { "Yes" } else { "No" };
            lines.push(format!(
                "| {} | {} | {} | {} | {} |",
                row.qualified_name, row.symbol_name, row.symbol_kind, line_range, public_str
            ));
        }

        if rows.len() > MAX_ROWS {
            lines.push(String::new());
            lines.push("> ... truncated".to_string());
        }

        if rows.is_empty() {
            lines.push("*No symbols indexed.*".to_string());
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("symbol_table.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

impl DocTemplate for ModuleSummaryTemplate {
    fn name(&self) -> &'static str {
        "module_summary"
    }

    fn description(&self) -> &'static str {
        "High-level module overview"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let modules = query_module_groups(storage)?;
        let deps = query_file_dependencies(storage)?;

        // Count inter-module edges
        let mut module_edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for dep in &deps {
            let src_dir = std::path::Path::new(&dep.source_file)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string());
            let tgt_dir = std::path::Path::new(&dep.target_file)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string());
            if src_dir != tgt_dir {
                module_edges.entry(src_dir).or_default().insert(tgt_dir);
            }
        }

        let mut lines = Vec::new();
        lines.push("# Module Summary".to_string());
        lines.push(String::new());

        for group in &modules {
            let edge_count = module_edges.get(&group.dir).map(|s| s.len()).unwrap_or(0);
            lines.push(format!(
                "- **{}**: {} file(s), {} outgoing inter-module edge(s)",
                group.dir,
                group.files.len(),
                edge_count
            ));
            for file in &group.files {
                lines.push(format!("  - `{file}`"));
            }
        }

        if modules.is_empty() {
            lines.push("*No file nodes found.*".to_string());
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("module_summary.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

impl DocTemplate for ModuleMapTemplate {
    fn name(&self) -> &'static str {
        "module_map"
    }

    fn description(&self) -> &'static str {
        "Mermaid flowchart of file-level dependencies grouped by directory"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let deps = query_file_dependencies(storage)?;
        let modules = query_module_groups(storage)?;

        let mut lines = Vec::new();
        lines.push("flowchart TD".to_string());

        // Subgraphs per directory
        for group in &modules {
            let subgraph_id = mermaid_id(&group.dir);
            lines.push(format!("    subgraph {subgraph_id} [{}]", group.dir));
            for file in &group.files {
                let id = mermaid_id(file);
                lines.push(format!("        {id}[\"{file}\"]"));
            }
            lines.push("    end".to_string());
        }

        // Edges between files
        for dep in &deps {
            let src = mermaid_id(&dep.source_file);
            let tgt = mermaid_id(&dep.target_file);
            lines.push(format!("    {src} --> {tgt}"));
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("module_map.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

impl DocTemplate for SymbolIndexTemplate {
    fn name(&self) -> &'static str {
        "symbol_index"
    }

    fn description(&self) -> &'static str {
        "Comprehensive Markdown table of indexed symbols"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let rows = query_symbol_table(storage)?;
        let mut lines = vec![
            "# Symbol Index".to_string(),
            String::new(),
            "| Qualified Name | Symbol Name | Kind | File Path | Line Start | Line End | Public |"
                .to_string(),
            "|---|---|---|---|---|---|---|".to_string(),
        ];

        const MAX_ROWS: usize = 10_000;
        for (count, row) in rows.iter().enumerate() {
            if count >= MAX_ROWS {
                break;
            }
            let public_str = if row.is_public { "Yes" } else { "No" };
            lines.push(format!(
                "| {} | {} | {} | {} | {} | {} | {} |",
                row.qualified_name,
                row.symbol_name,
                row.symbol_kind,
                row.file_path,
                row.line_start,
                row.line_end,
                public_str
            ));
        }

        if rows.len() > MAX_ROWS {
            lines.push(String::new());
            lines.push("> ... truncated".to_string());
        }

        if rows.is_empty() {
            lines.push("*No symbols indexed.*".to_string());
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("symbol_index.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

impl DocTemplate for ServiceBoundaryTemplate {
    fn name(&self) -> &'static str {
        "service_boundary"
    }

    fn description(&self) -> &'static str {
        "Mermaid subgraph from Louvain communities"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let communities = storage
            .run_community_louvain()
            .map_err(|e| DocGenerationError::QueryFailed(e.to_string()))?;

        let mut groups: BTreeMap<i64, BTreeSet<String>> = BTreeMap::new();
        for (node, comm) in communities {
            groups
                .entry(comm)
                .or_default()
                .insert(node.replace('\\', "/"));
        }

        let mut lines = vec![
            "# Service Boundaries".to_string(),
            String::new(),
            "```mermaid".to_string(),
            "graph TD".to_string(),
        ];

        for (comm_id, nodes) in &groups {
            let subgraph_id = format!("community_{}", comm_id);
            lines.push(format!(
                "    subgraph {subgraph_id} [Community {}]",
                comm_id
            ));
            for node in nodes {
                let id = mermaid_id(node);
                lines.push(format!("        {id}[\"{}\"]", node));
            }
            lines.push("    end".to_string());
        }

        lines.push("```".to_string());

        if groups.is_empty() {
            lines.push(String::new());
            lines.push("*No communities detected.*".to_string());
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("service_boundary.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

impl DocTemplate for ChangeHotspotReportTemplate {
    fn name(&self) -> &'static str {
        "change_hotspot_report"
    }

    fn description(&self) -> &'static str {
        "Ranked list of nodes by risk score with 1-hop neighbor context"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let risky = query_risky_nodes(storage)?;
        let edges = query_all_edges(storage)?;

        // Build adjacency for 1-hop lookup
        let mut neighbors: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for edge in &edges {
            neighbors
                .entry(edge.source.clone())
                .or_default()
                .insert(edge.target.clone());
            neighbors
                .entry(edge.target.clone())
                .or_default()
                .insert(edge.source.clone());
        }

        let mut lines = Vec::new();
        lines.push("# Change Hotspot Report".to_string());
        lines.push(String::new());

        for node in &risky {
            lines.push(format!(
                "- **{}** (`{}`) — risk score: {:.2}",
                node.label, node.id, node.risk_score
            ));
            if let Some(nbrs) = neighbors.get(&node.id)
                && !nbrs.is_empty()
            {
                lines.push("  - Neighbors:".to_string());
                for nbr in nbrs {
                    lines.push(format!("    - `{nbr}`"));
                }
            }
        }

        if risky.is_empty() {
            lines.push("*No hotspots detected.*".to_string());
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("change_hotspot_report.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

impl DocTemplate for SemanticNeighborIndexTemplate {
    fn name(&self) -> &'static str {
        "semantic_neighbor_index"
    }

    fn description(&self) -> &'static str {
        "For each high-risk node, list semantic neighbors"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let script = r#"
            ?[id, label, category, risk_score] := 
                *node{id, label, category, risk_score}, risk_score > 0.5
        "#;
        let res = storage
            .run_script(script)
            .map_err(|e| DocGenerationError::QueryFailed(e.to_string()))?;

        let mut high_risk = Vec::new();
        for row in res.rows {
            if let (
                Some(DataValue::Str(id)),
                Some(DataValue::Str(label)),
                Some(DataValue::Str(category)),
                Some(DataValue::Num(Num::Float(risk_score))),
            ) = (row.first(), row.get(1), row.get(2), row.get(3))
            {
                high_risk.push(RiskyNode {
                    id: id.to_string(),
                    label: label.to_string(),
                    category: category.to_string(),
                    risk_score: *risk_score,
                });
            }
        }
        high_risk.sort_by(|a, b| {
            b.risk_score
                .partial_cmp(&a.risk_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });

        // Try semantically_similar edges first, fallback to all edges
        let mut semantic_edges = Vec::new();
        let sim_script = r#"
            ?[source, target, relation] := *edge{source, target, relation: 'semantically_similar'}
        "#;
        if let Ok(sim_res) = storage.run_script(sim_script) {
            for row in sim_res.rows {
                if let (
                    Some(DataValue::Str(source)),
                    Some(DataValue::Str(target)),
                    Some(DataValue::Str(relation)),
                ) = (row.first(), row.get(1), row.get(2))
                {
                    semantic_edges.push(NeighborEdge {
                        source: source.to_string(),
                        target: target.to_string(),
                        relation: relation.to_string(),
                    });
                }
            }
        }

        let use_semantic = !semantic_edges.is_empty();
        let edges = if use_semantic {
            semantic_edges
        } else {
            query_all_edges(storage)?
        };

        let mut neighbor_map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for edge in &edges {
            neighbor_map
                .entry(edge.source.clone())
                .or_default()
                .insert(edge.target.clone());
            neighbor_map
                .entry(edge.target.clone())
                .or_default()
                .insert(edge.source.clone());
        }

        let mut lines = Vec::new();
        lines.push("# Semantic Neighbor Index".to_string());
        lines.push(String::new());
        if !use_semantic {
            lines.push(
                "> Fallback: using all `edge` relations (no `semantically_similar` edges found)."
                    .to_string(),
            );
            lines.push(String::new());
        }

        for node in &high_risk {
            lines.push(format!(
                "- **{}** (`{}`) — risk: {:.2}",
                node.label, node.id, node.risk_score
            ));
            if let Some(nbrs) = neighbor_map.get(&node.id) {
                if !nbrs.is_empty() {
                    lines.push("  - Neighbors:".to_string());
                    for nbr in nbrs {
                        lines.push(format!("    - `{nbr}`"));
                    }
                } else {
                    lines.push("  - *No neighbors.*".to_string());
                }
            } else {
                lines.push("  - *No neighbors.*".to_string());
            }
        }

        if high_risk.is_empty() {
            lines.push("*No high-risk nodes found.*".to_string());
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("semantic_neighbor_index.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

impl DocTemplate for DataFlowDiagramTemplate {
    fn name(&self) -> &'static str {
        "data_flow_diagram"
    }

    fn description(&self) -> &'static str {
        "Mermaid diagram of handler-to-model coupling using edge relations"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let symbols = query_symbols_with_kinds(storage)?;
        let edges = query_all_edges(storage)?;

        let symbol_kind_map: BTreeMap<String, String> = symbols
            .into_iter()
            .map(|s| (s.qualified_name, s.symbol_kind.to_lowercase()))
            .collect();

        let handler_kinds: BTreeSet<String> = ["function", "method", "fn"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let model_kinds: BTreeSet<String> = ["struct", "class", "interface", "type", "enum"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let mut lines = Vec::new();
        lines.push("flowchart LR".to_string());

        let mut drawn_nodes = BTreeSet::new();
        let mut drawn_edges = BTreeSet::new();

        for edge in &edges {
            let src_kind = symbol_kind_map.get(&edge.source).map(|s| s.as_str());
            let tgt_kind = symbol_kind_map.get(&edge.target).map(|s| s.as_str());

            let src_is_handler = src_kind.is_some_and(|k| handler_kinds.contains(k));
            let src_is_model = src_kind.is_some_and(|k| model_kinds.contains(k));
            let tgt_is_handler = tgt_kind.is_some_and(|k| handler_kinds.contains(k));
            let tgt_is_model = tgt_kind.is_some_and(|k| model_kinds.contains(k));

            let is_data_flow = (src_is_handler && tgt_is_model) || (src_is_model && tgt_is_handler);

            if is_data_flow {
                if !drawn_nodes.contains(&edge.source) {
                    let id = mermaid_id(&edge.source);
                    lines.push(format!("    {id}[\"{}\"]", edge.source));
                    drawn_nodes.insert(edge.source.clone());
                }
                if !drawn_nodes.contains(&edge.target) {
                    let id = mermaid_id(&edge.target);
                    lines.push(format!("    {id}[\"{}\"]", edge.target));
                    drawn_nodes.insert(edge.target.clone());
                }
                let edge_key = if edge.source < edge.target {
                    (edge.source.clone(), edge.target.clone())
                } else {
                    (edge.target.clone(), edge.source.clone())
                };
                if drawn_edges.insert(edge_key) {
                    let src_id = mermaid_id(&edge.source);
                    let tgt_id = mermaid_id(&edge.target);
                    lines.push(format!("    {src_id} --> {tgt_id}"));
                }
            }
        }

        if drawn_nodes.is_empty() {
            lines.push("    %% No handler-to-model edges detected".to_string());
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("data_flow_diagram.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

impl DocTemplate for TestCoverageGapTemplate {
    fn name(&self) -> &'static str {
        "test_coverage_gap"
    }

    fn description(&self) -> &'static str {
        "Symbols from non-test files lacking test coverage"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let symbols = query_symbols_with_kinds(storage)?;
        let edges = query_all_edges(storage)?;

        // Identify test symbols (file path contains test-related segments)
        let mut test_symbols: BTreeSet<String> = BTreeSet::new();
        let mut non_test_symbols: Vec<SymbolWithKind> = Vec::new();
        for sym in &symbols {
            let fp_lower = sym.file_path.to_lowercase();
            if fp_lower.contains("test")
                || fp_lower.contains("spec")
                || fp_lower.contains("_test.")
                || fp_lower.contains(".test.")
            {
                test_symbols.insert(sym.qualified_name.clone());
            } else {
                non_test_symbols.push(sym.clone());
            }
        }

        // Build adjacency: symbol -> connected test symbols
        let mut tested_by: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for edge in &edges {
            if test_symbols.contains(&edge.source) && !test_symbols.contains(&edge.target) {
                tested_by
                    .entry(edge.target.clone())
                    .or_default()
                    .insert(edge.source.clone());
            }
            if test_symbols.contains(&edge.target) && !test_symbols.contains(&edge.source) {
                tested_by
                    .entry(edge.source.clone())
                    .or_default()
                    .insert(edge.target.clone());
            }
        }

        let mut lines = vec![
            "# Test Coverage Gaps".to_string(),
            String::new(),
            "Symbols from non-test files with no edges to test symbols.".to_string(),
            String::new(),
        ];

        let mut gaps = Vec::new();
        for sym in &non_test_symbols {
            if !tested_by.contains_key(&sym.qualified_name) {
                gaps.push(sym);
            }
        }

        if gaps.is_empty() {
            lines.push("*No coverage gaps detected.*".to_string());
        } else {
            lines.push("| Qualified Name | Symbol Name | Kind | File Path |".to_string());
            lines.push("|---|---|---|---|".to_string());
            for sym in &gaps {
                lines.push(format!(
                    "| {} | {} | {} | {} |",
                    sym.qualified_name, sym.symbol_name, sym.symbol_kind, sym.file_path
                ));
            }
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("test_coverage_gap.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

pub struct ApiContractIndexTemplate;
pub struct AdrStalenessReportTemplate;
pub struct CiPipelineMapTemplate;
pub struct TokenProvenanceMapTemplate;
pub struct FederationSummaryTemplate;
pub struct DependencyHealthTemplate;
pub struct ObservabilitySignalSnapshotTemplate;
pub struct CallGraphDetailTemplate;

impl DocTemplate for ApiContractIndexTemplate {
    fn name(&self) -> &'static str {
        "api_contract_index"
    }

    fn description(&self) -> &'static str {
        "Table of OpenAPI endpoints"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let mut lines = vec![
            "# API Contract Index".to_string(),
            String::new(),
            "| Spec Path | Method | Path | Summary |".to_string(),
            "|---|---|---|---|".to_string(),
        ];

        // TODO: api_endpoints is currently only in SQLite; mirror to CozoDB for full query support.
        let script = r#"
            ?[id, label, metadata] := *node{id, label, category: 'api_endpoint', risk_score, metadata}
        "#;
        match storage.run_script(script) {
            Ok(res) => {
                let mut rows: Vec<(String, String, String, String)> = Vec::new();
                for row in res.rows {
                    if let (
                        Some(DataValue::Str(id)),
                        Some(DataValue::Str(label)),
                        Some(DataValue::Json(meta)),
                    ) = (row.first(), row.get(1), row.get(2))
                    {
                        let method = meta
                            .as_object()
                            .and_then(|o| o.get("method").and_then(|v| v.as_str()))
                            .unwrap_or("-")
                            .to_string();
                        let path = meta
                            .as_object()
                            .and_then(|o| o.get("path").and_then(|v| v.as_str()))
                            .unwrap_or("-")
                            .to_string();
                        rows.push((id.replace('\\', "/"), method, path, label.to_string()));
                    }
                }
                rows.sort();
                if rows.is_empty() {
                    lines.push("*No API endpoint data available in Knowledge Graph. Run contract indexing to populate.*".to_string());
                } else {
                    for (spec, method, path, summary) in rows {
                        lines.push(format!("| {spec} | {method} | {path} | {summary} |"));
                    }
                }
            }
            Err(err) => {
                warn!("ApiContractIndexTemplate query failed: {err}");
                lines.push("*No API endpoint data available in Knowledge Graph. Requires SQLite `api_endpoints` table or CozoDB mirror.*".to_string());
            }
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("api_contract_index.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

impl DocTemplate for AdrStalenessReportTemplate {
    fn name(&self) -> &'static str {
        "adr_staleness_report"
    }

    fn description(&self) -> &'static str {
        "Markdown table with ADR staleness info"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let script = r#"
            ?[id, tx_id, category, entry_type, entity_normalized, change_type, summary, reason, committed_at, is_breaking, verification_status, trace_id] := 
                *ledger_entry{id, tx_id, category, entry_type, entity_normalized, change_type, summary, reason, committed_at, is_breaking, verification_status, trace_id}
        "#;
        let res = storage
            .run_script(script)
            .map_err(|e| DocGenerationError::QueryFailed(e.to_string()))?;

        let mut entries: Vec<(String, String, String, i64)> = Vec::new();
        let now = chrono::Utc::now();

        for row in res.rows {
            if let (
                Some(DataValue::Str(entity)),
                Some(DataValue::Str(summary)),
                Some(DataValue::Str(committed_at)),
                Some(DataValue::Str(category)),
                Some(DataValue::Str(entry_type)),
            ) = (row.get(4), row.get(6), row.get(8), row.get(2), row.get(3))
            {
                let cat_lower = category.to_lowercase();
                let type_lower = entry_type.to_lowercase();
                if !cat_lower.contains("adr")
                    && !cat_lower.contains("arch")
                    && !type_lower.contains("adr")
                    && !type_lower.contains("arch")
                {
                    continue;
                }
                let days_old = match chrono::DateTime::parse_from_rfc3339(committed_at) {
                    Ok(dt) => {
                        let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                        duration.num_days()
                    }
                    Err(_) => -1,
                };
                entries.push((
                    entity.replace('\\', "/"),
                    summary.to_string(),
                    committed_at.to_string(),
                    days_old,
                ));
            }
        }

        entries.sort_by(|a, b| {
            let a_ord = if a.3 < 0 { i64::MAX } else { a.3 };
            let b_ord = if b.3 < 0 { i64::MAX } else { b.3 };
            b_ord.cmp(&a_ord).then_with(|| a.0.cmp(&b.0))
        });

        let mut lines = vec![
            "# ADR Staleness Report".to_string(),
            String::new(),
            "| Entity | Summary | Committed At | Days Since |".to_string(),
            "|---|---|---|---|".to_string(),
        ];

        if entries.is_empty() {
            lines.push("*No architecture decision records found in ledger.*".to_string());
        } else {
            for (entity, summary, committed_at, days) in entries {
                let days_str = if days < 0 {
                    "Unknown".to_string()
                } else {
                    format!("{days}")
                };
                lines.push(format!(
                    "| {entity} | {summary} | {committed_at} | {days_str} |"
                ));
            }
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("adr_staleness_report.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

impl DocTemplate for CiPipelineMapTemplate {
    fn name(&self) -> &'static str {
        "ci_pipeline_map"
    }

    fn description(&self) -> &'static str {
        "Mermaid flowchart of CI config files and their linked source files"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let script = r#"
            ?[id, label] := *node{id, label, category: 'ci_config'}
        "#;
        let ci_nodes = storage
            .run_script(script)
            .map_err(|e| DocGenerationError::QueryFailed(e.to_string()))?;

        let mut ci_ids: BTreeSet<String> = BTreeSet::new();
        let mut id_to_label: BTreeMap<String, String> = BTreeMap::new();
        for row in ci_nodes.rows {
            if let (Some(DataValue::Str(id)), Some(DataValue::Str(label))) =
                (row.first(), row.get(1))
            {
                let norm_id = id.replace('\\', "/");
                ci_ids.insert(norm_id.clone());
                id_to_label.insert(norm_id, label.to_string());
            }
        }

        let mut edges: Vec<(String, String)> = Vec::new();
        let edge_script = r#"
            ?[source, target] := *edge{source, target}
        "#;
        let edge_res = storage
            .run_script(edge_script)
            .map_err(|e| DocGenerationError::QueryFailed(e.to_string()))?;

        for row in edge_res.rows {
            if let (Some(DataValue::Str(src)), Some(DataValue::Str(tgt))) =
                (row.first(), row.get(1))
            {
                let src_norm = src.replace('\\', "/");
                let tgt_norm = tgt.replace('\\', "/");
                if ci_ids.contains(&src_norm) || ci_ids.contains(&tgt_norm) {
                    edges.push((src_norm, tgt_norm));
                }
            }
        }

        edges.sort();
        edges.dedup();

        let mut lines = Vec::new();
        lines.push("graph TD".to_string());

        for id in &ci_ids {
            let m_id = mermaid_id(id);
            let label = id_to_label.get(id).map(|s| s.as_str()).unwrap_or(id);
            lines.push(format!("    {m_id}[\"{label}\"]"));
        }

        for (src, tgt) in &edges {
            let src_id = mermaid_id(src);
            let tgt_id = mermaid_id(tgt);
            lines.push(format!("    {src_id} --> {tgt_id}"));
        }

        if ci_ids.is_empty() {
            lines.push(
                "    note[\"No CI config nodes found. Run indexing to populate.\"]".to_string(),
            );
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("ci_pipeline_map.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

impl DocTemplate for TokenProvenanceMapTemplate {
    fn name(&self) -> &'static str {
        "token_provenance_map"
    }

    fn description(&self) -> &'static str {
        "Markdown table of symbol-level ledger attribution"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        // ledger_link is the CozoDB proxy for symbol-to-ledger attribution.
        // TODO: full token_provenance requires SQLite table.
        let script = r#"
            ?[node_id, ledger_id, interaction_type] := *ledger_link{node_id, ledger_id, interaction_type}
        "#;
        let res = storage
            .run_script(script)
            .map_err(|e| DocGenerationError::QueryFailed(e.to_string()))?;

        let mut rows: Vec<(String, String, String)> = Vec::new();
        for row in res.rows {
            if let (
                Some(DataValue::Str(node_id)),
                Some(DataValue::Str(ledger_id)),
                Some(DataValue::Str(interaction_type)),
            ) = (row.first(), row.get(1), row.get(2))
            {
                rows.push((
                    node_id.replace('\\', "/"),
                    ledger_id.to_string(),
                    interaction_type.to_string(),
                ));
            }
        }
        rows.sort();

        let mut lines = vec![
            "# Token Provenance Map".to_string(),
            String::new(),
            "| Symbol / Node | Ledger ID | Interaction |".to_string(),
            "|---|---|---|".to_string(),
        ];

        if rows.is_empty() {
            lines.push("*No token provenance data available in Knowledge Graph. Requires SQLite `token_provenance` table or CozoDB mirror.*".to_string());
        } else {
            for (node_id, ledger_id, interaction) in rows {
                lines.push(format!("| {node_id} | {ledger_id} | {interaction} |"));
            }
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("token_provenance_map.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

impl DocTemplate for FederationSummaryTemplate {
    fn name(&self) -> &'static str {
        "federation_summary"
    }

    fn description(&self) -> &'static str {
        "Cross-repo ledger entries"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let script = r#"
            ?[id, tx_id, category, entry_type, entity_normalized, change_type, summary, reason, committed_at, is_breaking, verification_status, trace_id] := 
                *ledger_entry{id, tx_id, category, entry_type, entity_normalized, change_type, summary, reason, committed_at, is_breaking, verification_status, trace_id}
        "#;
        let res = storage
            .run_script(script)
            .map_err(|e| DocGenerationError::QueryFailed(e.to_string()))?;

        let mut entries: Vec<(String, String, String, String, String)> = Vec::new();
        for row in res.rows {
            if let (
                Some(DataValue::Str(entity)),
                Some(DataValue::Str(summary)),
                Some(DataValue::Str(committed_at)),
                Some(DataValue::Str(trace_id)),
                Some(DataValue::Str(category)),
            ) = (row.get(4), row.get(6), row.get(8), row.get(11), row.get(2))
            {
                if trace_id.is_empty() {
                    continue;
                }
                entries.push((
                    trace_id.to_string(),
                    entity.replace('\\', "/"),
                    category.to_string(),
                    summary.to_string(),
                    committed_at.to_string(),
                ));
            }
        }

        entries.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        let mut lines = vec![
            "# Federation Summary".to_string(),
            String::new(),
            "| Trace ID | Entity | Category | Summary | Committed At |".to_string(),
            "|---|---|---|---|---|".to_string(),
        ];

        if entries.is_empty() {
            lines.push("*No federated ledger entries found. Cross-repo entries typically have non-empty trace IDs.*".to_string());
        } else {
            for (trace, entity, category, summary, committed_at) in entries {
                lines.push(format!(
                    "| {trace} | {entity} | {category} | {summary} | {committed_at} |"
                ));
            }
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("federation_summary.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

impl DocTemplate for DependencyHealthTemplate {
    fn name(&self) -> &'static str {
        "dependency_health"
    }

    fn description(&self) -> &'static str {
        "Health scorecard per module combining coupling depth, test coverage, and file age"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let mut module_edges: BTreeMap<String, usize> = BTreeMap::new();

        let script = r#"
            ?[source_file, target_file] := 
                *edge{source: src_sym, target: tgt_sym},
                *project_symbol{qualified_name: src_sym, file_path: source_file},
                *project_symbol{qualified_name: tgt_sym, file_path: target_file},
                source_file != target_file
        "#;
        let res = storage
            .run_script(script)
            .map_err(|e| DocGenerationError::QueryFailed(e.to_string()))?;

        for row in res.rows {
            if let (Some(DataValue::Str(src)), Some(DataValue::Str(tgt))) =
                (row.first(), row.get(1))
            {
                let src_norm = src.replace('\\', "/");
                let tgt_norm = tgt.replace('\\', "/");
                let src_dir = std::path::Path::new(&src_norm)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| ".".to_string());
                let tgt_dir = std::path::Path::new(&tgt_norm)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| ".".to_string());
                if src_dir != tgt_dir {
                    *module_edges.entry(src_dir).or_insert(0) += 1;
                }
            }
        }

        let mut lines = vec![
            "# Dependency Health".to_string(),
            String::new(),
            "| Module | Coupling Depth (outgoing) | Test Coverage | File Age |".to_string(),
            "|---|---|---|---|".to_string(),
        ];

        if module_edges.is_empty() {
            lines.push("*No inter-module dependency data available.*".to_string());
        } else {
            for (module, count) in &module_edges {
                lines.push(format!(
                    "| {module} | {count} | *Requires SQLite `test_outcome_history`* | *Requires SQLite `project_files` age* |"
                ));
            }
        }
        lines.push(String::new());
        lines.push(
            "> TODO: Test coverage and file age require SQLite tables not yet mirrored to CozoDB."
                .to_string(),
        );

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("dependency_health.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

impl DocTemplate for ObservabilitySignalSnapshotTemplate {
    fn name(&self) -> &'static str {
        "observability_signal_snapshot"
    }

    fn description(&self) -> &'static str {
        "Table of observability signals"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let mut lines = vec![
            "# Observability Signal Snapshot".to_string(),
            String::new(),
            "| Signal Type | Label | Metric Value | Raw Excerpt |".to_string(),
            "|---|---|---|---|".to_string(),
        ];

        let script = r#"
            ?[id, label, metadata] := *node{id, label, category: 'observability_signal', risk_score, metadata}
        "#;
        match storage.run_script(script) {
            Ok(res) => {
                let mut rows: Vec<(String, String, String, String)> = Vec::new();
                for row in res.rows {
                    if let (
                        Some(DataValue::Str(id)),
                        Some(DataValue::Str(label)),
                        Some(DataValue::Json(meta)),
                    ) = (row.first(), row.get(1), row.get(2))
                    {
                        let metric = meta
                            .as_object()
                            .and_then(|o| o.get("metric_value").and_then(|v| v.as_str()))
                            .unwrap_or("-")
                            .to_string();
                        let excerpt = meta
                            .as_object()
                            .and_then(|o| o.get("raw_excerpt").and_then(|v| v.as_str()))
                            .unwrap_or("-")
                            .to_string();
                        rows.push((id.replace('\\', "/"), label.to_string(), metric, excerpt));
                    }
                }
                rows.sort();
                if rows.is_empty() {
                    lines.push("*No observability signal data available in Knowledge Graph. Requires SQLite `observability_snapshots` table or CozoDB mirror.*".to_string());
                } else {
                    for (sig_type, label, metric, excerpt) in rows {
                        lines.push(format!("| {sig_type} | {label} | {metric} | {excerpt} |"));
                    }
                }
            }
            Err(err) => {
                warn!("ObservabilitySignalSnapshotTemplate query failed: {err}");
                lines.push("*No observability signal data available in Knowledge Graph. Requires SQLite `observability_snapshots` table or CozoDB mirror.*".to_string());
            }
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("observability_signal_snapshot.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

impl DocTemplate for CallGraphDetailTemplate {
    fn name(&self) -> &'static str {
        "call_graph_detail"
    }

    fn description(&self) -> &'static str {
        "Per-file focused call graphs for high-complexity modules"
    }

    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let sym_script = r#"
            ?[file_path, qualified_name] := *project_symbol{file_path, qualified_name}
        "#;
        let sym_res = storage
            .run_script(sym_script)
            .map_err(|e| DocGenerationError::QueryFailed(e.to_string()))?;

        let mut file_to_symbols: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for row in sym_res.rows {
            if let (Some(DataValue::Str(fp)), Some(DataValue::Str(qn))) = (row.first(), row.get(1))
            {
                let fp_norm = fp.replace('\\', "/");
                file_to_symbols
                    .entry(fp_norm)
                    .or_default()
                    .insert(qn.to_string());
            }
        }

        let mut top_files: Vec<(String, usize)> = file_to_symbols
            .iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect();
        top_files.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        top_files.truncate(5);

        let edge_script = r#"
            ?[caller, callee] := *edge{source: caller, target: callee}
        "#;
        let edge_res = storage
            .run_script(edge_script)
            .map_err(|e| DocGenerationError::QueryFailed(e.to_string()))?;

        let mut all_edges: Vec<(String, String)> = Vec::new();
        for row in edge_res.rows {
            if let (Some(DataValue::Str(caller)), Some(DataValue::Str(callee))) =
                (row.first(), row.get(1))
            {
                all_edges.push((caller.to_string(), callee.to_string()));
            }
        }

        let mut lines = Vec::new();
        lines.push("# Call Graph Detail".to_string());
        lines.push(String::new());

        if top_files.is_empty() {
            lines.push(
                "*No project symbols indexed. Run indexing to populate call graphs.*".to_string(),
            );
        } else {
            for (file, count) in &top_files {
                lines.push(format!("## {file} ({count} symbols)"));
                lines.push(String::new());
                lines.push("```mermaid".to_string());
                lines.push("graph TD".to_string());

                let file_symbols = file_to_symbols.get(file).cloned().unwrap_or_default();
                let mut shown = BTreeSet::new();
                for (caller, callee) in &all_edges {
                    if file_symbols.contains(caller) {
                        let c_id = mermaid_id(caller);
                        let t_id = mermaid_id(callee);
                        if shown.insert((c_id.clone(), t_id.clone())) {
                            lines
                                .push(format!("    {c_id}[\"{caller}\"] --> {t_id}[\"{callee}\"]"));
                        }
                    }
                }

                if shown.is_empty() {
                    lines.push("    note[\"No outgoing calls for this file.\"]".to_string());
                }

                lines.push("```".to_string());
                lines.push(String::new());
            }
        }

        let content = lines.join("\n") + "\n";
        let path = output_dir.join("call_graph_detail.md");
        write_file(&path, &content)?;
        Ok(path)
    }
}

// --- Helpers ---

fn mermaid_id(path: &str) -> String {
    let sanitized: String = path
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("f_{sanitized}")
}

fn write_file(path: &Utf8Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(DocGenerationError::IoFailed)?;
    }
    let mut file = std::fs::File::create(path).map_err(DocGenerationError::IoFailed)?;
    file.write_all(content.as_bytes())
        .map_err(DocGenerationError::IoFailed)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8Path;
    use std::path::PathBuf;

    fn in_memory_cozo() -> CozoStorage {
        CozoStorage::new(&PathBuf::from("")).unwrap()
    }

    fn populate_file_nodes(cozo: &CozoStorage, files: &[&str]) {
        let rows: String = files
            .iter()
            .map(|f| format!("['{f}', '{f}', 'file', 0.0, {{}}]"))
            .collect::<Vec<_>>()
            .join(",\n");
        let script =
            format!("?[id, label, category, risk_score, metadata] <- [\n{rows}\n] :put node");
        cozo.run_script(&script).unwrap();
    }

    #[allow(clippy::type_complexity)]
    fn populate_symbols(
        cozo: &CozoStorage,
        symbols: &[(i64, &str, &str, &str, &str, bool, i64, i64)],
    ) {
        let rows: String = symbols
            .iter()
            .map(|(id, fp, qn, sn, sk, pub_, ls, le)| {
                format!("[{id}, '{fp}', '{qn}', '{sn}', '{sk}', {pub_}, {ls}, {le}]")
            })
            .collect::<Vec<_>>()
            .join(",\n");
        let script = format!(
            "?[id, file_path, qualified_name, symbol_name, symbol_kind, is_public, line_start, line_end] <- [\n{rows}\n] :put project_symbol"
        );
        cozo.run_script(&script).unwrap();
    }

    fn populate_edges(cozo: &CozoStorage, edges: &[(&str, &str)]) {
        let rows: String = edges
            .iter()
            .map(|(s, t)| format!("['{s}', '{t}', 'calls', 1.0, 'tx1']"))
            .collect::<Vec<_>>()
            .join(",\n");
        let script = format!(
            "?[source, target, relation, confidence, provenance_id] <- [\n{rows}\n] :put edge"
        );
        cozo.run_script(&script).unwrap();
    }

    fn populate_risky_nodes(cozo: &CozoStorage, nodes: &[(&str, f64)]) {
        let rows: String = nodes
            .iter()
            .map(|(id, score)| format!("['{id}', '{id}', 'code', {score}, {{}}]"))
            .collect::<Vec<_>>()
            .join(",\n");
        let script =
            format!("?[id, label, category, risk_score, metadata] <- [\n{rows}\n] :put node");
        cozo.run_script(&script).unwrap();
    }

    #[allow(clippy::type_complexity)]
    fn populate_ledger_entries(
        cozo: &CozoStorage,
        entries: &[(
            i64,
            &str,
            &str,
            &str,
            &str,
            &str,
            &str,
            &str,
            &str,
            bool,
            &str,
            &str,
        )],
    ) {
        let rows: String = entries
            .iter()
            .map(
                |(
                    id,
                    tx_id,
                    category,
                    entry_type,
                    entity,
                    change_type,
                    summary,
                    reason,
                    committed_at,
                    is_breaking,
                    verification_status,
                    trace_id,
                )| {
                    format!(
                        "[{id}, '{tx_id}', '{category}', '{entry_type}', '{entity}', '{change_type}', '{summary}', '{reason}', '{committed_at}', {is_breaking}, '{verification_status}', '{trace_id}']",
                        is_breaking = if *is_breaking { "true" } else { "false" }
                    )
                },
            )
            .collect::<Vec<_>>()
            .join(",\n");
        let script = format!(
            "?[id, tx_id, category, entry_type, entity_normalized, change_type, summary, reason, committed_at, is_breaking, verification_status, trace_id] <- [\n{rows}\n] :put ledger_entry"
        );
        cozo.run_script(&script).unwrap();
    }

    fn populate_ledger_links(cozo: &CozoStorage, links: &[(&str, &str, &str)]) {
        let rows: String = links
            .iter()
            .map(|(node_id, ledger_id, interaction_type)| {
                format!("['{node_id}', '{ledger_id}', '{interaction_type}']")
            })
            .collect::<Vec<_>>()
            .join(",\n");
        let script =
            format!("?[node_id, ledger_id, interaction_type] <- [\n{rows}\n] :put ledger_link");
        cozo.run_script(&script).unwrap();
    }

    fn populate_nodes_with_category(cozo: &CozoStorage, nodes: &[(&str, &str, &str, f64, &str)]) {
        let rows: String = nodes
            .iter()
            .map(|(id, label, category, risk_score, metadata)| {
                format!("['{id}', '{label}', '{category}', {risk_score}, {metadata}]")
            })
            .collect::<Vec<_>>()
            .join(",\n");
        let script =
            format!("?[id, label, category, risk_score, metadata] <- [\n{rows}\n] :put node");
        cozo.run_script(&script).unwrap();
    }

    // --- Registry tests ---

    #[test]
    fn test_registry_resolve() {
        let registry = DocRegistry::default_registry();
        assert!(registry.resolve("dependency_graph").is_some());
        assert!(registry.resolve("symbol_table").is_some());
        assert!(registry.resolve("module_summary").is_some());
        assert!(registry.resolve("module_map").is_some());
        assert!(registry.resolve("symbol_index").is_some());
        assert!(registry.resolve("service_boundary").is_some());
        assert!(registry.resolve("change_hotspot_report").is_some());
        assert!(registry.resolve("semantic_neighbor_index").is_some());
        assert!(registry.resolve("data_flow_diagram").is_some());
        assert!(registry.resolve("test_coverage_gap").is_some());
        assert!(registry.resolve("api_contract_index").is_some());
        assert!(registry.resolve("adr_staleness_report").is_some());
        assert!(registry.resolve("ci_pipeline_map").is_some());
        assert!(registry.resolve("token_provenance_map").is_some());
        assert!(registry.resolve("federation_summary").is_some());
        assert!(registry.resolve("dependency_health").is_some());
        assert!(registry.resolve("observability_signal_snapshot").is_some());
        assert!(registry.resolve("call_graph_detail").is_some());
        assert!(registry.resolve("nonexistent").is_none());
    }

    #[test]
    fn test_registry_run_all() {
        let cozo = in_memory_cozo();
        let registry = DocRegistry::default_registry();
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let paths = registry.run_all(&cozo, output_dir).unwrap();
        assert_eq!(paths.len(), 18);
        for path in &paths {
            let content = std::fs::read_to_string(path).unwrap();
            assert!(!content.is_empty());
        }
    }

    #[test]
    fn test_registry_run_filtered() {
        let cozo = in_memory_cozo();
        let registry = DocRegistry::default_registry();
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let paths = registry
            .run_filtered(
                &["module_map".to_string(), "symbol_index".to_string()],
                &cozo,
                output_dir,
            )
            .unwrap();
        assert_eq!(paths.len(), 2);
    }

    // --- Query tests ---

    #[test]
    fn test_dependency_graph_query() {
        let cozo = in_memory_cozo();
        populate_file_nodes(&cozo, &["src/a.rs", "src/b.rs"]);
        populate_symbols(
            &cozo,
            &[
                (1, "src/a.rs", "mod_a::foo", "foo", "fn", true, 1, 5),
                (2, "src/b.rs", "mod_b::bar", "bar", "fn", true, 1, 5),
            ],
        );
        populate_edges(&cozo, &[("mod_a::foo", "mod_b::bar")]);

        let deps = query_file_dependencies(&cozo).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].source_file, "src/a.rs");
        assert_eq!(deps[0].target_file, "src/b.rs");
    }

    #[test]
    fn test_symbol_table_query() {
        let cozo = in_memory_cozo();
        populate_symbols(
            &cozo,
            &[
                (1, "src/b.rs", "B", "B", "struct", true, 10, 20),
                (2, "src/a.rs", "A", "A", "fn", true, 1, 5),
                (3, "src/a.rs", "C", "C", "fn", false, 5, 10),
            ],
        );

        let rows = query_symbol_table(&cozo).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].file_path, "src/a.rs");
        assert_eq!(rows[0].line_start, 1);
        assert_eq!(rows[1].file_path, "src/a.rs");
        assert_eq!(rows[1].line_start, 5);
        assert_eq!(rows[2].file_path, "src/b.rs");
        assert_eq!(rows[2].line_start, 10);
    }

    #[test]
    fn test_module_groups_query() {
        let cozo = in_memory_cozo();
        populate_file_nodes(&cozo, &["src/a.rs", "src/b.rs", "tests/t.rs"]);
        let groups = query_module_groups(&cozo).unwrap();
        assert_eq!(groups.len(), 2);
        let src_group = groups.iter().find(|g| g.dir == "src").unwrap();
        assert_eq!(src_group.files.len(), 2);
        let tests_group = groups.iter().find(|g| g.dir == "tests").unwrap();
        assert_eq!(tests_group.files.len(), 1);
    }

    // --- Template tests ---

    #[test]
    fn test_dependency_graph_empty() {
        let cozo = in_memory_cozo();
        let template = DependencyGraphTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("graph TD"));
        assert!(!content.contains("-->"));
    }

    #[test]
    fn test_dependency_graph_mermaid_syntax() {
        let cozo = in_memory_cozo();
        populate_file_nodes(&cozo, &["src/a.rs", "src/b.rs"]);
        populate_symbols(
            &cozo,
            &[
                (1, "src/a.rs", "mod_a::foo", "foo", "fn", true, 1, 5),
                (2, "src/b.rs", "mod_b::bar", "bar", "fn", true, 1, 5),
            ],
        );
        populate_edges(&cozo, &[("mod_a::foo", "mod_b::bar")]);

        let template = DependencyGraphTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("graph TD"));
        assert!(content.contains("-->"));
        assert!(!content.contains("f_src_a_rs --> f_src_a_rs"));
    }

    #[test]
    fn test_symbol_table_markdown_headers() {
        let cozo = in_memory_cozo();
        populate_symbols(&cozo, &[(1, "src/a.rs", "A", "A", "fn", true, 1, 5)]);
        let template = SymbolTableTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("| Qualified Name |"));
    }

    #[test]
    fn test_module_summary_lists_all_modules() {
        let cozo = in_memory_cozo();
        populate_file_nodes(&cozo, &["src/a.rs", "tests/t.rs"]);
        let template = ModuleSummaryTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("src"));
        assert!(content.contains("tests"));
    }

    #[test]
    fn test_module_map_groups_by_directory() {
        let cozo = in_memory_cozo();
        populate_file_nodes(&cozo, &["src/a.rs", "src/b.rs", "tests/t.rs"]);
        populate_symbols(
            &cozo,
            &[
                (1, "src/a.rs", "mod_a::foo", "foo", "fn", true, 1, 5),
                (2, "src/b.rs", "mod_b::bar", "bar", "fn", true, 1, 5),
            ],
        );
        populate_edges(&cozo, &[("mod_a::foo", "mod_b::bar")]);

        let template = ModuleMapTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("flowchart TD"));
        assert!(content.contains("subgraph"));
        assert!(content.contains("src"));
        assert!(content.contains("tests"));
        assert!(content.contains("-->"));
    }

    #[test]
    fn test_symbol_index_comprehensive_table() {
        let cozo = in_memory_cozo();
        populate_symbols(
            &cozo,
            &[
                (1, "src/a.rs", "A", "A", "fn", true, 1, 5),
                (2, "src/b.rs", "B", "B", "struct", false, 10, 20),
            ],
        );

        let template = SymbolIndexTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# Symbol Index"));
        assert!(content.contains(
            "| Qualified Name | Symbol Name | Kind | File Path | Line Start | Line End | Public |"
        ));
        assert!(content.contains("| A | A | fn | src/a.rs | 1 | 5 | Yes |"));
        assert!(content.contains("| B | B | struct | src/b.rs | 10 | 20 | No |"));
    }

    #[test]
    fn test_service_boundary_empty_communities() {
        let cozo = in_memory_cozo();
        let template = ServiceBoundaryTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# Service Boundaries"));
        assert!(content.contains("*No communities detected.*"));
    }

    #[test]
    fn test_semantic_neighbor_index_high_risk() {
        let cozo = in_memory_cozo();
        populate_risky_nodes(&cozo, &[("mod_a::foo", 0.8)]);
        populate_edges(&cozo, &[("mod_a::foo", "mod_b::bar")]);

        let template = SemanticNeighborIndexTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# Semantic Neighbor Index"));
        assert!(content.contains("mod_a::foo"));
        assert!(content.contains("mod_b::bar"));
    }

    #[test]
    fn test_data_flow_diagram_handler_to_model() {
        let cozo = in_memory_cozo();
        populate_symbols(
            &cozo,
            &[
                (
                    1,
                    "src/handler.rs",
                    "app::get_user",
                    "get_user",
                    "fn",
                    true,
                    1,
                    5,
                ),
                (2, "src/model.rs", "app::User", "User", "struct", true, 1, 5),
            ],
        );
        populate_edges(&cozo, &[("app::get_user", "app::User")]);

        let template = DataFlowDiagramTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("flowchart LR"));
        assert!(content.contains("app::get_user"));
        assert!(content.contains("app::User"));
        assert!(content.contains("-->"));
    }

    #[test]
    fn test_test_coverage_gap_lists_non_test_symbols() {
        let cozo = in_memory_cozo();
        populate_symbols(
            &cozo,
            &[
                (1, "src/lib.rs", "lib::add", "add", "fn", true, 1, 5),
                (
                    2,
                    "tests/test.rs",
                    "tests::test_add",
                    "test_add",
                    "fn",
                    true,
                    1,
                    5,
                ),
            ],
        );
        populate_edges(&cozo, &[("tests::test_add", "lib::add")]);

        let template = TestCoverageGapTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# Test Coverage Gaps"));
        // lib::add is connected to a test, so no gaps
        assert!(content.contains("*No coverage gaps detected.*"));
    }

    #[test]
    fn test_test_coverage_gap_detects_uncovered() {
        let cozo = in_memory_cozo();
        populate_symbols(
            &cozo,
            &[
                (1, "src/lib.rs", "lib::add", "add", "fn", true, 1, 5),
                (2, "src/other.rs", "other::sub", "sub", "fn", true, 1, 5),
            ],
        );
        // No test symbols, no edges

        let template = TestCoverageGapTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("lib::add"));
        assert!(content.contains("other::sub"));
    }

    #[test]
    fn test_deterministic_output() {
        let cozo = in_memory_cozo();
        populate_file_nodes(&cozo, &["src/a.rs", "src/b.rs"]);
        populate_symbols(
            &cozo,
            &[
                (1, "src/a.rs", "mod_a::foo", "foo", "fn", true, 1, 5),
                (2, "src/b.rs", "mod_b::bar", "bar", "fn", true, 1, 5),
            ],
        );
        populate_edges(&cozo, &[("mod_a::foo", "mod_b::bar")]);

        let template = DependencyGraphTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path1 = template.generate(&cozo, output_dir).unwrap();
        let bytes1 = std::fs::read(&path1).unwrap();

        let path2 = template.generate(&cozo, output_dir).unwrap();
        let bytes2 = std::fs::read(&path2).unwrap();
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn test_empty_kg_produces_valid_output() {
        let cozo = in_memory_cozo();
        let registry = DocRegistry::default_registry();
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let paths = registry.run_all(&cozo, output_dir).unwrap();
        assert_eq!(paths.len(), 18);
        for path in &paths {
            let content = std::fs::read_to_string(path).unwrap();
            assert!(!content.is_empty());
        }
    }

    // --- Batch 2 template tests ---

    #[test]
    fn test_api_contract_index_template() {
        let cozo = in_memory_cozo();
        populate_nodes_with_category(
            &cozo,
            &[(
                "spec.yaml",
                "Get Users",
                "api_endpoint",
                0.0,
                r#"{"method":"GET","path":"/users"}"#,
            )],
        );
        let template = ApiContractIndexTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("API Contract Index"));
        assert!(content.contains("spec.yaml"));
        assert!(content.contains("GET"));
    }

    #[test]
    fn test_adr_staleness_report_template() {
        let cozo = in_memory_cozo();
        populate_ledger_entries(
            &cozo,
            &[(
                1,
                "tx1",
                "ADR",
                "DECISION",
                "arch_001",
                "ADD",
                "Use Postgres",
                "scalability",
                "2024-01-01T00:00:00Z",
                false,
                "pass",
                "",
            )],
        );
        let template = AdrStalenessReportTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("ADR Staleness Report"));
        assert!(content.contains("arch_001"));
        assert!(content.contains("Use Postgres"));
    }

    #[test]
    fn test_ci_pipeline_map_template() {
        let cozo = in_memory_cozo();
        populate_nodes_with_category(&cozo, &[(".github/ci.yml", "CI", "ci_config", 0.0, "{}")]);
        populate_edges(&cozo, &[(".github/ci.yml", "src/main.rs")]);
        let template = CiPipelineMapTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("graph TD"));
        assert!(content.contains("CI"));
    }

    #[test]
    fn test_token_provenance_map_template() {
        let cozo = in_memory_cozo();
        populate_ledger_links(&cozo, &[("sym1", "tx1", "modified")]);
        let template = TokenProvenanceMapTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Token Provenance Map"));
        assert!(content.contains("sym1"));
        assert!(content.contains("tx1"));
    }

    #[test]
    fn test_federation_summary_template() {
        let cozo = in_memory_cozo();
        populate_ledger_entries(
            &cozo,
            &[(
                1,
                "tx1",
                "FEAT",
                "IMPLEMENTATION",
                "file.rs",
                "ADD",
                "summary",
                "reason",
                "2024-01-01T00:00:00Z",
                false,
                "pass",
                "sibling-repo/abc",
            )],
        );
        let template = FederationSummaryTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Federation Summary"));
        assert!(content.contains("sibling-repo/abc"));
    }

    #[test]
    fn test_dependency_health_template() {
        let cozo = in_memory_cozo();
        populate_file_nodes(&cozo, &["src/a.rs", "lib/b.rs"]);
        populate_symbols(
            &cozo,
            &[
                (1, "src/a.rs", "mod_a::foo", "foo", "fn", true, 1, 5),
                (2, "lib/b.rs", "mod_b::bar", "bar", "fn", true, 1, 5),
            ],
        );
        populate_edges(&cozo, &[("mod_a::foo", "mod_b::bar")]);
        let template = DependencyHealthTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Dependency Health"));
        assert!(content.contains("src"));
    }

    #[test]
    fn test_observability_signal_snapshot_template() {
        let cozo = in_memory_cozo();
        populate_nodes_with_category(
            &cozo,
            &[(
                "latency",
                "p99",
                "observability_signal",
                0.5,
                r#"{"metric_value":"120ms","raw_excerpt":"latency_p99=120"}"#,
            )],
        );
        let template = ObservabilitySignalSnapshotTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Observability Signal Snapshot"));
        assert!(content.contains("latency"));
        assert!(content.contains("120ms"));
    }

    #[test]
    fn test_call_graph_detail_template() {
        let cozo = in_memory_cozo();
        populate_symbols(
            &cozo,
            &[
                (1, "src/a.rs", "mod_a::foo", "foo", "fn", true, 1, 5),
                (2, "src/a.rs", "mod_a::bar", "bar", "fn", true, 6, 10),
                (3, "src/b.rs", "mod_b::baz", "baz", "fn", true, 1, 5),
            ],
        );
        populate_edges(
            &cozo,
            &[("mod_a::foo", "mod_a::bar"), ("mod_a::foo", "mod_b::baz")],
        );
        let template = CallGraphDetailTemplate;
        let tmp = tempfile::tempdir().unwrap();
        let output_dir = Utf8Path::from_path(tmp.path()).unwrap();
        let path = template.generate(&cozo, output_dir).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Call Graph Detail"));
        assert!(content.contains("src/a.rs"));
        assert!(content.contains("mod_a::foo"));
    }
}
