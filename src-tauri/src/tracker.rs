use crate::{watcher, Db};
use std::{
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const POLL: Duration = Duration::from_secs(2);
const MAX_GAP_MS: i64 = 10_000; // longer gap = suspend/restart, don't extend the session

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}

pub fn start(app: AppHandle) {
    thread::spawn(move || {
        // (session id, app name, last time we saw it)
        let mut current: Option<(i64, String, i64)> = None;

        loop {
            thread::sleep(POLL);
            let now = now_ms();

            if matches!(&current, Some((_, _, last)) if now - last > MAX_GAP_MS) {
                current = None;
            }

            let Some(name) = watcher::active_app() else {
                current = None;
                continue;
            };

            let state = app.state::<Db>();
            let conn = state.0.lock().unwrap();

            let same = matches!(&current, Some((_, cur, _)) if *cur == name);
            if same {
                let (id, _, last) = current.as_mut().unwrap();
                conn.execute("UPDATE sessions SET ended_at = ?1 WHERE id = ?2", (now, *id)).ok();
                *last = now;
            } else {
                conn.execute("INSERT OR IGNORE INTO apps(name) VALUES (?1)", [&name]).ok();
                conn.execute(
                    "INSERT INTO sessions(app, started_at, ended_at) VALUES (?1, ?2, ?2)",
                    (&name, now),
                ).ok();
                current = Some((conn.last_insert_rowid(), name, now));
            }
        }
    });
}