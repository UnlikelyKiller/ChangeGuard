use crate::index::call_graph::{CallGraphBuilder, CallGraphStats};
use crate::index::docs::{DocIndexStats, parse_markdown};
use crate::index::entrypoint::{
    EntrypointKind, EntrypointStats, detect_python_entrypoints, detect_rust_entrypoints,
    detect_typescript_entrypoints,
};
use crate::index::languages::{Language, parse_symbols};
use crate::index::metrics::{ComplexityScorer, NativeComplexityScorer};
use crate::index::routes::{RouteExtractor, RouteStats};
use crate::index::symbols::Symbol;
use crate::index::topology::{DirectoryRole, TopologyIndexStats, classify_directory};
use crate::state::storage::StorageManager;
use camino::{Utf8Path, Utf8PathBuf};
use indicatif::{ProgressBar, ProgressStyle};
use miette::{IntoDiagnostic, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::time::Instant;
use tracing::{info, warn};

// --- Domain types mirroring project_files table ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    pub id: Option<i64>,
    pub file_path: String,
    pub language: Option<String>,
    pub content_hash: Option<String>,
    pub git_blob_oid: Option<String>,
    pub file_size: Option<i64>,
    pub mtime_ns: Option<i64>,
    pub parser_version: String,
    pub parse_status: String,
    pub last_indexed_at: String,
}

// --- Domain types mirroring project_symbols table ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSymbol {
    pub id: Option<i64>,
    pub file_id: i64,
    pub qualified_name: String,
    pub symbol_name: String,
    pub symbol_kind: String,
    pub visibility: Option<String>,
    pub entrypoint_kind: String,
    pub is_public: bool,
    pub cognitive_complexity: Option<i32>,
    pub cyclomatic_complexity: Option<i32>,
    pub line_start: Option<i32>,
    pub line_end: Option<i32>,
    pub byte_start: Option<i32>,
    pub byte_end: Option<i32>,
    pub signature_hash: Option<String>,
    pub confidence: f64,
    pub evidence: Option<String>,
    pub last_indexed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub files_indexed: usize,
    pub symbols_indexed: usize,
    pub parse_failures: usize,
    pub skipped_binary: usize,
    pub skipped_unsupported: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatus {
    pub total_files: usize,
    pub total_symbols: usize,
    pub stale_files: usize,
    pub last_indexed_at: Option<String>,
}

pub const MAX_FILES: usize = 10_000;
pub const BATCH_SIZE: usize = 500;

const BINARY_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "ico", "woff", "woff2", "ttf", "eot", "pdf", "zip", "tar", "gz",
    "exe", "dll", "so", "dylib", "wasm", "class", "jar", "pyc",
];

const SUPPORTED_EXTENSIONS: &[&str] = &["rs", "ts", "tsx", "js", "jsx", "py"];

const PARSER_VERSION: &str = "1";

pub struct ProjectIndexer {
    storage: StorageManager,
    repo_path: Utf8PathBuf,
}

impl ProjectIndexer {
    pub fn new(storage: StorageManager, repo_path: Utf8PathBuf) -> Self {
        Self { storage, repo_path }
    }

    /// Discover tracked files in the repository, filtering by supported language extensions
    /// and excluding binary files. Returns sorted for deterministic ordering.
    pub fn discover_files(&self) -> Result<Vec<Utf8PathBuf>> {
        let repo = gix::discover(&self.repo_path).into_diagnostic()?;
        let workdir = repo
            .workdir()
            .ok_or_else(|| miette::miette!("Bare repository has no work directory"))?;
        let workdir_path = Utf8PathBuf::from_path_buf(workdir.to_path_buf())
            .map_err(|_| miette::miette!("Work directory path is not valid UTF-8"))?;

        // Walk the working tree directory to find tracked files
        let mut files: Vec<Utf8PathBuf> = Vec::new();
        walk_tracked_files(&workdir_path, &mut files)?;

        // Filter by supported language extensions and exclude binary
        files.retain(|path| {
            let ext = path.extension().unwrap_or("");
            SUPPORTED_EXTENSIONS.contains(&ext) && !BINARY_EXTENSIONS.contains(&ext)
        });

        files.sort();
        Ok(files)
    }

    /// Parse a single file and return its ProjectFile row and any symbols found.
    /// On parse failure, returns a file row with parse_status = 'PARSE_FAILED'.
    pub fn index_file(&self, path: &Utf8Path) -> Result<(ProjectFile, Vec<ProjectSymbol>)> {
        let relative_path = path
            .strip_prefix(&self.repo_path)
            .unwrap_or(path)
            .to_string();

        let metadata = fs::symlink_metadata(path).into_diagnostic()?;
        let file_size = metadata.len() as i64;
        let mtime_ns = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos() as i64);

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                let now = chrono::Utc::now().to_rfc3339();
                let project_file = ProjectFile {
                    id: None,
                    file_path: relative_path,
                    language: None,
                    content_hash: None,
                    git_blob_oid: None,
                    file_size: Some(file_size),
                    mtime_ns,
                    parser_version: PARSER_VERSION.to_string(),
                    parse_status: "PARSE_FAILED".to_string(),
                    last_indexed_at: now,
                };
                warn!("Failed to read file {}: {}", path, e);
                return Ok((project_file, Vec::new()));
            }
        };

        let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
        let ext = path.extension().unwrap_or("");
        let language = Language::from_extension(ext).map(|l| format!("{:?}", l));

        let now = chrono::Utc::now().to_rfc3339();

        let (parse_status, mut symbols) = match parse_symbols(path.as_std_path(), &content) {
            Ok(Some(symbols)) => ("OK".to_string(), symbols),
            Ok(None) => ("OK".to_string(), Vec::new()),
            Err(e) => {
                warn!("Parse failure for {}: {}", path, e);
                ("PARSE_FAILED".to_string(), Vec::new())
            }
        };

        // Enrich symbols with complexity data from the tree-sitter scorer
        if parse_status == "OK"
            && let Some(lang) = Language::from_extension(ext)
        {
            let scorer = NativeComplexityScorer::new();
            if let Ok(file_complexity) = scorer.score_file(path, &content, lang) {
                // Merge complexity scores by matching symbol names
                for symbol in &mut symbols {
                    if let Some(sc) = file_complexity
                        .functions
                        .iter()
                        .find(|sc| sc.name == symbol.name)
                    {
                        symbol.cognitive_complexity = Some(sc.cognitive as i32);
                        symbol.cyclomatic_complexity = Some(sc.cyclomatic as i32);
                    }
                }
            }
        }

        let project_file = ProjectFile {
            id: None,
            file_path: relative_path.clone(),
            language,
            content_hash: Some(content_hash),
            git_blob_oid: None,
            file_size: Some(file_size),
            mtime_ns,
            parser_version: PARSER_VERSION.to_string(),
            parse_status,
            last_indexed_at: now.clone(),
        };

        let project_symbols: Vec<ProjectSymbol> = symbols
            .into_iter()
            .map(|s| symbol_to_project_symbol(&s, 0, &now))
            .collect();

        Ok((project_file, project_symbols))
    }

    /// Full index: clear existing data, scan all files, index everything.
    pub fn full_index(&mut self) -> Result<IndexStats> {
        let start = Instant::now();

        let mut files = self.discover_files()?;
        let total_discovered = files.len();

        if total_discovered > MAX_FILES {
            warn!(
                "Discovered {} source files, capping at {MAX_FILES}",
                total_discovered
            );
            files.truncate(MAX_FILES);
        }

        // Clear existing data
        self.clear_project_data()?;

        let pb = create_progress_bar(files.len());

        let mut files_indexed = 0usize;
        let mut symbols_indexed = 0usize;
        let mut parse_failures = 0usize;
        let mut batch_files: Vec<ProjectFile> = Vec::new();
        let mut batch_symbols: Vec<Vec<ProjectSymbol>> = Vec::new();

        for (i, file_path) in files.iter().enumerate() {
            let (project_file, project_symbols) = match self.index_file(file_path) {
                Ok(result) => result,
                Err(e) => {
                    warn!("Error indexing {}: {}", file_path, e);
                    parse_failures += 1;
                    pb.inc(1);
                    continue;
                }
            };

            if project_file.parse_status == "PARSE_FAILED" {
                parse_failures += 1;
            } else {
                files_indexed += 1;
            }
            symbols_indexed += project_symbols.len();

            batch_files.push(project_file);
            batch_symbols.push(project_symbols);

            if batch_files.len() >= BATCH_SIZE || i == files.len() - 1 {
                self.insert_batch(&batch_files, &batch_symbols)?;
                batch_files.clear();
                batch_symbols.clear();
            }

            pb.inc(1);
        }
        pb.finish_and_clear();

        // Store index metadata
        self.store_index_metadata()?;

        let duration_ms = start.elapsed().as_millis() as u64;

        info!(
            "Full index complete: {} files, {} symbols, {} parse failures in {}ms",
            files_indexed, symbols_indexed, parse_failures, duration_ms
        );

        Ok(IndexStats {
            files_indexed,
            symbols_indexed,
            parse_failures,
            skipped_binary: 0,
            skipped_unsupported: 0,
            duration_ms,
        })
    }

    /// Incremental index: only re-index files that changed since the last index.
    pub fn incremental_index(&mut self) -> Result<IndexStats> {
        let start = Instant::now();

        let current_files = self.discover_files()?;
        let current_file_set: HashSet<String> = current_files
            .iter()
            .map(|p| p.strip_prefix(&self.repo_path).unwrap_or(p).to_string())
            .collect();

        // Check if parser_version changed globally — force full re-index if so
        let stored_parser_version = self.get_metadata_value("parser_version");
        if stored_parser_version.as_deref() != Some(PARSER_VERSION) {
            info!("Parser version changed, performing full re-index");
            return self.full_index();
        }

        // Load existing file records
        let existing_files = self.load_existing_files()?;
        let existing_paths: HashSet<String> = existing_files.keys().cloned().collect();

        // Determine which files need re-indexing
        let mut files_to_reindex = Vec::new();
        let mut stale_count = 0usize;

        for file_path in &current_files {
            let relative = file_path
                .strip_prefix(&self.repo_path)
                .unwrap_or(file_path)
                .to_string();

            if let Some(existing) = existing_files.get(&relative) {
                let current_hash = self.compute_file_hash(file_path)?;
                if existing.content_hash.as_deref() != Some(&current_hash) {
                    files_to_reindex.push(file_path.clone());
                    stale_count += 1;
                }
            } else {
                // New file
                files_to_reindex.push(file_path.clone());
                stale_count += 1;
            }
        }

        // Mark deleted files
        for existing_path in &existing_paths {
            if !current_file_set.contains(existing_path) {
                self.mark_file_deleted(existing_path)?;
                stale_count += 1;
            }
        }

        if stale_count == 0 {
            let duration_ms = start.elapsed().as_millis() as u64;
            info!(
                "Incremental index: no changes detected in {}ms",
                duration_ms
            );
            return Ok(IndexStats {
                files_indexed: 0,
                symbols_indexed: 0,
                parse_failures: 0,
                skipped_binary: 0,
                skipped_unsupported: 0,
                duration_ms,
            });
        }

        // Re-index stale files
        let pb = create_progress_bar(files_to_reindex.len());

        let mut files_indexed = 0usize;
        let mut symbols_indexed = 0usize;
        let mut parse_failures = 0usize;
        let mut batch_files: Vec<ProjectFile> = Vec::new();
        let mut batch_symbols: Vec<Vec<ProjectSymbol>> = Vec::new();

        for (i, file_path) in files_to_reindex.iter().enumerate() {
            let (project_file, project_symbols) = match self.index_file(file_path) {
                Ok(result) => result,
                Err(e) => {
                    warn!("Error indexing {}: {}", file_path, e);
                    parse_failures += 1;
                    pb.inc(1);
                    continue;
                }
            };

            if project_file.parse_status == "PARSE_FAILED" {
                parse_failures += 1;
            } else {
                files_indexed += 1;
            }
            symbols_indexed += project_symbols.len();

            // Delete old symbols for this file before re-inserting
            let _ = self.delete_file_symbols(&project_file.file_path);

            batch_files.push(project_file);
            batch_symbols.push(project_symbols);

            if batch_files.len() >= BATCH_SIZE || i == files_to_reindex.len() - 1 {
                self.upsert_batch(&batch_files, &batch_symbols)?;
                batch_files.clear();
                batch_symbols.clear();
            }

            pb.inc(1);
        }
        pb.finish_and_clear();

        // Update index metadata
        self.store_index_metadata()?;

        let duration_ms = start.elapsed().as_millis() as u64;
        info!(
            "Incremental index complete: {} files re-indexed, {} symbols in {}ms",
            files_indexed, symbols_indexed, duration_ms
        );

        Ok(IndexStats {
            files_indexed,
            symbols_indexed,
            parse_failures,
            skipped_binary: 0,
            skipped_unsupported: 0,
            duration_ms,
        })
    }

    /// Check index status without modifying the database.
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

            let current_hash = self.compute_file_hash(file_path)?;
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

    /// Query a single file by path.
    pub fn file_for_path(&self, path: &str) -> Result<Option<ProjectFile>> {
        let conn = self.storage.get_connection();
        let result = conn.query_row(
            "SELECT id, file_path, language, content_hash, git_blob_oid, file_size, mtime_ns, \
             parser_version, parse_status, last_indexed_at \
             FROM project_files WHERE file_path = ?1",
            [path],
            |row| {
                Ok(ProjectFile {
                    id: Some(row.get::<_, i64>(0)?),
                    file_path: row.get::<_, String>(1)?,
                    language: row.get::<_, Option<String>>(2)?,
                    content_hash: row.get::<_, Option<String>>(3)?,
                    git_blob_oid: row.get::<_, Option<String>>(4)?,
                    file_size: row.get::<_, Option<i64>>(5)?,
                    mtime_ns: row.get::<_, Option<i64>>(6)?,
                    parser_version: row.get::<_, String>(7)?,
                    parse_status: row.get::<_, String>(8)?,
                    last_indexed_at: row.get::<_, String>(9)?,
                })
            },
        );

        match result {
            Ok(pf) => Ok(Some(pf)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).into_diagnostic(),
        }
    }

    /// Query symbols for a file by file_id.
    pub fn symbols_for_file(&self, file_id: i64) -> Result<Vec<ProjectSymbol>> {
        let conn = self.storage.get_connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, file_id, qualified_name, symbol_name, symbol_kind, visibility, \
             entrypoint_kind, is_public, cognitive_complexity, cyclomatic_complexity, \
             line_start, line_end, byte_start, byte_end, signature_hash, confidence, \
             evidence, last_indexed_at \
             FROM project_symbols WHERE file_id = ?1",
            )
            .into_diagnostic()?;

        let symbols = stmt
            .query_map([file_id], |row| {
                Ok(ProjectSymbol {
                    id: Some(row.get::<_, i64>(0)?),
                    file_id: row.get::<_, i64>(1)?,
                    qualified_name: row.get::<_, String>(2)?,
                    symbol_name: row.get::<_, String>(3)?,
                    symbol_kind: row.get::<_, String>(4)?,
                    visibility: row.get::<_, Option<String>>(5)?,
                    entrypoint_kind: row.get::<_, String>(6)?,
                    is_public: row.get::<_, i32>(7)? != 0,
                    cognitive_complexity: row.get::<_, Option<i32>>(8)?,
                    cyclomatic_complexity: row.get::<_, Option<i32>>(9)?,
                    line_start: row.get::<_, Option<i32>>(10)?,
                    line_end: row.get::<_, Option<i32>>(11)?,
                    byte_start: row.get::<_, Option<i32>>(12)?,
                    byte_end: row.get::<_, Option<i32>>(13)?,
                    signature_hash: row.get::<_, Option<String>>(14)?,
                    confidence: row.get::<_, f64>(15)?,
                    evidence: row.get::<_, Option<String>>(16)?,
                    last_indexed_at: row.get::<_, String>(17)?,
                })
            })
            .into_diagnostic()?
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()?;

        Ok(symbols)
    }

    // --- Private helpers ---

    fn clear_project_data(&mut self) -> Result<()> {
        let conn = self.storage.get_connection_mut();
        conn.execute("DELETE FROM project_symbols", [])
            .into_diagnostic()?;
        conn.execute("DELETE FROM project_files", [])
            .into_diagnostic()?;
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

        tx.commit().into_diagnostic()?;
        Ok(())
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

        tx.commit().into_diagnostic()?;
        Ok(())
    }

    fn store_index_metadata(&mut self) -> Result<()> {
        let conn = self.storage.get_connection_mut();
        let now = chrono::Utc::now().to_rfc3339();

        let metadata = [
            ("parser_version", PARSER_VERSION),
            ("last_indexed_at", &now),
            ("index_version", "1"),
            ("schema_version", "1"),
        ];

        for (key, value) in metadata {
            conn.execute(
                "INSERT OR REPLACE INTO index_metadata (key, value) VALUES (?1, ?2)",
                (key, value),
            )
            .into_diagnostic()?;
        }

        Ok(())
    }

    fn get_metadata_value(&self, key: &str) -> Option<String> {
        let conn = self.storage.get_connection();
        conn.query_row(
            "SELECT value FROM index_metadata WHERE key = ?1",
            [key],
            |row| row.get::<_, String>(0),
        )
        .ok()
    }

    fn load_existing_files(&self) -> Result<std::collections::HashMap<String, ProjectFile>> {
        let conn = self.storage.get_connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, file_path, language, content_hash, git_blob_oid, file_size, \
                 mtime_ns, parser_version, parse_status, last_indexed_at \
                 FROM project_files WHERE parse_status != 'DELETED'",
            )
            .into_diagnostic()?;

        let rows = stmt
            .query_map([], |row| {
                Ok(ProjectFile {
                    id: Some(row.get::<_, i64>(0)?),
                    file_path: row.get::<_, String>(1)?,
                    language: row.get::<_, Option<String>>(2)?,
                    content_hash: row.get::<_, Option<String>>(3)?,
                    git_blob_oid: row.get::<_, Option<String>>(4)?,
                    file_size: row.get::<_, Option<i64>>(5)?,
                    mtime_ns: row.get::<_, Option<i64>>(6)?,
                    parser_version: row.get::<_, String>(7)?,
                    parse_status: row.get::<_, String>(8)?,
                    last_indexed_at: row.get::<_, String>(9)?,
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

    fn mark_file_deleted(&mut self, file_path: &str) -> Result<()> {
        let conn = self.storage.get_connection_mut();
        conn.execute(
            "UPDATE project_files SET parse_status = 'DELETED' WHERE file_path = ?1",
            [file_path],
        )
        .into_diagnostic()?;
        conn.execute(
            "DELETE FROM project_symbols WHERE file_id IN \
             (SELECT id FROM project_files WHERE file_path = ?1)",
            [file_path],
        )
        .into_diagnostic()?;
        Ok(())
    }

    fn delete_file_symbols(&mut self, file_path: &str) -> Result<()> {
        let conn = self.storage.get_connection_mut();
        conn.execute(
            "DELETE FROM project_symbols WHERE file_id IN \
             (SELECT id FROM project_files WHERE file_path = ?1)",
            [file_path],
        )
        .into_diagnostic()?;
        Ok(())
    }

    fn compute_file_hash(&self, path: &Utf8Path) -> Result<String> {
        let content = fs::read_to_string(path).into_diagnostic()?;
        Ok(blake3::hash(content.as_bytes()).to_hex().to_string())
    }

    /// Discover documentation files (README.md, CONTRIBUTING.md, ARCHITECTURE.md,
    /// plus any .md files linked from README.md, one level deep).
    pub fn discover_doc_files(&self) -> Result<Vec<Utf8PathBuf>> {
        let mut doc_files = Vec::new();
        let priority_files = ["README.md", "CONTRIBUTING.md", "ARCHITECTURE.md"];

        for name in &priority_files {
            let path = self.repo_path.join(name);
            if path.exists() {
                doc_files.push(path);
            }
        }

        // Follow internal links from README.md (one level deep)
        let readme_path = self.repo_path.join("README.md");
        if readme_path.exists()
            && let Ok(content) = fs::read_to_string(&readme_path)
        {
            let parsed = parse_markdown(&content, "README.md");
            for link in &parsed.internal_links {
                let linked_path = self.repo_path.join(&link.target);
                if linked_path.exists()
                    && linked_path.extension().is_some_and(|e| e == "md")
                    && !doc_files.contains(&linked_path)
                {
                    doc_files.push(linked_path);
                }
            }
        }

        // Also check docs/ directory for ARCHITECTURE.md
        let docs_arch = self.repo_path.join("docs").join("ARCHITECTURE.md");
        if docs_arch.exists() && !doc_files.contains(&docs_arch) {
            doc_files.push(docs_arch);
        }

        doc_files.sort();
        doc_files.dedup();
        Ok(doc_files)
    }

    /// Index documentation files into the project_docs table.
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

            // Find or create the project_files entry for this doc
            let file_id = self.ensure_file_entry(&relative_path, &content, &now)?;

            // Upsert the project_docs entry
            let sections_json =
                serde_json::to_string(&parsed.sections).unwrap_or_else(|_| "[]".to_string());
            let code_blocks_json =
                serde_json::to_string(&parsed.code_blocks).unwrap_or_else(|_| "[]".to_string());
            let internal_links_json =
                serde_json::to_string(&parsed.internal_links).unwrap_or_else(|_| "[]".to_string());

            let conn = self.storage.get_connection_mut();
            conn.execute(
                "INSERT OR REPLACE INTO project_docs \
                 (file_id, title, summary, sections, code_blocks, internal_links, confidence, last_indexed_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    file_id,
                    parsed.title,
                    parsed.summary,
                    sections_json,
                    code_blocks_json,
                    internal_links_json,
                    1.0_f64,
                    now,
                ],
            ).into_diagnostic()?;

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

        // Check if file already exists
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

        // Insert new file entry
        let conn = self.storage.get_connection_mut();
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, git_blob_oid, \
             file_size, mtime_ns, parser_version, parse_status, last_indexed_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                relative_path,
                "Markdown",
                content_hash,
                Option::<String>::None,
                content.len() as i64,
                Option::<i64>::None,
                PARSER_VERSION,
                "OK",
                now,
            ],
        )
        .into_diagnostic()?;

        Ok(conn.last_insert_rowid())
    }

    /// Index directory topology by classifying all directories in the repo.
    pub fn index_topology(&mut self) -> Result<TopologyIndexStats> {
        let all_files = self.discover_files()?;
        let now = chrono::Utc::now().to_rfc3339();

        // Group files by directory
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

        // Also add intermediate directories that may not have files directly
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
                let role_str = classification.role.as_str();
                conn.execute(
                    "INSERT OR REPLACE INTO project_topology (dir_path, role, confidence, evidence, last_indexed_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        dir_path,
                        role_str,
                        classification.confidence,
                        classification.evidence,
                        now,
                    ],
                ).into_diagnostic()?;

                *role_counts.entry(classification.role).or_insert(0) += 1;
                directories_classified += 1;
            } else {
                unclassified += 1;
            }
        }

        info!(
            "Topology index complete: {} directories classified, {} unclassified",
            directories_classified, unclassified
        );

        Ok(TopologyIndexStats {
            directories_classified,
            unclassified,
            role_counts,
        })
    }

    /// Classify entry points for all indexed symbols.
    pub fn classify_entrypoints(&mut self) -> Result<EntrypointStats> {
        let conn = self.storage.get_connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, file_id, symbol_name, symbol_kind, is_public FROM project_symbols \
                 ORDER BY file_id",
            )
            .into_diagnostic()?;

        let rows: Vec<(i64, i64, String, String, bool)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i32>(4)? != 0,
                ))
            })
            .into_diagnostic()?
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()?;

        drop(stmt);

        // Group symbols by file_id
        let mut file_symbols: HashMap<i64, Vec<(i64, String, String, bool)>> = HashMap::new();
        for (id, file_id, name, kind, is_public) in &rows {
            file_symbols.entry(*file_id).or_default().push((
                *id,
                name.clone(),
                kind.clone(),
                *is_public,
            ));
        }

        // Get file paths for each file_id
        let mut file_paths: HashMap<i64, String> = HashMap::new();
        let conn2 = self.storage.get_connection();
        let mut path_stmt = conn2
            .prepare("SELECT id, file_path, language FROM project_files")
            .into_diagnostic()?;
        let path_rows: Vec<(i64, String, Option<String>)> = path_stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .into_diagnostic()?
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()?;
        drop(path_stmt);

        for (id, path, _lang) in &path_rows {
            file_paths.insert(*id, path.clone());
        }

        let mut stats = EntrypointStats::default();
        let conn3 = self.storage.get_connection_mut();
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

            // Read file content for detection
            let full_path = self.repo_path.join(&file_path);
            let content = match fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Build Symbol structs for detection
            let sym_vec: Vec<Symbol> = symbols
                .iter()
                .map(|(_, name, kind, is_public)| Symbol {
                    name: name.clone(),
                    kind: match kind.as_str() {
                        "Function" => crate::index::symbols::SymbolKind::Function,
                        "Method" => crate::index::symbols::SymbolKind::Method,
                        "Class" => crate::index::symbols::SymbolKind::Class,
                        "Struct" => crate::index::symbols::SymbolKind::Struct,
                        "Enum" => crate::index::symbols::SymbolKind::Enum,
                        "Trait" => crate::index::symbols::SymbolKind::Trait,
                        "Interface" => crate::index::symbols::SymbolKind::Interface,
                        "Type" => crate::index::symbols::SymbolKind::Type,
                        "Variable" => crate::index::symbols::SymbolKind::Variable,
                        "Constant" => crate::index::symbols::SymbolKind::Constant,
                        "Module" => crate::index::symbols::SymbolKind::Module,
                        _ => crate::index::symbols::SymbolKind::Function,
                    },
                    is_public: *is_public,
                    cognitive_complexity: None,
                    cyclomatic_complexity: None,
                    line_start: None,
                    line_end: None,
                    qualified_name: None,
                    byte_start: None,
                    byte_end: None,
                    entrypoint_kind: None,
                })
                .collect();

            // Dispatch to language-specific detector
            let classifications = match file_lang.as_deref() {
                Some("Rust") => detect_rust_entrypoints(&content, &sym_vec),
                Some("TypeScript") | Some("JavaScript") => {
                    detect_typescript_entrypoints(&content, &sym_vec, &file_path)
                }
                Some("Python") => detect_python_entrypoints(&content, &sym_vec, &file_path),
                _ => continue,
            };

            // Update entrypoint_kind for each classified symbol
            for class in &classifications {
                // Find the corresponding DB row
                let db_id = symbols
                    .iter()
                    .find(|(_, name, _, _)| name == &class.symbol_name)
                    .map(|(id, _, _, _)| *id);

                if let Some(id) = db_id {
                    conn3.execute(
                        "UPDATE project_symbols SET entrypoint_kind = ?1, confidence = ?2, evidence = ?3, last_indexed_at = ?4 WHERE id = ?5",
                        rusqlite::params![
                            class.kind.as_str(),
                            class.confidence,
                            class.evidence,
                            now,
                            id,
                        ],
                    ).into_diagnostic()?;

                    match class.kind {
                        EntrypointKind::Entrypoint => stats.entrypoints += 1,
                        EntrypointKind::Handler => stats.handlers += 1,
                        EntrypointKind::PublicApi => stats.public_apis += 1,
                        EntrypointKind::Test => stats.tests += 1,
                        EntrypointKind::Internal => stats.internal += 1,
                    }
                }
            }
        }

        info!(
            "Entrypoint classification complete: {} entrypoints, {} handlers, {} public APIs, {} tests, {} internal",
            stats.entrypoints, stats.handlers, stats.public_apis, stats.tests, stats.internal
        );

        Ok(stats)
    }

    /// Build the call graph: extract call edges from source files and resolve callees.
    pub fn build_call_graph(&self) -> Result<CallGraphStats> {
        let builder =
            CallGraphBuilder::new(&self.storage, self.repo_path.as_std_path().to_path_buf());
        builder.build()
    }

    /// Extract API routes from source files and store them in the api_routes table.
    pub fn extract_routes(&self) -> Result<RouteStats> {
        let extractor =
            RouteExtractor::new(&self.storage, self.repo_path.as_std_path().to_path_buf());
        extractor.extract()
    }

    /// Delete API routes where the handler belongs to any of the given file IDs.
    /// Used for incremental re-indexing of specific files.
    pub fn clear_routes(&self, file_ids: &[i64]) -> Result<()> {
        let extractor =
            RouteExtractor::new(&self.storage, self.repo_path.as_std_path().to_path_buf());
        extractor.clear_routes(file_ids)
    }

    /// Delete structural edges where the caller belongs to any of the given file IDs.
    /// Used for incremental re-indexing of specific files.
    pub fn clear_structural_edges(&self, file_ids: &[i64]) -> Result<()> {
        if file_ids.is_empty() {
            return Ok(());
        }

        let conn = self.storage.get_connection();
        // Delete one by one to avoid dynamic Params trait issues with rusqlite
        for &fid in file_ids {
            conn.execute(
                "DELETE FROM structural_edges WHERE caller_file_id = ?1",
                [fid],
            )
            .into_diagnostic()?;
        }
        Ok(())
    }
}

// --- Helper: walk tracked files from the working tree ---

fn walk_tracked_files(dir: &Utf8Path, files: &mut Vec<Utf8PathBuf>) -> Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|_| miette::miette!("Invalid UTF-8 path"))?;

        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        // Skip hidden dirs and known non-source dirs
        if file_name_str.starts_with('.')
            || matches!(
                file_name_str.as_ref(),
                "target" | "node_modules" | "dist" | "build"
            )
        {
            continue;
        }

        if path.is_dir() {
            walk_tracked_files(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

// --- Helper: create progress bar ---

fn create_progress_bar(total: usize) -> ProgressBar {
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::with_template("Indexing: {pos}/{len} files... {spinner}")
            .unwrap_or_else(|_| ProgressStyle::with_template("{pos}/{len}").unwrap()),
    );
    pb
}

// --- SQL helper functions ---

fn insert_file_row(conn: &Connection, pf: &ProjectFile) -> Result<()> {
    conn.execute(
        "INSERT INTO project_files (file_path, language, content_hash, git_blob_oid, \
         file_size, mtime_ns, parser_version, parse_status, last_indexed_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            pf.file_path,
            pf.language,
            pf.content_hash,
            pf.git_blob_oid,
            pf.file_size,
            pf.mtime_ns,
            pf.parser_version,
            pf.parse_status,
            pf.last_indexed_at,
        ],
    )
    .into_diagnostic()?;
    Ok(())
}

fn upsert_file_row(conn: &Connection, pf: &ProjectFile) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO project_files \
         (file_path, language, content_hash, git_blob_oid, file_size, mtime_ns, \
          parser_version, parse_status, last_indexed_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            pf.file_path,
            pf.language,
            pf.content_hash,
            pf.git_blob_oid,
            pf.file_size,
            pf.mtime_ns,
            pf.parser_version,
            pf.parse_status,
            pf.last_indexed_at,
        ],
    )
    .into_diagnostic()?;
    Ok(())
}

fn get_file_id_by_path(conn: &Connection, file_path: &str) -> Result<i64> {
    conn.query_row(
        "SELECT id FROM project_files WHERE file_path = ?1",
        [file_path],
        |row| row.get::<_, i64>(0),
    )
    .into_diagnostic()
}

fn insert_symbol_row(conn: &Connection, ps: &ProjectSymbol, file_id: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO project_symbols \
         (file_id, qualified_name, symbol_name, symbol_kind, visibility, \
          entrypoint_kind, is_public, cognitive_complexity, cyclomatic_complexity, \
          line_start, line_end, byte_start, byte_end, signature_hash, confidence, \
          evidence, last_indexed_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        rusqlite::params![
            file_id,
            ps.qualified_name,
            ps.symbol_name,
            ps.symbol_kind,
            ps.visibility,
            ps.entrypoint_kind,
            ps.is_public as i32,
            ps.cognitive_complexity,
            ps.cyclomatic_complexity,
            ps.line_start,
            ps.line_end,
            ps.byte_start,
            ps.byte_end,
            ps.signature_hash,
            ps.confidence,
            ps.evidence,
            ps.last_indexed_at,
        ],
    )
    .into_diagnostic()?;
    Ok(())
}

fn symbol_to_project_symbol(s: &Symbol, file_id: i64, now: &str) -> ProjectSymbol {
    let qualified_name = s.qualified_name.clone().unwrap_or_else(|| s.name.clone());
    let visibility = if s.is_public {
        Some("public".to_string())
    } else {
        Some("private".to_string())
    };

    ProjectSymbol {
        id: None,
        file_id,
        qualified_name,
        symbol_name: s.name.clone(),
        symbol_kind: format!("{:?}", s.kind),
        visibility,
        entrypoint_kind: "INTERNAL".to_string(),
        is_public: s.is_public,
        cognitive_complexity: s.cognitive_complexity,
        cyclomatic_complexity: s.cyclomatic_complexity,
        line_start: s.line_start,
        line_end: s.line_end,
        byte_start: s.byte_start,
        byte_end: s.byte_end,
        signature_hash: None,
        confidence: 1.0,
        evidence: None,
        last_indexed_at: now.to_string(),
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::migrations::get_migrations;
    use rusqlite::Connection;

    fn in_memory_storage() -> StorageManager {
        let conn = Connection::open_in_memory().unwrap();
        let mut conn = conn;
        get_migrations().to_latest(&mut conn).unwrap();
        StorageManager::init_from_conn(conn)
    }

    #[test]
    fn test_binary_extensions_filtered() {
        assert!(BINARY_EXTENSIONS.contains(&"png"));
        assert!(BINARY_EXTENSIONS.contains(&"exe"));
        assert!(BINARY_EXTENSIONS.contains(&"dll"));
    }

    #[test]
    fn test_supported_extensions() {
        assert!(SUPPORTED_EXTENSIONS.contains(&"rs"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"ts"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"py"));
        assert!(!SUPPORTED_EXTENSIONS.contains(&"java"));
    }

    #[test]
    fn test_symbol_to_project_symbol() {
        use crate::index::symbols::{Symbol, SymbolKind};

        let symbol = Symbol {
            name: "my_function".to_string(),
            kind: SymbolKind::Function,
            is_public: true,
            cognitive_complexity: Some(5),
            cyclomatic_complexity: Some(3),
            line_start: Some(10),
            line_end: Some(20),
            qualified_name: Some("MyModule::my_function".to_string()),
            byte_start: None,
            byte_end: None,
            entrypoint_kind: None,
        };

        let ps = symbol_to_project_symbol(&symbol, 42, "2026-01-01T00:00:00Z");

        assert_eq!(ps.file_id, 42);
        assert_eq!(ps.qualified_name, "MyModule::my_function");
        assert_eq!(ps.symbol_name, "my_function");
        assert_eq!(ps.symbol_kind, "Function");
        assert_eq!(ps.visibility, Some("public".to_string()));
        assert_eq!(ps.entrypoint_kind, "INTERNAL");
        assert!(ps.is_public);
        assert_eq!(ps.cognitive_complexity, Some(5));
        assert_eq!(ps.cyclomatic_complexity, Some(3));
        assert_eq!(ps.line_start, Some(10));
        assert_eq!(ps.line_end, Some(20));
        assert_eq!(ps.confidence, 1.0);
    }

    #[test]
    fn test_insert_and_query_project_file() {
        let storage = in_memory_storage();
        let indexer = ProjectIndexer::new(storage, Utf8PathBuf::from("/tmp/test_repo"));

        let pf = ProjectFile {
            id: None,
            file_path: "src/main.rs".to_string(),
            language: Some("Rust".to_string()),
            content_hash: Some("abc123".to_string()),
            git_blob_oid: None,
            file_size: Some(1024),
            mtime_ns: None,
            parser_version: PARSER_VERSION.to_string(),
            parse_status: "OK".to_string(),
            last_indexed_at: "2026-01-01T00:00:00Z".to_string(),
        };

        let conn = indexer.storage.get_connection();
        let tx = conn.unchecked_transaction().unwrap();
        insert_file_row(&tx, &pf).unwrap();
        let file_id = tx.last_insert_rowid();
        tx.commit().unwrap();

        assert!(file_id > 0);

        let result = indexer.file_for_path("src/main.rs").unwrap().unwrap();
        assert_eq!(result.file_path, "src/main.rs");
        assert_eq!(result.language, Some("Rust".to_string()));
        assert_eq!(result.parse_status, "OK");
    }

    #[test]
    fn test_insert_and_query_project_symbols() {
        let storage = in_memory_storage();
        let indexer = ProjectIndexer::new(storage, Utf8PathBuf::from("/tmp/test_repo"));

        let now = "2026-01-01T00:00:00Z";

        let pf = ProjectFile {
            id: None,
            file_path: "src/lib.rs".to_string(),
            language: Some("Rust".to_string()),
            content_hash: Some("hash123".to_string()),
            git_blob_oid: None,
            file_size: Some(500),
            mtime_ns: None,
            parser_version: PARSER_VERSION.to_string(),
            parse_status: "OK".to_string(),
            last_indexed_at: now.to_string(),
        };

        let conn = indexer.storage.get_connection();
        let tx = conn.unchecked_transaction().unwrap();
        insert_file_row(&tx, &pf).unwrap();
        let file_id = tx.last_insert_rowid();

        let ps = ProjectSymbol {
            id: None,
            file_id,
            qualified_name: "my_crate::hello".to_string(),
            symbol_name: "hello".to_string(),
            symbol_kind: "Function".to_string(),
            visibility: Some("public".to_string()),
            entrypoint_kind: "INTERNAL".to_string(),
            is_public: true,
            cognitive_complexity: Some(3),
            cyclomatic_complexity: Some(2),
            line_start: Some(1),
            line_end: Some(10),
            byte_start: None,
            byte_end: None,
            signature_hash: None,
            confidence: 1.0,
            evidence: None,
            last_indexed_at: now.to_string(),
        };

        insert_symbol_row(&tx, &ps, file_id).unwrap();
        tx.commit().unwrap();

        let symbols = indexer.symbols_for_file(file_id).unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].symbol_name, "hello");
        assert_eq!(symbols[0].qualified_name, "my_crate::hello");
        assert_eq!(symbols[0].cognitive_complexity, Some(3));
    }

    #[test]
    fn test_mark_file_deleted() {
        let storage = in_memory_storage();
        let mut indexer = ProjectIndexer::new(storage, Utf8PathBuf::from("/tmp/test_repo"));

        let pf = ProjectFile {
            id: None,
            file_path: "src/deleted.rs".to_string(),
            language: Some("Rust".to_string()),
            content_hash: Some("hash456".to_string()),
            git_blob_oid: None,
            file_size: Some(200),
            mtime_ns: None,
            parser_version: PARSER_VERSION.to_string(),
            parse_status: "OK".to_string(),
            last_indexed_at: "2026-01-01T00:00:00Z".to_string(),
        };

        let conn = indexer.storage.get_connection();
        let tx = conn.unchecked_transaction().unwrap();
        insert_file_row(&tx, &pf).unwrap();
        tx.commit().unwrap();

        indexer.mark_file_deleted("src/deleted.rs").unwrap();

        let result = indexer.file_for_path("src/deleted.rs").unwrap().unwrap();
        assert_eq!(result.parse_status, "DELETED");
    }

    #[test]
    fn test_index_stats_serialization() {
        let stats = IndexStats {
            files_indexed: 100,
            symbols_indexed: 500,
            parse_failures: 2,
            skipped_binary: 10,
            skipped_unsupported: 5,
            duration_ms: 3000,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("files_indexed"));
        assert!(json.contains("symbols_indexed"));
    }

    #[test]
    fn test_index_status_serialization() {
        let status = IndexStatus {
            total_files: 50,
            total_symbols: 200,
            stale_files: 3,
            last_indexed_at: Some("2026-01-01T00:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("total_files"));
        assert!(json.contains("stale_files"));
    }
}
