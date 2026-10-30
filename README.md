# Monograph

A local-first screen time tracker. Monograph records which apps you use and for how long, lets you sort them into **personal** and **work** groups, and charts the result over days, weeks, months and years.

All data stays on your machine in a single SQLite file. Nothing is sent anywhere.

## Features

- Tracks the focused app in the background, as a session with a start and end time
- Choose exactly which apps are tracked with a checkbox; everything else is ignored
- Assign tracked apps to **personal** or **work**. Groups are applied when the chart is drawn, so reassigning an app also updates its history
- Step chart with two lines (work solid, personal dashed) and Day / Week / Month / Year views with back/forward navigation
- Runs in the background: closing the window hides it, and an optional "Start at login" setting launches it hidden
- Single instance: launching Monograph again just brings up the running one
- Records only the app identifier, never window titles or page contents

## Platform support

| Platform | Status |
|---|---|
| Linux, niri (Wayland) | Tested, uses `niri msg` to read the focused window |
| Linux, X11 | Should work through the `active-win-pos-rs` crate, untested |
| Windows | Should work through `active-win-pos-rs`, untested |
| macOS | Should work through `active-win-pos-rs`, untested; may need extra permissions |
| Other Wayland compositors (Hyprland, Sway, GNOME, KDE) | Not supported yet |

Each OS has its own small "watcher" in [`src-tauri/src/watcher.rs`](src-tauri/src/watcher.rs), so adding a compositor means adding one function there.

## Tech stack

- [Tauri v2](https://tauri.app) (Rust backend, system webview)
- React + TypeScript + Vite
- SQLite via [rusqlite](https://github.com/rusqlite/rusqlite)
- [Recharts](https://recharts.org) for the chart

## Building from source

### Prerequisites

Arch Linux:

```bash
sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file openssl \
  appmenu-gtk-module libappindicator-gtk3 librsvg xdotool rustup nodejs npm
rustup default stable
```

Other platforms: follow the [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/).

### Run in development

```bash
git clone https://github.com/adityaroychoudhary/monograph.git
cd monograph
npm install
npm run tauri dev
```

### Build and install a release binary

```bash
npm run tauri build -- --no-bundle
install -Dm755 src-tauri/target/release/monograph ~/.local/bin/monograph
```

Launch it, then tick **Start Monograph at login** inside the app. Enable autostart from the installed binary rather than a dev build, because the login entry stores the path of whichever binary you enabled it from.

### Tray and window access

Closing the window keeps Monograph running in the background. If your desktop has no system tray (for example niri without a bar), run `monograph` again to bring the window back, and bind it to a key. In niri:

```kdl
binds {
    Mod+Shift+M { spawn "/home/<you>/.local/bin/monograph"; }
}
```

To quit completely, use the tray menu or `pkill monograph`.

## Your data

The database is a normal SQLite file:

| OS | Location |
|---|---|
| Linux | `~/.local/share/com.adi.monograph/tracker.db` |
| macOS | `~/Library/Application Support/com.adi.monograph/tracker.db` |
| Windows | `%APPDATA%\com.adi.monograph\tracker.db` |

Schema:

```sql
apps(name TEXT PRIMARY KEY, grp TEXT, tracked INTEGER)
sessions(id INTEGER PRIMARY KEY, app TEXT, started_at INTEGER, ended_at INTEGER)  -- unix ms
```

Because raw sessions are stored, you can analyse them with anything that reads SQLite:

```bash
# export to CSV
sqlite3 -header -csv ~/.local/share/com.adi.monograph/tracker.db \
  "SELECT s.started_at, s.ended_at, s.app, a.grp
   FROM sessions s JOIN apps a ON a.name = s.app" > sessions.csv

# safe backup while the app is running
sqlite3 ~/.local/share/com.adi.monograph/tracker.db ".backup monograph-backup.db"
```

## Known limitations

- Browser profiles cannot be told apart, so a browser is tracked as one app
- There is no idle detection yet, so time while you are away from the keyboard still counts
- Only niri is supported on Wayland for now

## Roadmap

- [ ] UI redesign
- [ ] Idle detection
- [ ] Watchers for Hyprland, Sway, GNOME and KDE
- [ ] In-app export and quit button
- [ ] Per-app breakdown charts
- [ ] Testing and fixes on Windows and macOS
- [ ] AUR package

## License

MIT. See [LICENSE](LICENSE).
