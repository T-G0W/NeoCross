//! Global hotkeys on macOS and Linux (X11). Windows reads the keyboard through Raw Input in the
//! overlay thread instead, which never takes a key away from the game.
//!
//! Here the system grabs a registered key, so only the menu key is registered all the time;
//! Z, C, X, V and F3 are registered only while the menu is open.

use crate::app::{Key, Shared};
use crate::settings::MenuKey;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn code(key: MenuKey) -> Code {
    match key {
        MenuKey::F1 => Code::F1,
        MenuKey::F2 => Code::F2,
        MenuKey::F4 => Code::F4,
        MenuKey::F5 => Code::F5,
        MenuKey::F6 => Code::F6,
        MenuKey::F7 => Code::F7,
        MenuKey::F8 => Code::F8,
        MenuKey::F9 => Code::F9,
        MenuKey::F10 => Code::F10,
        MenuKey::F11 => Code::F11,
        MenuKey::F12 => Code::F12,
        MenuKey::Insert => Code::Insert,
        MenuKey::Home => Code::Home,
        MenuKey::End => Code::End,
        MenuKey::PageUp => Code::PageUp,
        MenuKey::PageDown => Code::PageDown,
        MenuKey::Pause => Code::Pause,
    }
}

const MENU_KEYS: [(Code, Key); 5] = [
    (Code::KeyZ, Key::Color),
    (Code::KeyC, Key::Size),
    (Code::KeyX, Key::Shape),
    (Code::KeyV, Key::Visibility),
    (Code::F3, Key::Quit),
];

pub struct Hotkeys {
    manager: Option<GlobalHotKeyManager>,
    map: Arc<Mutex<HashMap<u32, (Key, bool)>>>,
    menu: Option<(MenuKey, HotKey)>,
    menu_keys: Vec<HotKey>,
}

impl Hotkeys {
    pub fn new(shared: Arc<Shared>) -> Hotkeys {
        let map: Arc<Mutex<HashMap<u32, (Key, bool)>>> = Arc::default();
        let lookup = map.clone();
        GlobalHotKeyEvent::set_event_handler(Some(move |e: GlobalHotKeyEvent| {
            if e.state != HotKeyState::Pressed {
                return;
            }
            let hit = lookup.lock().ok().and_then(|m| m.get(&e.id).copied());
            if let Some((key, shift)) = hit {
                shared.on_key(key, shift);
            }
        }));
        Hotkeys { manager: GlobalHotKeyManager::new().ok(), map, menu: None, menu_keys: Vec::new() }
    }

    fn register(&self, hk: HotKey, key: Key, shift: bool) -> bool {
        let Some(m) = &self.manager else { return false };
        if m.register(hk).is_err() {
            return false;
        }
        if let Ok(mut map) = self.map.lock() {
            map.insert(hk.id(), (key, shift));
        }
        true
    }

    fn unregister(&self, hk: HotKey) {
        if let Some(m) = &self.manager {
            let _ = m.unregister(hk);
        }
        if let Ok(mut map) = self.map.lock() {
            map.remove(&hk.id());
        }
    }

    /// Registers exactly the keys the current state needs.
    pub fn sync(&mut self, menu_key: MenuKey, menu_open: bool) {
        if self.menu.map(|(k, _)| k) != Some(menu_key) {
            if let Some((_, hk)) = self.menu.take() {
                self.unregister(hk);
            }
            let hk = HotKey::new(None, code(menu_key));
            if self.register(hk, Key::Menu, false) {
                self.menu = Some((menu_key, hk));
            }
        }
        if menu_open && self.menu_keys.is_empty() {
            for (c, key) in MENU_KEYS {
                for (mods, shift) in [(None, false), (Some(Modifiers::SHIFT), true)] {
                    let hk = HotKey::new(mods, c);
                    if self.register(hk, key, shift) {
                        self.menu_keys.push(hk);
                    }
                }
            }
        } else if !menu_open && !self.menu_keys.is_empty() {
            for hk in std::mem::take(&mut self.menu_keys) {
                self.unregister(hk);
            }
        }
    }
}
