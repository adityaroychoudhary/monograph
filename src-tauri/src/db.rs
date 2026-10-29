use rusqlite::{Connection, Result};
use std::path::Path;

pub fn open(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;

         CREATE TABLE IF NOT EXISTS apps (
             name TEXT PRIMARY KEY,
             grp  TEXT NOT NULL DEFAULT 'unassigned'
         );

         CREATE TABLE IF NOT EXISTS sessions (
             id         INTEGER PRIMARY KEY AUTOINCREMENT,
             app        TEXT NOT NULL REFERENCES apps(name),
             started_at INTEGER NOT NULL,  -- unix ms
             ended_at   INTEGER NOT NULL   -- unix ms
         );

         CREATE INDEX IF NOT EXISTS idx_sessions_started
             ON sessions(started_at);",
    )?;
    Ok(conn)
}
