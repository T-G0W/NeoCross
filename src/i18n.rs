use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

static EN: AtomicBool = AtomicBool::new(true);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    #[default]
    Auto,
    De,
    En,
}

impl Language {
    pub const ALL: [Language; 3] = [Language::Auto, Language::De, Language::En];

    pub fn label(self) -> String {
        match self {
            Language::Auto => format!("{} ({})", crate::t!("Automatisch", "Automatic"), system_language().label()),
            Language::De => "Deutsch".into(),
            Language::En => "English".into(),
        }
    }

    pub fn resolve(self) -> Language {
        match self {
            Language::Auto => system_language(),
            other => other,
        }
    }
}

pub fn system_language() -> Language {
    static SYS: OnceLock<Language> = OnceLock::new();
    *SYS.get_or_init(|| if system_is_german() { Language::De } else { Language::En })
}

#[cfg(windows)]
fn system_is_german() -> bool {
    let langid = unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() };
    langid & 0x3FF == 0x07
}

#[cfg(not(windows))]
fn system_is_german() -> bool {
    sys_locale::get_locale().is_some_and(|l| l.to_ascii_lowercase().starts_with("de"))
}

pub fn set_language(lang: Language) {
    EN.store(lang.resolve() == Language::En, Ordering::Relaxed);
}

pub fn is_en() -> bool {
    EN.load(Ordering::Relaxed)
}

/// `t!("Deutsch", "English")` picks the text for the active language.
#[macro_export]
macro_rules! t {
    ($de:expr, $en:expr) => {
        if $crate::i18n::is_en() { $en } else { $de }
    };
}
