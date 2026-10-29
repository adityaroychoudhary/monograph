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

#[derive(Serialize)]
pub struct Bucket {
    t: i64,        
    work: i64,     
    personal: i64, 
}

#[tauri::command]
pub fn usage_buckets(
    db: State<Db>,
    from_ms: i64,
    to_ms: i64,
    bucket_min: i64,
    tz_offset_min: i64,
) -> Result<Vec<Bucket>, String> {
    let step = bucket_min * 60_000;
    let off = tz_offset_min * 60_000;
    let start = from_ms - (from_ms + off).rem_euclid(step);
    let n = ((to_ms - start) / step + 1) as usize;
    let mut buckets: Vec<Bucket> = (0..n)
        .map(|i| Bucket { t: start + i as i64 * step, work: 0, personal: 0 })
        .collect();

    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT s.started_at, s.ended_at, a.grp
             FROM sessions s JOIN apps a ON a.name = s.app
             WHERE s.ended_at > ?1 AND s.started_at < ?2
               AND a.grp IN ('work', 'personal')",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map((from_ms, to_ms), |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, String>(2)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (s, e, grp) in rows {
        let first = ((s.max(start) - start) / step) as usize;
        let last = (((e.min(to_ms)) - start) / step) as usize;
        for i in first..=last.min(n - 1) {
            let b_start = start + i as i64 * step;
            let overlap = e.min(b_start + step) - s.max(b_start);
            if overlap > 0 {
                if grp == "work" {
                    buckets[i].work += overlap;
                } else {
                    buckets[i].personal += overlap;
                }
            }
        }
    }
    Ok(buckets)
}