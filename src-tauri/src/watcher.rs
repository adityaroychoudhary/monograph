fn generic() -> Option<String> {
    active_win_pos_rs::get_active_window()
        .ok()
        .map(|w| w.app_name)
}

#[cfg(target_os = "linux")]
fn niri() -> Option<String> {
    let out = std::process::Command::new("niri")
        .args(["msg", "--json", "focused-window"])
        .output()
        .ok()?;
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    v.get("app_id")?.as_str().map(String::from)
}

#[cfg(target_os = "linux")]
pub fn active_app() -> Option<String> {
    if std::env::var_os("NIRI_SOCKET").is_some() {
        niri()
    } else {
        generic()
    }
}

#[cfg(not(target_os = "linux"))]
pub fn active_app() -> Option<String> {
    generic()
}