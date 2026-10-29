use crate::Db;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct AppRow {
    name: String,
    grp: String,
    tracked: bool,
    total_secs: i64,
}

#[tauri::command]
pub fn list_apps(db: State<Db>) -> Result<Vec<AppRow>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT a.name, a.grp, a.tracked, COALESCE(SUM(s.ended_at - s.started_at), 0) / 1000
             FROM apps a LEFT JOIN sessions s ON s.app = a.name
             GROUP BY a.name
             ORDER BY 4 DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(AppRow { name: r.get(0)?, grp: r.get(1)?, tracked: r.get(2)?, total_secs: r.get(3)? })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub fn set_group(db: State<Db>, name: String, grp: String) -> Result<(), String> {
    if !["personal", "work", "unassigned"].contains(&grp.as_str()) {
        return Err("invalid group".into());
    }
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE apps SET grp = ?1 WHERE name = ?2", (&grp, &name))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_tracked(db: State<Db>, name: String, tracked: bool) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE apps SET tracked = ?1 WHERE name = ?2", (tracked, &name))
        .map_err(|e| e.to_string())?;
    Ok(())
}