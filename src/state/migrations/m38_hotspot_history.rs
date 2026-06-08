use rusqlite_migration::M;

pub fn m38_hotspot_history() -> Vec<M<'static>> {
    vec![M::up(
        "CREATE TABLE hotspot_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            snapshot_id INTEGER REFERENCES snapshots(id),
            file_path TEXT NOT NULL,
            score REAL NOT NULL,
            display_score REAL NOT NULL,
            complexity INTEGER NOT NULL,
            frequency REAL NOT NULL,
            centrality REAL,
            timestamp TEXT NOT NULL
        );

        CREATE TABLE temporal_coupling_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            snapshot_id INTEGER REFERENCES snapshots(id),
            file_a TEXT NOT NULL,
            file_b TEXT NOT NULL,
            score REAL NOT NULL,
            timestamp TEXT NOT NULL
        );",
    )]
}
