// Import SQLite connection and query macro
use rusqlite::{Connection, params};

/// Creates (or opens) the SQLite database and ensures the clipboard table exists
pub fn init() -> Connection {
    // Open a file named cliptrack.db in the current directory
    let conn = Connection::open("cliptrack.db").expect("Failed to open DB");

    // Create a table for clipboard entries if it doesn't exist yet
    conn.execute(
        "CREATE TABLE IF NOT EXISTS clipboard (
            id INTEGER PRIMARY KEY AUTOINCREMENT,         -- unique ID for each entry
            content TEXT NOT NULL,                        -- the copied text
            copied_at DATETIME DEFAULT CURRENT_TIMESTAMP  -- when it was copied
        )",
        [],
    ).expect("Failed to create table");

    conn
}

/// Inserts a new clipboard entry into the database
pub fn save_clip(conn: &Connection, content: &str) {
    conn.execute(
        "INSERT INTO clipboard (content) VALUES (?)",
        params![content], // Use ? to safely insert the variable
    ).expect("Failed to insert clipboard entry");
}
