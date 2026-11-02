use crate::crosshair::{COLORS, SIZES, Shape, Style};
use crate::i18n::Language;
use crate::paths;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuKey {
    F1,
    #[default]
    F2,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Pause,
}

impl MenuKey {
    pub const ALL: [MenuKey; 17] = [
        MenuKey::F1,
        MenuKey::F2,
        MenuKey::F4,
        MenuKey::F5,
        MenuKey::F6,
        MenuKey::F7,
        MenuKey::F8,
        MenuKey::F9,
        MenuKey::F10,
        MenuKey::F11,
        MenuKey::F12,
        MenuKey::Insert,
        MenuKey::Home,
        MenuKey::End,
        MenuKey::PageUp,
        MenuKey::PageDown,
        MenuKey::Pause,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MenuKey::F1 => "F1",
            MenuKey::F2 => "F2",
            MenuKey::F4 => "F4",
            MenuKey::F5 => "F5",
            MenuKey::F6 => "F6",
            MenuKey::F7 => "F7",
            MenuKey::F8 => "F8",
            MenuKey::F9 => "F9",
            MenuKey::F10 => "F10",
            MenuKey::F11 => "F11",
            MenuKey::F12 => "F12",
            MenuKey::Insert => crate::t!("Einfg", "Insert"),
            MenuKey::Home => crate::t!("Pos1", "Home"),
            MenuKey::End => crate::t!("Ende", "End"),
            MenuKey::PageUp => crate::t!("Bild ↑", "Page Up"),
            MenuKey::PageDown => crate::t!("Bild ↓", "Page Down"),
            MenuKey::Pause => "Pause",
        }
    }

    /// Windows virtual-key code.
    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn vk(self) -> u16 {
        match self {
            MenuKey::F1 => 0x70,
            MenuKey::F2 => 0x71,
            MenuKey::F4 => 0x73,
            MenuKey::F5 => 0x74,
            MenuKey::F6 => 0x75,
            MenuKey::F7 => 0x76,
            MenuKey::F8 => 0x77,
            MenuKey::F9 => 0x78,
            MenuKey::F10 => 0x79,
            MenuKey::F11 => 0x7A,
            MenuKey::F12 => 0x7B,
            MenuKey::Insert => 0x2D,
            MenuKey::Home => 0x24,
            MenuKey::End => 0x23,
            MenuKey::PageUp => 0x21,
            MenuKey::PageDown => 0x22,
            MenuKey::Pause => 0x13,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub style: Style,
    /// Crosshair shown or hidden.
    pub visible: bool,
    /// Monitor id (`\\.\DISPLAY2` on Windows, the monitor name elsewhere); `None` = primary.
    pub monitor: Option<String>,
    /// Fine adjustment of the aim point in physical pixels.
    pub offset: [i32; 2],
    pub menu_key: MenuKey,
    pub language: Language,
    pub first_start: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            style: Style::default(),
            visible: true,
            monitor: None,
            offset: [0, 0],
            menu_key: MenuKey::F2,
            language: Language::Auto,
            first_start: true,
        }
    }
}

/// The settings file of NeoCross 1.x: three indices into its fixed lists.
#[derive(Deserialize)]
struct LegacySettings {
    shape: usize,
    color: usize,
    size: usize,
}

impl Settings {
    pub fn load() -> Settings {
        let mut s = match std::fs::read_to_string(paths::settings_file()) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => Self::from_legacy().unwrap_or_default(),
        };
        s.style.sanitize();
        s.offset = s.offset.map(|v| v.clamp(-2000, 2000));
        s
    }

    fn from_legacy() -> Option<Settings> {
        let text = std::fs::read_to_string(paths::legacy_settings_file()?).ok()?;
        Self::from_legacy_json(&text)
    }

    fn from_legacy_json(text: &str) -> Option<Settings> {
        let old: LegacySettings = serde_json::from_str(text).ok()?;
        let mut s = Settings::default();
        s.style.shape = *Shape::CLASSIC.get(old.shape)?;
        s.style.color = COLORS.get(old.color)?.0;
        s.style.size = SIZES.get(old.size)?.0;
        s.first_start = false;
        Some(s)
    }

    pub fn save(&self) -> std::io::Result<()> {
        // The debounced saver and an explicit save at exit may meet here.
        static WRITING: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = WRITING.lock().unwrap_or_else(|e| e.into_inner());
        let path = paths::settings_file();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self).map_err(std::io::Error::other)?)?;
        std::fs::rename(tmp, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_v1_settings() {
        // First-run defaults of 1.0.1, then the last entry of every list.
        let s = Settings::from_legacy_json(r#"{"shape": 5, "color": 3, "size": 1}"#).unwrap();
        assert_eq!((s.style.shape, s.style.color, s.style.size), (Shape::XShape, [0xFF, 0xD7, 0x00], 8.0));
        let s = Settings::from_legacy_json(r#"{"shape": 6, "color": 7, "size": 5}"#).unwrap();
        assert_eq!((s.style.shape, s.style.color, s.style.size), (Shape::Triangle, [0x00, 0xFF, 0xFF], 30.0));
        assert!(Settings::from_legacy_json(r#"{"shape": 9, "color": 0, "size": 0}"#).is_none());
    }

    #[test]
    fn round_trips_and_tolerates_unknown_fields() {
        let s = Settings::default();
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(serde_json::from_str::<Settings>(&json).unwrap(), s);
        let partial: Settings = serde_json::from_str(r#"{"style": {"shape": "halo"}, "future": 1}"#).unwrap();
        assert_eq!(partial.style.shape, Shape::Halo);
        assert_eq!(partial.style.size, Style::default().size);
    }
}
