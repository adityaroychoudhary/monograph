mod db;
mod tracker;
mod watcher;
mod commands;

use std::sync::Mutex;
use tauri::Manager;

pub struct Db(pub Mutex<rusqlite::Connection>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let path = dir.join("tracker.db");
            let conn = db::open(&path)?;
            println!("Database ready at {:?}", path);
            app.manage(Db(Mutex::new(conn)));
            tracker::start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
    commands::list_apps,
    commands::set_group,
    commands::set_tracked,
    commands::usage_buckets
])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
