use std::path::PathBuf;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REPO_URL: &str = "https://github.com/T-G0W/NeoCross";

fn home() -> PathBuf {
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// `%APPDATA%\T-GOW\NeoCross`, `~/Library/Application Support/T-GOW/NeoCross` or
/// `$XDG_CONFIG_HOME/neocross`. `NEOCROSS_DATA_DIR` overrides it for a portable setup.
pub fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("NEOCROSS_DATA_DIR") {
        return PathBuf::from(dir);
    }
    if cfg!(windows) {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home().join("AppData").join("Roaming"))
            .join("T-GOW")
            .join("NeoCross")
    } else if cfg!(target_os = "macos") {
        home().join("Library").join("Application Support").join("T-GOW").join("NeoCross")
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .unwrap_or_else(|| home().join(".config"))
            .join("neocross")
    }
}

pub fn settings_file() -> PathBuf {
    data_dir().join("settings.json")
}

/// Where NeoCross 1.x kept its settings (`%APPDATA%\NeoCross\settings.json`).
pub fn legacy_settings_file() -> Option<PathBuf> {
    if !cfg!(windows) {
        return None;
    }
    std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join("NeoCross").join("settings.json"))
}
