use rusqlite_migration::{Migrations, M};

pub fn get_migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(
            "CREATE TABLE snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                head_hash TEXT,
                branch_name TEXT,
                is_clean INTEGER NOT NULL,
                packet_json TEXT NOT NULL
            );"
        ),
    ])
}
