use rusqlite::{params, Connection, Result};
use chrono::Local;

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("cliptrack.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS clipboard (
            id INTEGER PRIMARY KEY,
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

pub fn insert_clip(conn: &Connection, content: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = conn.execute(
        "INSERT INTO clipboard (content, timestamp) VALUES (?1, ?2)",
        params![content, timestamp],
    );
}

pub fn get_all_clips(conn: &Connection) -> Vec<(String, String)> {
    let mut stmt = conn
        .prepare("SELECT content, timestamp FROM clipboard ORDER BY id DESC LIMIT 50")
        .unwrap();
    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap();

    rows.map(|r| r.unwrap()).collect()
}

pub fn clear_history(conn: &Connection) {
    let _ = conn.execute("DELETE FROM clipboard", []);
}
