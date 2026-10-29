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
    // migrate older databases; fails harmlessly if the column already exists
if conn
    .execute("ALTER TABLE apps ADD COLUMN tracked INTEGER NOT NULL DEFAULT 0", [])
    .is_ok()
{
    // keep tracking the apps you've already assigned to a group
    conn.execute("UPDATE apps SET tracked = 1 WHERE grp != 'unassigned'", [])?;
}
    Ok(conn)
}
