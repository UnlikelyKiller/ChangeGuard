use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::snippet::SnippetGenerator;
use tantivy::tokenizer::{LowerCaser, TextAnalyzer, WhitespaceTokenizer};
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy};

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    pub path: String,
    pub line_count: usize,
    pub score: f32,
    pub snippet: Option<String>,
    pub line_number: Option<usize>,
}

pub struct TantivySearchEngine {
    index: Index,
    reader: IndexReader,
    schema: Schema,
}

impl TantivySearchEngine {
    pub fn open_or_create(path: &Path) -> Result<Self> {
        // Build schema.  The trigrams field uses "code_trigram" (registered
        // below) so that WhitespaceTokenizer preserves underscore-containing
        // tokens like "te_" / "e_s" / "_sc" intact.
        // @cg-tx: baff3d54-a2ba-4c1c-ac8a-99bdd2435221 (Track J2)
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("path", TEXT | STORED);
        schema_builder.add_text_field("content", TEXT | STORED);
        schema_builder.add_u64_field("line_count", STORED);
        schema_builder.add_text_field("language", TEXT | STORED);

        // Trigrams are space-separated; use a whitespace tokenizer + lower-caser
        // so that cross-underscore trigrams (e.g. "te_", "_sc") survive ingestion.
        // SimpleTokenizer (the TEXT default) treats '_' as a word boundary and
        // destroys these tokens.
        schema_builder.add_text_field(
            "trigrams",
            TextOptions::default().set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("code_trigram")
                    .set_index_option(IndexRecordOption::Basic),
            ),
        );
        let schema = schema_builder.build();

        if !path.exists() {
            std::fs::create_dir_all(path).into_diagnostic()?;
        }

        let index = Index::open_or_create(
            tantivy::directory::MmapDirectory::open(path).into_diagnostic()?,
            schema.clone(),
        )
        .into_diagnostic()?;

        // Register the custom tokenizer BEFORE any write or search operations.
        // WhitespaceTokenizer splits only on whitespace, preserving '_' within
        // trigrams.  LowerCaser normalises case to match query-time lowercasing.
        // If this index was created with an older schema (before J2), Tantivy
        // will raise an error about an unregistered tokenizer; the caller
        // should run `changeguard index --semantic` to rebuild.
        index.tokenizers().register(
            "code_trigram",
            TextAnalyzer::builder(WhitespaceTokenizer::default())
                .filter(LowerCaser)
                .build(),
        );

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .into_diagnostic()?;

        Ok(Self {
            index,
            reader,
            schema,
        })
    }

    pub fn get_writer(&self, memory_budget_bytes: usize) -> Result<IndexWriter> {
        self.index.writer(memory_budget_bytes).into_diagnostic()
    }

    /// Force the reader to reload so that recently committed segments are visible.
    /// Useful in tests and after large batch indexing operations.
    pub fn reload_reader(&self) -> Result<()> {
        self.reader.reload().into_diagnostic()
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let searcher = self.reader.searcher();
        let content_field = self.schema.get_field("content").unwrap();
        let path_field = self.schema.get_field("path").unwrap();
        let line_count_field = self.schema.get_field("line_count").unwrap();

        let query_parser = QueryParser::for_index(&self.index, vec![content_field, path_field]);
        let query = query_parser.parse_query(query_str).into_diagnostic()?;

        let snippet_generator = SnippetGenerator::create(&searcher, &*query, content_field).into_diagnostic()?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit).order_by_score())
            .into_diagnostic()?;

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address).into_diagnostic()?;

            let path = retrieved_doc
                .get_first(path_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let line_count = retrieved_doc
                .get_first(line_count_field)
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;

            let mut snippet_opt = None;
            let mut line_number_opt = None;

            if let Some(content_val) = retrieved_doc.get_first(content_field).and_then(|v| v.as_str()) {
                let snippet = snippet_generator.snippet_from_doc(&retrieved_doc);
                let highlighted = snippet.to_html();
                if !highlighted.is_empty() {
                    snippet_opt = Some(highlighted.replace("<b>", "\x1b[1m").replace("</b>", "\x1b[0m"));
                    // Heuristic for line number: find the first match in the content by looking at the snippet.
                    let plain_snippet = snippet.fragment();
                    if let Some(idx) = content_val.find(plain_snippet) {
                        let lines_before = content_val[..idx].chars().filter(|&c| c == '\n').count();
                        line_number_opt = Some(lines_before + 1);
                    } else {
                        line_number_opt = Some(1);
                    }
                }
            }

            results.push(SearchResult {
                path,
                line_count,
                score,
                snippet: snippet_opt,
                line_number: line_number_opt,
            });
        }

        Ok(results)
    }

    pub fn search_trigrams(&self, trigrams: &[String], limit: usize) -> Result<Vec<String>> {
        use tantivy::query::BooleanQuery;
        use tantivy::query::TermQuery;
        use tantivy::schema::IndexRecordOption;

        let searcher = self.reader.searcher();
        let trigrams_field = self.schema.get_field("trigrams").unwrap();
        let path_field = self.schema.get_field("path").unwrap();

        let mut subqueries: Vec<(tantivy::query::Occur, Box<dyn tantivy::query::Query>)> =
            Vec::new();
        for trigram in trigrams {
            // Tantivy's default text tokenizer lowercases terms during indexing,
            // but TermQuery bypasses the tokenizer. Lowercase to match.
            let lower = trigram.to_lowercase();
            let term = Term::from_field_text(trigrams_field, &lower);
            let query = TermQuery::new(term, IndexRecordOption::Basic);
            subqueries.push((tantivy::query::Occur::Must, Box::new(query)));
        }

        if subqueries.is_empty() {
            return Ok(Vec::new());
        }

        let query = BooleanQuery::new(subqueries);
        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit).order_by_score())
            .into_diagnostic()?;

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address).into_diagnostic()?;
            let path = retrieved_doc
                .get_first(path_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            results.push(path);
        }

        Ok(results)
    }

    pub fn all_paths(&self, limit: usize) -> Result<Vec<String>> {
        use tantivy::query::AllQuery;

        let searcher = self.reader.searcher();
        let path_field = self.schema.get_field("path").unwrap();

        let top_docs = searcher
            .search(&AllQuery, &TopDocs::with_limit(limit).order_by_score())
            .into_diagnostic()?;

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address).into_diagnostic()?;
            let path = retrieved_doc
                .get_first(path_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            results.push(path);
        }

        Ok(results)
    }

    pub fn clear(&self) -> Result<()> {
        let mut writer = self.get_writer(50_000_000)?;
        writer.delete_all_documents().into_diagnostic()?;
        writer.commit().into_diagnostic()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::trigram::extract_trigrams;
    use tantivy::TantivyDocument;
    use tempfile::TempDir;

    fn make_engine(dir: &TempDir) -> TantivySearchEngine {
        TantivySearchEngine::open_or_create(dir.path()).expect("open_or_create")
    }

    fn index_doc(engine: &TantivySearchEngine, path: &str, content: &str) {
        let schema = engine.schema();
        let path_field = schema.get_field("path").expect("path field");
        let content_field = schema.get_field("content").expect("content field");
        let line_count_field = schema.get_field("line_count").expect("line_count field");
        let trigrams_field = schema.get_field("trigrams").expect("trigrams field");

        let tgrams_str = extract_trigrams(content)
            .into_iter()
            .collect::<Vec<_>>()
            .join(" ");

        let mut writer = engine.get_writer(15_000_000).expect("writer");
        let mut doc = TantivyDocument::default();
        doc.add_text(path_field, path);
        doc.add_text(content_field, content);
        doc.add_u64(line_count_field, 1);
        doc.add_text(trigrams_field, &tgrams_str);
        writer.add_document(doc).expect("add_document");
        writer.commit().expect("commit");
        // Force the reader to see the freshly committed segment; without this,
        // OnCommitWithDelay may not expose the new docs within the test window.
        engine.reload_reader().expect("reload_reader");
    }

    /// RED → GREEN: before the code_trigram tokenizer was registered,
    /// search_trigrams returned zero results for any identifier containing '_'
    /// because SimpleTokenizer splits on '_'.  After the fix this must pass.
    #[test]
    fn trigram_search_finds_underscore_identifier() {
        let dir = TempDir::new().expect("tempdir");
        let engine = make_engine(&dir);

        index_doc(
            &engine,
            "src/search/regex_filter.rs",
            "fn execute_scan() {}",
        );

        let tgrams: Vec<String> = extract_trigrams("execute_scan").into_iter().collect();

        let results = engine
            .search_trigrams(&tgrams, 10)
            .expect("search_trigrams");

        assert!(
            !results.is_empty(),
            "expected >=1 result for 'execute_scan' trigrams, got 0. \
             Check that the code_trigram tokenizer is registered."
        );
    }

    /// Additional check: storage_cozo (another underscore identifier).
    #[test]
    fn trigram_search_finds_storage_cozo() {
        let dir = TempDir::new().expect("tempdir");
        let engine = make_engine(&dir);

        index_doc(&engine, "src/state/cozo.rs", "struct storage_cozo {}");

        let tgrams: Vec<String> = extract_trigrams("storage_cozo").into_iter().collect();

        let results = engine
            .search_trigrams(&tgrams, 10)
            .expect("search_trigrams");

        assert!(
            !results.is_empty(),
            "expected >=1 result for 'storage_cozo' trigrams, got 0."
        );
    }

    /// Regression guard: non-underscore, non-space identifiers must still work.
    /// (Space-containing trigrams like "fn " are not searched via search_trigrams
    /// in production — regex_filter falls back to all_paths when patterns produce
    /// only space-containing trigrams.  This test focuses on pure-alpha trigrams.)
    #[test]
    fn trigram_search_finds_non_underscore_pattern() {
        let dir = TempDir::new().expect("tempdir");
        let engine = make_engine(&dir);

        index_doc(&engine, "src/main.rs", "struct MainRunner {}");

        let tgrams: Vec<String> = extract_trigrams("MainRunner").into_iter().collect();

        let results = engine
            .search_trigrams(&tgrams, 10)
            .expect("search_trigrams");

        assert!(
            !results.is_empty(),
            "expected >=1 result for 'MainRunner' trigrams, got 0."
        );
    }
}
