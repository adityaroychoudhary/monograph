mod commands;
mod db;
mod tracker;
mod watcher;

use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WindowEvent,
};
use tauri_plugin_autostart::MacosLauncher;

pub struct Db(pub Mutex<rusqlite::Connection>);

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // must be the first plugin
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_opener::init())
        // closing the window hides it; the tracker keeps running
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            // database + tracker
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let path = dir.join("tracker.db");
            let conn = db::open(&path)?;
            println!("Database ready at {:?}", path);
            app.manage(Db(Mutex::new(conn)));
            tracker::start(app.handle().clone());

            // tray icon
            let open = MenuItem::with_id(app, "open", "Open Monograph", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_main(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;

            // start hidden when launched by autostart
            if !std::env::args().any(|a| a == "--hidden") {
                show_main(app.handle());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_apps,
            commands::set_group,
            commands::set_tracked,
            commands::usage_buckets,
            commands::get_autostart,
            commands::set_autostart
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}