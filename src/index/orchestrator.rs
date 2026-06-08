use crate::index::analysis::analyze_file;
use crate::index::call_graph::CallGraphBuilder;
use crate::index::centrality::CentralityComputer;
use crate::index::ci_gates::CIGateExtractor;
use crate::index::data_models::DataModelExtractor;
use crate::index::docs::{DocIndexStats, parse_markdown};
use crate::index::entrypoint::{
    EntrypointKind, EntrypointStats, detect_python_entrypoints, detect_rust_entrypoints,
    detect_typescript_entrypoints,
};
use crate::index::env_schema::EnvSchemaIndexer;
use crate::index::languages::Language;
use crate::index::observability::ObservabilityExtractor;
use crate::index::routes::RouteExtractor;
use crate::index::test_mapping::TestMapper;
use crate::index::topology::{DirectoryRole, TopologyIndexStats, classify_directory};
use crate::index::types::{ProjectFile, ProjectSymbol, symbol_to_project_symbol};
use crate::index::walker::RepoWalker;
use crate::index::worker_pool::{JobResult, WorkerPool};
use crate::state::storage::StorageManager;
use camino::{Utf8Path, Utf8PathBuf};
use indicatif::{ProgressBar, ProgressStyle};
use miette::{IntoDiagnostic, Result};
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{info, warn};

pub const MAX_FILES: usize = 10_000;
pub const BATCH_SIZE: usize = 500;
pub const PARSER_VERSION: &str = "1";

pub const BINARY_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "ico", "woff", "woff2", "ttf", "eot", "pdf", "zip", "tar", "gz",
    "exe", "dll", "so", "dylib", "wasm", "class", "jar", "pyc",
];

pub const SUPPORTED_EXTENSIONS: &[&str] = &["rs", "ts", "tsx", "js", "jsx", "py", "go"];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexStats {
    pub files_indexed: usize,
    pub symbols_indexed: usize,
    pub parse_failures: usize,
    pub skipped_binary: usize,
    pub skipped_unsupported: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexStatus {
    pub total_files: usize,
    pub total_symbols: usize,
    pub stale_files: usize,
    pub last_indexed_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceIndexStats {
    pub services_inferred: usize,
    pub files_assigned: usize,
}

use crate::config::model::Config;

pub struct ProjectIndexer {
    storage: StorageManager,
    repo_path: Utf8PathBuf,
    config: Config,
}

impl ProjectIndexer {
    pub fn new(storage: StorageManager, repo_path: Utf8PathBuf, config: Config) -> Self {
        Self { storage, repo_path, config }
    }

    pub fn cozo(&self) -> Option<&crate::state::storage_cozo::CozoStorage> {
        self.storage.cozo.as_ref()
    }

    pub fn storage(&self) -> &StorageManager {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut StorageManager {
        &mut self.storage
    }

    pub fn new_for_worker(repo_path: Utf8PathBuf) -> Self {
        Self {
            storage: StorageManager::init_from_conn(
                rusqlite::Connection::open_in_memory().unwrap(),
            ),
            repo_path,
            config: Config::default(),
        }
    }

    pub fn build_kg_native(
        &self,
        local_model_config: &crate::config::model::LocalModelConfig,
    ) -> Result<()> {
        let Some(cozo) = &self.storage.cozo else {
            info!("CozoDB not available, skipping native KG build");
            return Ok(());
        };

        let stats =
            crate::index::graph_loader::build_native_graph(&self.storage, cozo, "native_kg", &self.config)?;

        // Optionally enrich with semantic extraction on a sample of files
        match self.get_semantic_sample_files() {
            Ok(sample_files) if !sample_files.is_empty() => {
                let extractor = crate::ai::semantic_extractor::SemanticExtractor::new(
                    crate::ai::semantic_extractor::SemanticExtractorConfig::default(),
                );
                match extractor.extract_batch(sample_files, local_model_config) {
                    Ok(result) => {
                        if let Err(e) =
                            crate::ai::semantic_extractor::SemanticExtractor::ingest_into_cozo(
                                &result,
                                cozo,
                                "semantic_kg",
                            )
                        {
                            warn!("Semantic extraction ingestion failed: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Semantic extraction failed: {}", e);
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to collect semantic sample files: {}", e);
            }
        }

        let communities = crate::index::graph_loader::run_community_louvain(cozo)?;
        let node_count = cozo.node_count()?;
        let edge_count = cozo.edge_count()?;

        info!(
            "Native KG build complete: {} nodes, {} edges, {} communities ({} files, {} symbols)",
            node_count,
            edge_count,
            communities.len(),
            stats.files_indexed,
            stats.symbols_indexed
        );

        Ok(())
    }

    fn get_semantic_sample_files(&self) -> Result<Vec<(std::path::PathBuf, String)>> {
        let conn = self.storage.get_connection();
        let mut stmt = conn
            .prepare(
                "SELECT file_path FROM project_files \
                 WHERE parse_status = 'OK' \
                 AND language IN ('Rust', 'TypeScript', 'Python', 'Go') \
                 LIMIT 10",
            )
            .into_diagnostic()?;

        let rows: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .into_diagnostic()?
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()?;
        drop(stmt);

        let mut files = Vec::new();
        for path_str in rows {
            let full_path = self.repo_path.join(&path_str);
            if let Ok(content) =
                crate::util::fs::read_to_string_with_encoding(full_path.as_std_path())
            {
                files.push((PathBuf::from(full_path.as_str()), content));
            }
        }
        Ok(files)
    }

    pub fn index_file(&self, path: &Utf8Path) -> Result<(ProjectFile, Vec<ProjectSymbol>)> {
        let relative = path.strip_prefix(&self.repo_path).unwrap_or(path);
        let outcome = analyze_file(relative.as_std_path(), self.repo_path.as_std_path());

        let now = chrono::Utc::now().to_rfc3339();
        let mut pf = ProjectFile {
            id: None,
            file_path: relative.to_string().replace('\\', "/"),
            language: relative
                .extension()
                .and_then(Language::from_extension)
                .map(|l| format!("{:?}", l)),
            content_hash: None,
            git_blob_oid: None,
            file_size: fs::metadata(path).ok().map(|m| m.len() as i64),
            mtime_ns: None,
            parser_version: PARSER_VERSION.to_string(),
            parse_status: if outcome.analysis_status.symbols
                == crate::impact::packet::AnalysisStatus::Ok
            {
                "OK".to_string()
            } else {
                "PARSE_FAILED".to_string()
            },
            last_indexed_at: now.clone(),
        };

        if let Ok(content) = crate::util::fs::read_to_string_with_encoding(path.as_std_path()) {
            pf.content_hash = Some(blake3::hash(content.as_bytes()).to_hex().to_string());
        }

        let ps = outcome
            .symbols
            .unwrap_or_default()
            .into_iter()
            .map(|s| symbol_to_project_symbol(&s, 0, &now))
            .collect();

        Ok((pf, ps))
    }

    pub fn index_file_with_edges(
        &self,
        path: &Utf8Path,
    ) -> Result<(
        ProjectFile,
        Vec<ProjectSymbol>,
        Vec<crate::index::call_graph::CallEdge>,
    )> {
        let (project_file, project_symbols) = self.index_file(path)?;
        if project_file.parse_status != "OK" {
            return Ok((project_file, project_symbols, Vec::new()));
        }

        let content = match crate::util::fs::read_to_string_with_encoding(path.as_std_path()) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to read file {} for call extraction: {}", path, e);
                return Ok((project_file, project_symbols, Vec::new()));
            }
        };

        let symbols: Vec<crate::index::symbols::Symbol> = project_symbols
            .iter()
            .filter_map(|ps| {
                Some(crate::index::symbols::Symbol {
                    name: ps.symbol_name.clone(),
                    kind: crate::index::symbols::SymbolKind::parse(&ps.symbol_kind)?,
                    is_public: ps.is_public,
                    cognitive_complexity: ps.cognitive_complexity,
                    cyclomatic_complexity: ps.cyclomatic_complexity,
                    line_start: ps.line_start,
                    line_end: ps.line_end,
                    qualified_name: Some(ps.qualified_name.clone()),
                    byte_start: ps.byte_start,
                    byte_end: ps.byte_end,
                    entrypoint_kind: Some(ps.entrypoint_kind.clone()),
                    metadata: ps
                        .metadata
                        .as_ref()
                        .and_then(|m| serde_json::from_str(m).ok())
                        .unwrap_or_default(),
                })
            })
            .collect();

        let calls =
            match crate::index::languages::extract_calls(path.as_std_path(), &content, &symbols) {
                Ok(c) => c,
                Err(e) => {
                    warn!("Call extraction failed for {}: {}", path, e);
                    Vec::new()
                }
            };

        Ok((project_file, project_symbols, calls))
    }

    pub fn check_status(&self) -> Result<IndexStatus> {
        let conn = self.storage.get_connection();

        let total_files: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM project_files WHERE parse_status != 'DELETED'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .into_diagnostic()? as usize;

        let total_symbols: usize = conn
            .query_row("SELECT COUNT(*) FROM project_symbols", [], |row| {
                row.get::<_, i64>(0)
            })
            .into_diagnostic()? as usize;

        let last_indexed_at: Option<String> = conn
            .query_row(
                "SELECT MAX(last_indexed_at) FROM project_files",
                [],
                |row| row.get::<_, Option<String>>(0),
            )
            .into_diagnostic()?;

        // Count stale files by comparing content_hash with current files
        let current_files = self.discover_files()?;
        let mut stale_count = 0usize;

        for file_path in &current_files {
            let relative = file_path
                .strip_prefix(&self.repo_path)
                .unwrap_or(file_path)
                .to_string();

            let current_hash =
                match crate::util::fs::read_to_string_with_encoding(file_path.as_std_path()) {
                    Ok(c) => blake3::hash(c.as_bytes()).to_hex().to_string(),
                    Err(_) => continue,
                };
            let stored_hash: Option<String> = conn
                .query_row(
                    "SELECT content_hash FROM project_files WHERE file_path = ?1",
                    [&relative],
                    |row| row.get::<_, Option<String>>(0),
                )
                .ok()
                .flatten();

            if stored_hash.as_deref() != Some(&current_hash) {
                stale_count += 1;
            }
        }

        Ok(IndexStatus {
            total_files,
            total_symbols,
            stale_files: stale_count,
            last_indexed_at,
        })
    }

    /// Full index: clear existing data, scan all files, index everything in parallel.
    pub fn full_index(&mut self) -> Result<IndexStats> {
        let start = Instant::now();
        let files = self.discover_files()?;

        self.clear_project_data()?;

        let pb = create_progress_bar(files.len());
        let pool = WorkerPool::new(0);
        let repo_path = self.repo_path.clone();

        let rx = pool.process_parsing(files, Some(pb.clone()), move |path| {
            let relative = path.strip_prefix(&repo_path).unwrap_or(path);
            let outcome = analyze_file(relative.as_std_path(), repo_path.as_std_path());

            let now = chrono::Utc::now().to_rfc3339();
            let mut pf = ProjectFile {
                id: None,
                file_path: relative.to_string().replace('\\', "/"),
                language: relative
                    .extension()
                    .and_then(Language::from_extension)
                    .map(|l| format!("{:?}", l)),
                content_hash: None,
                git_blob_oid: None,
                file_size: fs::metadata(path).ok().map(|m| m.len() as i64),
                mtime_ns: None,
                parser_version: PARSER_VERSION.to_string(),
                parse_status: if outcome.analysis_status.symbols
                    == crate::impact::packet::AnalysisStatus::Ok
                {
                    "OK".to_string()
                } else {
                    "PARSE_FAILED".to_string()
                },
                last_indexed_at: now.clone(),
            };

            if let Ok(content) = crate::util::fs::read_to_string_with_encoding(path.as_std_path()) {
                pf.content_hash = Some(blake3::hash(content.as_bytes()).to_hex().to_string());
            }

            let ps = outcome
                .symbols
                .unwrap_or_default()
                .into_iter()
                .map(|s| symbol_to_project_symbol(&s, 0, &now))
                .collect();

            Ok((pf, ps))
        })?;

        let stats = self.collect_results(rx, true)?;
        pb.finish_and_clear();
        self.store_index_metadata()?;

        let duration_ms = start.elapsed().as_millis() as u64;
        info!("Full index complete in {}ms", duration_ms);
        Ok(IndexStats {
            duration_ms,
            ..stats
        })
    }

    /// Incremental index: only re-index files that changed.
    pub fn incremental_index(&mut self) -> Result<IndexStats> {
        let start = Instant::now();
        let current_files = self.discover_files()?;

        let existing_files = self.load_existing_files()?;
        let mut files_to_reindex = Vec::new();

        for file_path in &current_files {
            let relative = file_path
                .strip_prefix(&self.repo_path)
                .unwrap_or(file_path)
                .to_string();
            if let Some(existing) = existing_files.get(&relative) {
                match crate::util::fs::read_to_string_with_encoding(file_path.as_std_path()) {
                    Ok(content) => {
                        let hash = blake3::hash(content.as_bytes()).to_hex().to_string();
                        if existing.content_hash.as_deref() != Some(&hash) {
                            files_to_reindex.push(file_path.clone());
                        }
                    }
                    Err(_) => {
                        // If we can't read it now, it might be locked or invalid, queue for re-index to let the worker handle/report it.
                        files_to_reindex.push(file_path.clone());
                    }
                }
            } else {
                files_to_reindex.push(file_path.clone());
            }
        }

        if files_to_reindex.is_empty() {
            return Ok(IndexStats {
                duration_ms: start.elapsed().as_millis() as u64,
                files_indexed: 0,
                symbols_indexed: 0,
                parse_failures: 0,
                skipped_binary: 0,
                skipped_unsupported: 0,
            });
        }

        let pb = create_progress_bar(files_to_reindex.len());
        let pool = WorkerPool::new(0);
        let repo_path = self.repo_path.clone();

        let rx = pool.process_parsing(files_to_reindex, Some(pb.clone()), move |path| {
            let relative = path.strip_prefix(&repo_path).unwrap_or(path);
            let outcome = analyze_file(relative.as_std_path(), repo_path.as_std_path());
            let now = chrono::Utc::now().to_rfc3339();
            let mut pf = ProjectFile {
                id: None,
                file_path: relative.to_string().replace('\\', "/"),
                language: relative
                    .extension()
                    .and_then(Language::from_extension)
                    .map(|l| format!("{:?}", l)),
                content_hash: None,
                git_blob_oid: None,
                file_size: fs::metadata(path).ok().map(|m| m.len() as i64),
                mtime_ns: None,
                parser_version: PARSER_VERSION.to_string(),
                parse_status: if outcome.analysis_status.symbols
                    == crate::impact::packet::AnalysisStatus::Ok
                {
                    "OK".to_string()
                } else {
                    "PARSE_FAILED".to_string()
                },
                last_indexed_at: now.clone(),
            };
            if let Ok(content) = crate::util::fs::read_to_string_with_encoding(path.as_std_path()) {
                pf.content_hash = Some(blake3::hash(content.as_bytes()).to_hex().to_string());
            }
            let ps = outcome
                .symbols
                .unwrap_or_default()
                .into_iter()
                .map(|s| symbol_to_project_symbol(&s, 0, &now))
                .collect();
            Ok((pf, ps))
        })?;

        let stats = self.collect_results(rx, false)?;
        pb.finish_and_clear();
        self.store_index_metadata()?;

        Ok(IndexStats {
            duration_ms: start.elapsed().as_millis() as u64,
            ..stats
        })
    }

    fn collect_results(
        &mut self,
        rx: crossbeam::channel::Receiver<JobResult>,
        is_full: bool,
    ) -> Result<IndexStats> {
        let mut files_indexed = 0;
        let mut symbols_indexed = 0;
        let mut parse_failures = 0;
        let mut batch_files = Vec::new();
        let mut batch_symbols = Vec::new();

        while let Ok(result) = rx.recv() {
            match result {
                JobResult::Parsed(pf, ps) => {
                    if pf.parse_status == "PARSE_FAILED" {
                        parse_failures += 1;
                    } else {
                        files_indexed += 1;
                    }
                    symbols_indexed += ps.len();

                    if !is_full {
                        let _ = self.delete_file_symbols(&pf.file_path);
                    }

                    batch_files.push(pf);
                    batch_symbols.push(ps);

                    if batch_files.len() >= BATCH_SIZE {
                        if is_full {
                            self.insert_batch(&batch_files, &batch_symbols)?;
                        } else {
                            self.upsert_batch(&batch_files, &batch_symbols)?;
                        }
                        batch_files.clear();
                        batch_symbols.clear();
                    }
                }
                JobResult::Failure(path, err) => {
                    warn!("Parallel index failure for {}: {}", path, err);
                    parse_failures += 1;
                }
                _ => {}
            }
        }

        if !batch_files.is_empty() {
            if is_full {
                self.insert_batch(&batch_files, &batch_symbols)?;
            } else {
                self.upsert_batch(&batch_files, &batch_symbols)?;
            }
        }

        Ok(IndexStats {
            files_indexed,
            symbols_indexed,
            parse_failures,
            skipped_binary: 0,
            skipped_unsupported: 0,
            duration_ms: 0,
        })
    }

    pub fn discover_files(&self) -> Result<Vec<Utf8PathBuf>> {
        RepoWalker::new(
            self.repo_path.clone(),
            SUPPORTED_EXTENSIONS,
            BINARY_EXTENSIONS,
        )
        .discover_files()
    }

    pub fn discover_doc_files(&self) -> Result<Vec<Utf8PathBuf>> {
        RepoWalker::new(
            self.repo_path.clone(),
            SUPPORTED_EXTENSIONS,
            BINARY_EXTENSIONS,
        )
        .discover_doc_files()
    }

    pub fn index_docs(&mut self) -> Result<DocIndexStats> {
        let doc_files = self.discover_doc_files()?;
        let has_readme = self.repo_path.join("README.md").exists();
        let mut docs_indexed = 0usize;
        let mut parse_failures = 0usize;
        let now = chrono::Utc::now().to_rfc3339();

        for doc_path in &doc_files {
            let relative_path = doc_path
                .strip_prefix(&self.repo_path)
                .unwrap_or(doc_path)
                .to_string();
            let content = match fs::read_to_string(doc_path) {
                Ok(c) => c,
                Err(e) => {
                    warn!("Failed to read doc file {}: {}", doc_path, e);
                    parse_failures += 1;
                    continue;
                }
            };
            let parsed = parse_markdown(&content, &relative_path);
            let file_id = self.ensure_file_entry(&relative_path, &content, &now)?;
            let sections_json =
                serde_json::to_string(&parsed.sections).unwrap_or_else(|_| "[]".to_string());
            let code_blocks_json =
                serde_json::to_string(&parsed.code_blocks).unwrap_or_else(|_| "[]".to_string());
            let internal_links_json =
                serde_json::to_string(&parsed.internal_links).unwrap_or_else(|_| "[]".to_string());

            let conn = self.storage.get_connection_mut();
            conn.execute("INSERT OR REPLACE INTO project_docs (file_id, title, summary, sections, code_blocks, internal_links, confidence, last_indexed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![file_id, parsed.title, parsed.summary, sections_json, code_blocks_json, internal_links_json, 1.0_f64, now]).into_diagnostic()?;
            docs_indexed += 1;
        }
        Ok(DocIndexStats {
            docs_indexed,
            parse_failures,
            missing_readme: !has_readme,
        })
    }

    fn ensure_file_entry(&mut self, relative_path: &str, content: &str, now: &str) -> Result<i64> {
        let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
        let conn = self.storage.get_connection();
        let existing_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM project_files WHERE file_path = ?1",
                [relative_path],
                |row| row.get(0),
            )
            .ok();
        if let Some(id) = existing_id {
            return Ok(id);
        }
        let conn = self.storage.get_connection_mut();
        conn.execute("INSERT INTO project_files (file_path, language, content_hash, git_blob_oid, file_size, mtime_ns, parser_version, parse_status, last_indexed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![relative_path, "Markdown", content_hash, Option::<String>::None, content.len() as i64, Option::<i64>::None, PARSER_VERSION, "OK", now]).into_diagnostic()?;
        Ok(conn.last_insert_rowid())
    }

    pub fn index_topology(&mut self) -> Result<TopologyIndexStats> {
        let all_files = self.discover_files()?;
        let now = chrono::Utc::now().to_rfc3339();
        let mut dir_files: HashMap<String, Vec<String>> = HashMap::new();
        for file_path in &all_files {
            let relative = file_path.strip_prefix(&self.repo_path).unwrap_or(file_path);
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
        let conn = self.storage.get_connection_mut();
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

    pub fn classify_entrypoints(&mut self) -> Result<EntrypointStats> {
        let conn = self.storage.get_connection();
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
        let mut file_symbols: HashMap<
            i64,
            Vec<(i64, String, String, bool, Option<String>)>,
        > = HashMap::new();
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
            let full_path = self.repo_path.join(&file_path);
            let Ok(content) =
                crate::util::fs::read_to_string_with_encoding(full_path.as_std_path())
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

            let conn_mut = self.storage.get_connection_mut();
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

    pub fn build_call_graph(&self) -> Result<crate::index::call_graph::CallGraphStats> {
        CallGraphBuilder::new(&self.storage, self.repo_path.as_std_path().to_path_buf()).build()
    }

    pub fn extract_routes(&self) -> Result<crate::index::routes::RouteStats> {
        RouteExtractor::new(&self.storage, self.repo_path.as_std_path().to_path_buf()).extract()
    }

    pub fn clear_routes(&self, file_ids: &[i64]) -> Result<()> {
        RouteExtractor::new(&self.storage, self.repo_path.as_std_path().to_path_buf())
            .clear_routes(file_ids)
    }

    pub fn clear_structural_edges(&self, file_ids: &[i64]) -> Result<()> {
        if file_ids.is_empty() {
            return Ok(());
        }
        let conn = self.storage.get_connection();
        for &fid in file_ids {
            conn.execute(
                "DELETE FROM structural_edges WHERE caller_file_id = ?1",
                [fid],
            )
            .into_diagnostic()?;
        }
        Ok(())
    }

    pub fn extract_data_models(&self) -> Result<crate::index::data_models::DataModelStats> {
        DataModelExtractor::new(&self.storage, self.repo_path.as_std_path().to_path_buf()).extract()
    }

    pub fn clear_data_models(&self, file_ids: &[i64]) -> Result<()> {
        DataModelExtractor::new(&self.storage, self.repo_path.as_std_path().to_path_buf())
            .clear_data_models(file_ids)
    }

    pub fn extract_observability(&self) -> Result<crate::index::observability::ObservabilityStats> {
        ObservabilityExtractor::new(&self.storage, self.repo_path.as_std_path().to_path_buf())
            .extract()
    }

    pub fn compute_centrality(&self) -> Result<crate::index::centrality::CentralityStats> {
        CentralityComputer::new(&self.storage).compute()
    }

    pub fn get_all_call_edges(&self) -> Result<Vec<crate::index::call_graph::CallEdge>> {
        use crate::index::call_graph::{CallEdge, CallKind, ResolutionStatus};
        let conn = self.storage.get_connection();
        let mut stmt = conn.prepare("SELECT COALESCE(ps_caller.qualified_name, ps_caller.symbol_name), pf_caller.file_path, COALESCE(ps_callee.qualified_name, ps_callee.symbol_name), pf_callee.file_path, se.call_kind, se.resolution_status, se.confidence, se.evidence FROM structural_edges se JOIN project_symbols ps_caller ON se.caller_symbol_id = ps_caller.id JOIN project_files pf_caller ON se.caller_file_id = pf_caller.id LEFT JOIN project_symbols ps_callee ON se.callee_symbol_id = ps_callee.id LEFT JOIN project_files pf_callee ON se.callee_file_id = pf_callee.id").into_diagnostic()?;
        let edges = stmt
            .query_map([], |row| {
                Ok(CallEdge {
                    caller_name: row.get(0)?,
                    caller_file: PathBuf::from(row.get::<_, String>(1)?),
                    callee_name: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    callee_file: row.get::<_, Option<String>>(3)?.map(PathBuf::from),
                    call_kind: match row.get::<_, String>(4)?.as_str() {
                        "METHOD_CALL" => CallKind::MethodCall,
                        "TRAIT_DISPATCH" => CallKind::TraitDispatch,
                        "DYNAMIC" => CallKind::Dynamic,
                        "EXTERNAL" => CallKind::External,
                        _ => CallKind::Direct,
                    },
                    resolution_status: match row.get::<_, String>(5)?.as_str() {
                        "AMBIGUOUS" => ResolutionStatus::Ambiguous,
                        "UNRESOLVED" => ResolutionStatus::Unresolved,
                        "CAPPED" => ResolutionStatus::Capped,
                        _ => ResolutionStatus::Resolved,
                    },
                    confidence: row.get(6)?,
                    evidence: row.get(7)?,
                })
            })
            .into_diagnostic()?
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()?;
        Ok(edges)
    }

    pub fn extract_test_mappings(&self) -> Result<crate::index::test_mapping::TestMappingStats> {
        TestMapper::new(&self.storage, self.repo_path.as_std_path().to_path_buf()).extract()
    }

    pub fn extract_ci_gates(&self) -> Result<crate::index::ci_gates::CIGateStats> {
        CIGateExtractor::new(&self.storage, self.repo_path.as_std_path().to_path_buf()).extract()
    }

    pub fn extract_env_schema(&self) -> Result<crate::index::env_schema::EnvSchemaStats> {
        EnvSchemaIndexer::new(&self.storage, self.repo_path.as_std_path().to_path_buf()).extract()
    }

    pub fn infer_services(&mut self) -> Result<ServiceIndexStats> {
        use crate::coverage::services::{DataModelSource, DirectoryTopology, infer_services};
        use crate::impact::packet::{ApiRoute, DataModel};
        use crate::index::call_graph::CallGraph;
        let (routes, data_models, call_graph) = {
            let conn = self.storage.get_connection();
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
                edges: self.get_all_call_edges()?,
            };
            (routes, data_models, call_graph)
        };

        let topology = DirectoryTopology {
            classifications: self
                .storage
                .get_directory_classifications()
                .unwrap_or_default(),
        };
        let services = infer_services(&routes, &data_models, &call_graph, &topology, &self.config.services.definitions);

        let mut files_assigned = 0;
        let conn_mut = self.storage.get_connection_mut();
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
        Ok(ServiceIndexStats {
            services_inferred: services.len(),
            files_assigned,
        })
    }

    // --- State & Storage Helpers ---

    fn clear_project_data(&mut self) -> Result<()> {
        let conn = self.storage.get_connection_mut();
        for table in [
            "symbol_centrality",
            "structural_edges",
            "api_routes",
            "data_models",
            "observability_patterns",
            "test_mapping",
            "ci_gates",
            "env_references",
            "env_declarations",
            "project_docs",
            "project_topology",
            "project_symbols",
            "project_files",
        ] {
            conn.execute(&format!("DELETE FROM {}", table), [])
                .into_diagnostic()?;
        }
        Ok(())
    }

    fn insert_batch(
        &mut self,
        files: &[ProjectFile],
        symbols: &[Vec<ProjectSymbol>],
    ) -> Result<()> {
        let conn = self.storage.get_connection_mut();
        let tx = conn.unchecked_transaction().into_diagnostic()?;
        for (i, pf) in files.iter().enumerate() {
            insert_file_row(&tx, pf)?;
            let file_id = tx.last_insert_rowid();
            for ps in &symbols[i] {
                insert_symbol_row(&tx, ps, file_id)?;
            }
        }
        tx.commit().into_diagnostic()
    }

    fn upsert_batch(
        &mut self,
        files: &[ProjectFile],
        symbols: &[Vec<ProjectSymbol>],
    ) -> Result<()> {
        let conn = self.storage.get_connection_mut();
        let tx = conn.unchecked_transaction().into_diagnostic()?;
        for (i, pf) in files.iter().enumerate() {
            upsert_file_row(&tx, pf)?;
            let file_id = get_file_id_by_path(&tx, &pf.file_path)?;
            for ps in &symbols[i] {
                insert_symbol_row(&tx, ps, file_id)?;
            }
        }
        tx.commit().into_diagnostic()
    }

    fn store_index_metadata(&mut self) -> Result<()> {
        let conn = self.storage.get_connection_mut();
        let now = chrono::Utc::now().to_rfc3339();
        for (key, value) in [
            ("parser_version", PARSER_VERSION),
            ("last_indexed_at", &now),
            ("index_version", "1"),
            ("schema_version", "1"),
        ] {
            conn.execute(
                "INSERT OR REPLACE INTO index_metadata (key, value) VALUES (?1, ?2)",
                (key, value),
            )
            .into_diagnostic()?;
        }
        Ok(())
    }

    fn load_existing_files(&self) -> Result<HashMap<String, ProjectFile>> {
        let conn = self.storage.get_connection();
        let mut stmt = conn.prepare("SELECT id, file_path, language, content_hash, git_blob_oid, file_size, mtime_ns, parser_version, parse_status, last_indexed_at FROM project_files WHERE parse_status != 'DELETED'").into_diagnostic()?;
        let rows = stmt
            .query_map([], |row| {
                Ok(ProjectFile {
                    id: Some(row.get(0)?),
                    file_path: row.get(1)?,
                    language: row.get(2)?,
                    content_hash: row.get(3)?,
                    git_blob_oid: row.get(4)?,
                    file_size: row.get(5)?,
                    mtime_ns: row.get(6)?,
                    parser_version: row.get(7)?,
                    parse_status: row.get(8)?,
                    last_indexed_at: row.get(9)?,
                })
            })
            .into_diagnostic()?
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()?;
        Ok(rows
            .into_iter()
            .map(|pf| (pf.file_path.clone(), pf))
            .collect())
    }

    pub fn delete_file_symbols(&mut self, file_path: &str) -> Result<()> {
        let conn = self.storage.get_connection_mut();
        delete_file_index_dependents(conn, file_path)?;
        conn.execute("DELETE FROM project_symbols WHERE file_id IN (SELECT id FROM project_files WHERE file_path = ?1)", [file_path]).into_diagnostic()?;
        Ok(())
    }
}

fn create_progress_bar(total: usize) -> ProgressBar {
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::with_template("Indexing: {pos}/{len} files... {spinner}")
            .unwrap_or_else(|_| ProgressStyle::with_template("{pos}/{len}").unwrap()),
    );
    pb
}

pub fn insert_file_row(conn: &Connection, pf: &ProjectFile) -> Result<()> {
    conn.execute("INSERT INTO project_files (file_path, language, content_hash, git_blob_oid, file_size, mtime_ns, parser_version, parse_status, last_indexed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![pf.file_path, pf.language, pf.content_hash, pf.git_blob_oid, pf.file_size, pf.mtime_ns, pf.parser_version, pf.parse_status, pf.last_indexed_at]).into_diagnostic()?;
    Ok(())
}

pub fn upsert_file_row(conn: &Connection, pf: &ProjectFile) -> Result<()> {
    conn.execute("INSERT INTO project_files (file_path, language, content_hash, git_blob_oid, file_size, mtime_ns, parser_version, parse_status, last_indexed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) ON CONFLICT(file_path) DO UPDATE SET language=excluded.language, content_hash=excluded.content_hash, git_blob_oid=excluded.git_blob_oid, file_size=excluded.file_size, mtime_ns=excluded.mtime_ns, parser_version=excluded.parser_version, parse_status=excluded.parse_status, last_indexed_at=excluded.last_indexed_at",
        rusqlite::params![pf.file_path, pf.language, pf.content_hash, pf.git_blob_oid, pf.file_size, pf.mtime_ns, pf.parser_version, pf.parse_status, pf.last_indexed_at]).into_diagnostic()?;
    Ok(())
}

pub fn get_file_id_by_path(conn: &Connection, file_path: &str) -> Result<i64> {
    conn.query_row(
        "SELECT id FROM project_files WHERE file_path = ?1",
        [file_path],
        |row| row.get(0),
    )
    .into_diagnostic()
}

pub fn delete_file_index_dependents(conn: &Connection, file_path: &str) -> Result<()> {
    let file_id_subquery = "SELECT id FROM project_files WHERE file_path = ?1";
    let symbol_id_subquery = "SELECT id FROM project_symbols WHERE file_id IN (SELECT id FROM project_files WHERE file_path = ?1)";
    for statement in [
        format!(
            "DELETE FROM symbol_centrality WHERE file_id IN ({file_id_subquery}) OR symbol_id IN ({symbol_id_subquery})"
        ),
        format!(
            "DELETE FROM structural_edges WHERE caller_file_id IN ({file_id_subquery}) OR callee_file_id IN ({file_id_subquery}) OR caller_symbol_id IN ({symbol_id_subquery}) OR callee_symbol_id IN ({symbol_id_subquery})"
        ),
        format!(
            "DELETE FROM api_routes WHERE handler_file_id IN ({file_id_subquery}) OR handler_symbol_id IN ({symbol_id_subquery})"
        ),
        format!("DELETE FROM data_models WHERE model_file_id IN ({file_id_subquery})"),
        format!("DELETE FROM observability_patterns WHERE file_id IN ({file_id_subquery})"),
        format!(
            "DELETE FROM test_mapping WHERE test_file_id IN ({file_id_subquery}) OR tested_file_id IN ({file_id_subquery}) OR test_symbol_id IN ({symbol_id_subquery}) OR tested_symbol_id IN ({symbol_id_subquery})"
        ),
        format!("DELETE FROM ci_gates WHERE ci_file_id IN ({file_id_subquery})"),
        format!(
            "DELETE FROM env_references WHERE file_id IN ({file_id_subquery}) OR symbol_id IN ({symbol_id_subquery})"
        ),
        format!("DELETE FROM env_declarations WHERE source_file_id IN ({file_id_subquery})"),
        format!("DELETE FROM project_docs WHERE file_id IN ({file_id_subquery})"),
    ] {
        conn.execute(&statement, [file_path]).into_diagnostic()?;
    }
    Ok(())
}

pub fn insert_symbol_row(conn: &Connection, ps: &ProjectSymbol, file_id: i64) -> Result<()> {
    conn.execute("INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, visibility, entrypoint_kind, is_public, cognitive_complexity, cyclomatic_complexity, line_start, line_end, byte_start, byte_end, signature_hash, confidence, evidence, last_indexed_at, metadata) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18) ON CONFLICT(file_id, qualified_name, symbol_kind) DO UPDATE SET symbol_name=excluded.symbol_name, visibility=excluded.visibility, entrypoint_kind=excluded.entrypoint_kind, is_public=excluded.is_public, cognitive_complexity=excluded.cognitive_complexity, cyclomatic_complexity=excluded.cyclomatic_complexity, line_start=excluded.line_start, line_end=excluded.line_end, byte_start=excluded.byte_start, byte_end=excluded.byte_end, signature_hash=excluded.signature_hash, confidence=excluded.confidence, evidence=excluded.evidence, last_indexed_at=excluded.last_indexed_at, metadata=excluded.metadata",
        rusqlite::params![file_id, ps.qualified_name, ps.symbol_name, ps.symbol_kind, ps.visibility, ps.entrypoint_kind, ps.is_public as i32, ps.cognitive_complexity, ps.cyclomatic_complexity, ps.line_start, ps.line_end, ps.byte_start, ps.byte_end, ps.signature_hash, ps.confidence, ps.evidence, ps.last_indexed_at, ps.metadata]).into_diagnostic()?;
    Ok(())
}
