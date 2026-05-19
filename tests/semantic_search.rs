//! Integration tests for VectorStore::query covering all reachable query tiers.
//!
//! - Tier 1: HNSW approximate nearest-neighbour index (L2 over normalized vectors)
//! - Tier 2: Cozo-native cos_dist fallback
//! - Tier 3: Rust-side cosine_sim last-resort safety net (not reachable with cozo-redux;
//!   preserved as a guard against a future fork that drops `cos_dist`.)
//!
//! See `conductor/track56-1/`.

use changeguard::index::symbols::SymbolKind;
use changeguard::semantic::chunker::AstChunk;
use changeguard::semantic::vector_store::VectorStore;
use changeguard::state::storage_cozo::CozoStorage;
use tempfile::TempDir;

fn sled_cozo(tmp: &TempDir) -> CozoStorage {
    let path = tmp.path().join("test.cozo");
    CozoStorage::new(&path).unwrap()
}

fn make_chunk(file_path: &str, name: &str, offset: usize, content: &str) -> AstChunk {
    AstChunk {
        file_path: file_path.to_string(),
        name: name.to_string(),
        offset,
        content: content.to_string(),
        kind: SymbolKind::Function,
        docstring: None,
        range: (0, content.len()),
        lines: (offset, offset + 1),
    }
}

fn consistency_chunks() -> Vec<AstChunk> {
    (0..5)
        .map(|i| make_chunk(&format!("f{}.rs", i), &format!("fn_{}", i), i, "fn"))
        .collect()
}

fn consistency_embeddings() -> Vec<Vec<f32>> {
    vec![
        vec![1.0, 2.0, 3.0],
        vec![2.0, 0.5, 0.0],
        vec![0.0, 0.0, 5.0],
        vec![3.0, 3.0, 3.0],
        vec![0.1, 10.0, 0.1],
    ]
}

/// Tier 1: HNSW happy path using cosine-equivalent normalized L2 distance.
#[test]
fn hnsw_query_returns_ordered_results() {
    let tmp = tempfile::tempdir().unwrap();
    let storage = sled_cozo(&tmp);
    let store = VectorStore::new(&storage, 3, false).unwrap();

    let chunks = vec![
        make_chunk("a.rs", "fn_a", 0, "fn a"),
        make_chunk("b.rs", "fn_b", 1, "fn b"),
        make_chunk("c.rs", "fn_c", 2, "fn c"),
    ];
    // Non-unit embeddings where chunk[0] is closest to [1.0, 0.0, 0.0].
    let embeddings = vec![
        vec![2.0, 0.0, 0.0],
        vec![0.0, 3.0, 0.0],
        vec![0.0, 0.0, 4.0],
    ];

    store.index_chunks(chunks, embeddings).unwrap();

    let results = store.query(vec![0.9, 0.1, 0.0], 2).unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, "a.rs");
}

/// Tier 2: cos_dist fallback, with no HNSW index present.
#[test]
fn cos_dist_fallback_when_hnsw_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let storage = sled_cozo(&tmp);
    let store = VectorStore::new_without_hnsw(&storage, 3).unwrap();

    let chunks = vec![
        make_chunk("x.rs", "fn_x", 0, "fn x"),
        make_chunk("y.rs", "fn_y", 1, "fn y"),
    ];
    let embeddings = vec![vec![2.0, 0.0, 0.0], vec![0.0, 3.0, 0.0]];

    store.index_chunks(chunks, embeddings).unwrap();

    let results = store.query(vec![0.9, 0.1, 0.0], 2).unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, "x.rs");
}

/// HNSW normalized L2 and cos_dist fallback return the same ordering for the same data.
#[test]
fn hnsw_and_cos_dist_produce_same_ordering() {
    let hnsw_tmp = tempfile::tempdir().unwrap();
    let hnsw_storage = sled_cozo(&hnsw_tmp);
    let store_hnsw = VectorStore::new(&hnsw_storage, 3, false).unwrap();
    store_hnsw
        .index_chunks(consistency_chunks(), consistency_embeddings())
        .unwrap();
    let hnsw_results = store_hnsw.query(vec![1.0, 1.0, 1.0], 5).unwrap();

    let cos_tmp = tempfile::tempdir().unwrap();
    let cos_storage = sled_cozo(&cos_tmp);
    let store_no_hnsw = VectorStore::new_without_hnsw(&cos_storage, 3).unwrap();
    store_no_hnsw
        .index_chunks(consistency_chunks(), consistency_embeddings())
        .unwrap();
    let cos_dist_results = store_no_hnsw.query(vec![1.0, 1.0, 1.0], 5).unwrap();

    assert_eq!(hnsw_results.len(), cos_dist_results.len());
    let hnsw_names: Vec<&str> = hnsw_results.iter().map(|(_, n, _, _)| n.as_str()).collect();
    let cos_names: Vec<&str> = cos_dist_results
        .iter()
        .map(|(_, n, _, _)| n.as_str())
        .collect();
    assert_eq!(&hnsw_names[..2], &cos_names[..2]);
}

/// A dimension mismatch between query and stored vectors must produce an error.
#[test]
fn dimension_mismatch_produces_error() {
    let tmp = tempfile::tempdir().unwrap();
    let storage = sled_cozo(&tmp);
    let store = VectorStore::new_without_hnsw(&storage, 3).unwrap();

    let chunks = vec![make_chunk("p.rs", "fn_p", 0, "fn p")];
    let embeddings = vec![vec![1.0, 0.0, 0.0]];

    store.index_chunks(chunks, embeddings).unwrap();

    let result = store.query(vec![0.9, 0.1, 0.0, 0.0], 2);
    assert!(result.is_err(), "Dimension mismatch must return an error");
}
