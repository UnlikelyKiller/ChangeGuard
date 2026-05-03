use rusqlite::Connection;
fn main() {
    let conn = Connection::open(".changeguard/state/ledger.db").unwrap();
    let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0)).unwrap();
    println!("Current DB user_version: {}", version);
}
