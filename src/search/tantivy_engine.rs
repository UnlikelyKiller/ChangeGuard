use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy};

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    pub path: String,
    pub line_count: usize,
    pub score: f32,
}

pub struct TantivySearchEngine {
    index: Index,
    reader: IndexReader,
    schema: Schema,
}

impl TantivySearchEngine {
    pub fn open_or_create(path: &Path) -> Result<Self> {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("path", TEXT | STORED);
        schema_builder.add_text_field("content", TEXT);
        schema_builder.add_u64_field("line_count", STORED);
        schema_builder.add_text_field("language", TEXT | STORED);
        schema_builder.add_text_field("trigrams", TEXT); // For regex pre-filtering
        let schema = schema_builder.build();

        if !path.exists() {
            std::fs::create_dir_all(path).into_diagnostic()?;
        }

        let index = Index::open_or_create(
            tantivy::directory::MmapDirectory::open(path).into_diagnostic()?,
            schema.clone(),
        )
        .into_diagnostic()?;

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

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit))
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

            results.push(SearchResult {
                path,
                line_count,
                score,
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
            let term = Term::from_field_text(trigrams_field, trigram);
            let query = TermQuery::new(term, IndexRecordOption::Basic);
            subqueries.push((tantivy::query::Occur::Must, Box::new(query)));
        }

        if subqueries.is_empty() {
            return Ok(Vec::new());
        }

        let query = BooleanQuery::new(subqueries);
        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit))
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
            .search(&AllQuery, &TopDocs::with_limit(limit))
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
