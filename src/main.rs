//! NeoCross – a minimalist, click-through crosshair overlay, by T-GOW.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod assets;
mod autostart;
mod backdrop;
mod crosshair;
#[cfg(feature = "docs")]
mod gallery;
#[cfg(not(windows))]
mod hotkeys;
mod i18n;
mod instance;
#[cfg(windows)]
mod ipc;
mod overlay;
mod paths;
mod platform;
mod settings;
mod tray;
mod ui;

use instance::Instance;

const HELP: &str = "NeoCross – crosshair overlay by T-GOW

Usage: neocross [--autostart] [--reset]

  --autostart   start with the menu closed (used by \"Start with the system\")
  --reset       start with default settings
  --version     print the version
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |flag: &str| args.iter().any(|a| a == flag);
    let value = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned();
    if has("--version") || has("-V") {
        println!("NeoCross {}", paths::VERSION);
        return;
    }
    if has("--help") || has("-h") {
        print!("{HELP}");
        return;
    }

    #[cfg(windows)]
    if has("--menu") {
        menu_process(!has("--no-activate"), value("--pos"));
        return;
    }

    #[cfg(feature = "docs")]
    if let Some(dir) = value("--docs").map(std::path::PathBuf::from) {
        // README images: rendered in a sandbox with its own settings.
        unsafe { std::env::set_var("NEOCROSS_DATA_DIR", std::env::temp_dir().join("neocross-docs")) };
        gallery::write_all(&dir);
        let shared = app::Shared::new(settings::Settings::default(), false);
        let _ = ui::run(shared, ui::Mode::Docs(dir));
        return;
    }
    let _ = value;

    // Wayland lets no program place, raise or click-through its windows; XWayland does.
    #[cfg(target_os = "linux")]
    if std::env::var_os("NEOCROSS_WAYLAND").is_none() {
        unsafe { std::env::remove_var("WAYLAND_DISPLAY") };
    }

    let guard = match instance::acquire() {
        Instance::First(guard) => guard,
        Instance::AlreadyRunning => return,
    };

    let mut settings = settings::Settings::load();
    if has("--reset") {
        settings = settings::Settings::default();
    }
    i18n::set_language(settings.language);
    let open_menu = !autostart::launched_at_login() || settings.first_start;
    settings.first_start = false;

    let shared = app::Shared::new(settings, true);
    shared.save_now();
    guard.listen(shared.clone());

    #[cfg(windows)]
    overlay::run(shared.clone(), open_menu);
    #[cfg(not(windows))]
    let _ = ui::run(shared.clone(), ui::Mode::Standalone { visible_at_start: open_menu });

    shared.save_now();
}

/// Windows: the menu window, started by the running NeoCross while the menu is open.
#[cfg(windows)]
fn menu_process(activate: bool, pos: Option<String>) {
    let Some(settings) = ipc::first_settings() else { return };
    i18n::set_language(settings.language);
    let shared = app::Shared::new(settings, false);
    ipc::attach_client(&shared);
    let position = pos.and_then(|p| {
        let (x, y) = p.split_once(',')?;
        Some([x.parse().ok()?, y.parse().ok()?])
    });
    let _ = ui::run(shared, ui::Mode::Menu { activate, position });
}
