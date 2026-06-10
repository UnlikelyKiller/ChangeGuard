use super::ProjectIndexer;
use crate::index::entrypoint::{
    EntrypointKind, EntrypointStats, detect_python_entrypoints, detect_rust_entrypoints,
    detect_typescript_entrypoints,
};
use crate::index::topology::{DirectoryRole, TopologyIndexStats, classify_directory};
use miette::{IntoDiagnostic, Result};
use std::collections::{HashMap, HashSet};

pub fn index_topology(indexer: &mut ProjectIndexer) -> Result<TopologyIndexStats> {
    let all_files = super::discovery::discover_files(indexer)?;
    let now = chrono::Utc::now().to_rfc3339();
    let mut dir_files: HashMap<String, Vec<String>> = HashMap::new();
    for file_path in &all_files {
        let relative = file_path
            .strip_prefix(&indexer.repo_path)
            .unwrap_or(file_path);
        if let Some(parent) = relative.parent() {
            let dir = parent.to_string().replace('\\', "/");
            if !dir.is_empty() {
                dir_files.entry(dir).or_default().push(relative.to_string());
            }
        }
    }
    let mut all_dirs: HashSet<String> = dir_files.keys().cloned().collect();
    for dir in dir_files.keys() {
        let mut current = dir.as_str();
        while let Some(parent) = std::path::Path::new(current)
            .parent()
            .and_then(|p| p.to_str())
        {
            if !parent.is_empty() && !all_dirs.contains(parent) {
                all_dirs.insert(parent.to_string());
            }
            current = parent;
        }
    }
    let mut directories_classified = 0usize;
    let mut unclassified = 0usize;
    let mut role_counts: HashMap<DirectoryRole, usize> = HashMap::new();
    let conn = indexer.storage.get_connection_mut();
    for dir_path in &all_dirs {
        let files: Vec<&str> = dir_files
            .get(dir_path)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default();
        if let Some(classification) = classify_directory(dir_path, &files) {
            conn.execute("INSERT OR REPLACE INTO project_topology (dir_path, role, confidence, evidence, last_indexed_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![dir_path, classification.role.as_str(), classification.confidence, classification.evidence, now]).into_diagnostic()?;
            *role_counts.entry(classification.role).or_insert(0) += 1;
            directories_classified += 1;
        } else {
            unclassified += 1;
        }
    }
    Ok(TopologyIndexStats {
        directories_classified,
        unclassified,
        role_counts,
    })
}

pub fn classify_entrypoints(indexer: &mut ProjectIndexer) -> Result<EntrypointStats> {
    let conn = indexer.storage.get_connection();
    let mut stmt = conn.prepare("SELECT id, file_id, symbol_name, symbol_kind, is_public, metadata FROM project_symbols ORDER BY file_id").into_diagnostic()?;
    let rows: Vec<(i64, i64, String, String, bool, Option<String>)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get::<_, i32>(4)? != 0,
                row.get(5)?,
            ))
        })
        .into_diagnostic()?
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;
    drop(stmt);

    #[allow(clippy::type_complexity)]
    let mut file_symbols: HashMap<i64, Vec<(i64, String, String, bool, Option<String>)>> =
        HashMap::new();
    for (id, file_id, name, kind, is_public, metadata) in &rows {
        file_symbols.entry(*file_id).or_default().push((
            *id,
            name.clone(),
            kind.clone(),
            *is_public,
            metadata.clone(),
        ));
    }

    let mut path_stmt = conn
        .prepare("SELECT id, file_path, language FROM project_files")
        .into_diagnostic()?;
    let path_rows: Vec<(i64, String, Option<String>)> = path_stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .into_diagnostic()?
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;
    drop(path_stmt);

    let mut file_paths: HashMap<i64, String> = HashMap::new();
    for (id, path, _lang) in &path_rows {
        file_paths.insert(*id, path.clone());
    }

    let mut stats = EntrypointStats::default();
    let now = chrono::Utc::now().to_rfc3339();

    for (file_id, symbols) in &file_symbols {
        let file_path = match file_paths.get(file_id) {
            Some(p) => p.clone(),
            None => continue,
        };
        let file_lang = path_rows
            .iter()
            .find(|(id, _, _)| id == file_id)
            .and_then(|(_, _, lang)| lang.clone());
        let full_path = indexer.repo_path.join(&file_path);
        let Ok(content) = crate::util::fs::read_to_string_with_encoding(full_path.as_std_path())
        else {
            continue;
        };

        let sym_vec: Vec<crate::index::symbols::Symbol> = symbols
            .iter()
            .map(
                |(_, name, kind, is_public, metadata)| crate::index::symbols::Symbol {
                    name: name.clone(),
                    kind: crate::index::symbols::SymbolKind::parse(kind)
                        .unwrap_or(crate::index::symbols::SymbolKind::Function),
                    is_public: *is_public,
                    cognitive_complexity: None,
                    cyclomatic_complexity: None,
                    line_start: None,
                    line_end: None,
                    qualified_name: None,
                    byte_start: None,
                    byte_end: None,
                    entrypoint_kind: None,
                    metadata: metadata
                        .as_ref()
                        .and_then(|m| serde_json::from_str(m).ok())
                        .unwrap_or_default(),
                },
            )
            .collect();

        let classifications = match file_lang.as_deref() {
            Some("Rust") => detect_rust_entrypoints(&content, &sym_vec),
            Some("TypeScript") | Some("JavaScript") => {
                detect_typescript_entrypoints(&content, &sym_vec, &file_path)
            }
            Some("Python") => detect_python_entrypoints(&content, &sym_vec, &file_path),
            _ => continue,
        };

        let conn_mut = indexer.storage.get_connection_mut();
        for class in &classifications {
            let db_id = symbols
                .iter()
                .find(|(_, name, _, _, _)| name == &class.symbol_name)
                .map(|(id, _, _, _, _)| *id);
            if let Some(id) = db_id {
                conn_mut.execute("UPDATE project_symbols SET entrypoint_kind = ?1, confidence = ?2, evidence = ?3, last_indexed_at = ?4 WHERE id = ?5",
                    rusqlite::params![class.kind.as_str(), class.confidence, class.evidence, now, id]).into_diagnostic()?;
                match class.kind {
                    EntrypointKind::Entrypoint => stats.entrypoints += 1,
                    EntrypointKind::Handler => stats.handlers += 1,
                    EntrypointKind::PublicApi => stats.public_apis += 1,
                    EntrypointKind::Test => stats.tests += 1,
                    EntrypointKind::Ffi => stats.ffi += 1,
                    EntrypointKind::Macro => stats.macros += 1,
                    EntrypointKind::Internal => stats.internal += 1,
                }
            }
        }
    }
    Ok(stats)
}

pub fn infer_services(indexer: &mut ProjectIndexer) -> Result<super::ServiceIndexStats> {
    use crate::coverage::services::{DataModelSource, DirectoryTopology, infer_services};
    use crate::impact::packet::{ApiRoute, DataModel};
    use crate::index::call_graph::CallGraph;
    let (routes, data_models, call_graph) = {
        let conn = indexer.storage.get_connection();
        let mut route_stmt = conn.prepare("SELECT method, path_pattern, handler_symbol_name, framework, route_source, mount_prefix, is_dynamic, route_confidence, evidence, \
                                               auth_requirements, schema_refs, owning_service, consumers FROM api_routes").into_diagnostic()?;
        let routes: Vec<ApiRoute> = route_stmt
            .query_map([], |row| {
                let auth_raw: Option<String> = row.get(9)?;
                let schema_raw: Option<String> = row.get(10)?;
                let consumers_raw: Option<String> = row.get(12)?;

                let auth_requirements = auth_raw.and_then(|s| serde_json::from_str(&s).ok());
                let schema_refs = schema_raw.and_then(|s| serde_json::from_str(&s).ok());
                let consumers = consumers_raw.and_then(|s| serde_json::from_str(&s).ok());

                Ok(ApiRoute {
                    method: row.get(0)?,
                    path_pattern: row.get(1)?,
                    handler_symbol_name: row.get(2)?,
                    framework: row.get(3)?,
                    route_source: row.get(4)?,
                    mount_prefix: row.get(5)?,
                    is_dynamic: row.get::<_, i32>(6)? != 0,
                    route_confidence: row.get(7)?,
                    evidence: row.get(8)?,
                    auth_requirements,
                    schema_refs,
                    owning_service: row.get(11)?,
                    consumers,
                })
            })
            .into_diagnostic()?
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()?;
        let mut dm_stmt = conn.prepare("SELECT dm.model_name, dm.model_kind, dm.confidence, dm.evidence, pf.file_path FROM data_models dm JOIN project_files pf ON dm.model_file_id = pf.id").into_diagnostic()?;
        let data_models: Vec<DataModelSource> = dm_stmt
            .query_map([], |row| {
                Ok(DataModelSource {
                    model: DataModel {
                        model_name: row.get(0)?,
                        model_kind: row.get(1)?,
                        confidence: row.get(2)?,
                        evidence: row.get(3)?,
                    },
                    source_path: row.get(4)?,
                })
            })
            .into_diagnostic()?
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()?;
        let call_graph = CallGraph {
            edges: super::extraction::get_all_call_edges(indexer)?,
        };
        (routes, data_models, call_graph)
    };

    let topology = DirectoryTopology {
        classifications: indexer
            .storage
            .get_directory_classifications()
            .unwrap_or_default(),
    };
    let services = infer_services(
        &routes,
        &data_models,
        &call_graph,
        &topology,
        &indexer.config.services.definitions,
    );

    let mut files_assigned = 0;
    let conn_mut = indexer.storage.get_connection_mut();
    let tx = conn_mut.unchecked_transaction().into_diagnostic()?;
    tx.execute("UPDATE project_files SET service_name = NULL", [])
        .into_diagnostic()?;
    let mut sorted_services = services.clone();
    sorted_services.sort_by(|a, b| {
        b.directory
            .components()
            .count()
            .cmp(&a.directory.components().count())
    });
    for service in &sorted_services {
        let dir_str = service.directory.to_string_lossy().replace('\\', "/");
        let affected = if dir_str.is_empty() || dir_str == "." {
            tx.execute("UPDATE project_files SET service_name = ?1 WHERE file_path NOT LIKE '%/%' AND service_name IS NULL", rusqlite::params![service.name])
        } else {
            let pattern = format!("{}/%", dir_str);
            tx.execute("UPDATE project_files SET service_name = ?1 WHERE (file_path LIKE ?2 OR file_path = ?3) AND service_name IS NULL", rusqlite::params![service.name, pattern, dir_str])
        }.into_diagnostic()?;
        files_assigned += affected;
    }
    tx.commit().into_diagnostic()?;
    Ok(super::ServiceIndexStats {
        services_inferred: services.len(),
        files_assigned,
    })
}
