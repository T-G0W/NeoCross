//! State shared by the menu, the overlay, the tray icon and the hotkeys.

use crate::settings::Settings;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Duration;

/// Things that need the thread owning the windows. Settings changes are applied on the spot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Show or hide the menu. From a hotkey the game keeps focus (`activate: false`).
    ToggleMenu {
        activate: bool,
    },
    ShowMenu,
    Quit,
}

/// Keys NeoCross listens to. The menu key works everywhere, the others only while the menu is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Menu,
    Quit,
    Color,
    Size,
    Shape,
    Visibility,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Monitor {
    pub id: String,
    pub label: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub scale: f32,
    pub primary: bool,
}

impl Monitor {
    pub fn pick<'a>(list: &'a [Monitor], id: Option<&str>) -> Option<&'a Monitor> {
        id.and_then(|id| list.iter().find(|m| m.id == id))
            .or_else(|| list.iter().find(|m| m.primary))
            .or_else(|| list.first())
    }
}

type Hook<T> = Mutex<Option<Box<T>>>;

pub struct Shared {
    settings: Mutex<Settings>,
    rev: AtomicU64,
    actions: Mutex<Vec<Action>>,
    ctx: OnceLock<eframe::egui::Context>,
    save_tx: Mutex<Option<Sender<()>>>,
    /// Wakes the thread that owns the overlay (Windows core).
    notify: Hook<dyn Fn() + Send + Sync>,
    /// Hands local changes to the other process (Windows: core ⇄ menu).
    forward: Hook<dyn Fn(&Settings) + Send + Sync>,
    pub monitors: Mutex<Vec<Monitor>>,
    pub menu_open: AtomicBool,
    /// A text field in the menu has keyboard focus: letter hotkeys are ignored.
    pub typing: AtomicBool,
}

fn lock<T: ?Sized>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl Shared {
    /// `persist`: this process owns the settings file.
    pub fn new(settings: Settings, persist: bool) -> Arc<Shared> {
        let shared = Arc::new(Shared {
            settings: Mutex::new(settings),
            rev: AtomicU64::new(1),
            actions: Mutex::new(Vec::new()),
            ctx: OnceLock::new(),
            save_tx: Mutex::new(None),
            notify: Mutex::new(None),
            forward: Mutex::new(None),
            monitors: Mutex::new(Vec::new()),
            menu_open: AtomicBool::new(false),
            typing: AtomicBool::new(false),
        });
        if persist {
            shared.start_saver();
        }
        shared
    }

    /// Writes the settings once things are quiet for a moment, e.g. after a slider was dragged.
    fn start_saver(self: &Arc<Self>) {
        let (tx, rx) = mpsc::channel::<()>();
        *lock(&self.save_tx) = Some(tx);
        let weak = Arc::downgrade(self);
        std::thread::Builder::new()
            .name("settings".into())
            .spawn(move || {
                while rx.recv().is_ok() {
                    while rx.recv_timeout(Duration::from_millis(400)).is_ok() {}
                    let Some(shared) = weak.upgrade() else { return };
                    let _ = shared.settings().save();
                }
            })
            .expect("settings thread");
    }

    pub fn attach_ui(&self, ctx: &eframe::egui::Context) {
        let _ = self.ctx.set(ctx.clone());
    }

    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn set_notify(&self, f: impl Fn() + Send + Sync + 'static) {
        *lock(&self.notify) = Some(Box::new(f));
    }

    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn set_forward(&self, f: impl Fn(&Settings) + Send + Sync + 'static) {
        *lock(&self.forward) = Some(Box::new(f));
    }

    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn clear_forward(&self) {
        *lock(&self.forward) = None;
    }

    pub fn settings(&self) -> Settings {
        lock(&self.settings).clone()
    }

    /// Changes the settings; the overlay, menu and settings file follow automatically.
    pub fn update(&self, f: impl FnOnce(&mut Settings)) {
        self.apply(f, true);
    }

    #[cfg_attr(not(windows), allow(dead_code))]
    /// Takes over settings that came from the other process, without sending them back.
    pub fn apply_remote(&self, s: Settings) {
        self.apply(|x| *x = s, false);
    }

    fn apply(&self, f: impl FnOnce(&mut Settings), forward: bool) {
        let changed = {
            let mut s = lock(&self.settings);
            let before = s.clone();
            f(&mut s);
            s.style.sanitize();
            (*s != before).then(|| s.clone())
        };
        let Some(now) = changed else { return };
        self.rev.fetch_add(1, Ordering::Relaxed);
        self.wake();
        if let Some(tx) = lock(&self.save_tx).as_ref() {
            let _ = tx.send(());
        }
        if forward && let Some(f) = lock(&self.forward).as_ref() {
            f(&now);
        }
    }

    /// Increases with every change, so views can tell when to redraw.
    pub fn rev(&self) -> u64 {
        self.rev.load(Ordering::Relaxed)
    }

    /// Something outside the settings changed (e.g. autostart): views should look again.
    pub fn mark_dirty(&self) {
        self.rev.fetch_add(1, Ordering::Relaxed);
        self.wake();
    }

    pub fn save_now(&self) {
        let _ = self.settings().save();
    }

    fn wake(&self) {
        if let Some(f) = lock(&self.notify).as_ref() {
            f();
        }
        if let Some(ctx) = self.ctx.get() {
            ctx.request_repaint();
            #[cfg(not(windows))]
            ctx.request_repaint_of(crate::overlay::viewport_id());
        }
    }

    pub fn push(&self, action: Action) {
        lock(&self.actions).push(action);
        self.wake();
    }

    pub fn take_actions(&self) -> Vec<Action> {
        std::mem::take(&mut *lock(&self.actions))
    }

    pub fn set_monitors(&self, list: Vec<Monitor>) {
        let mut m = lock(&self.monitors);
        if *m != list {
            *m = list;
            drop(m);
            if let Some(ctx) = self.ctx.get() {
                ctx.request_repaint();
            }
        }
    }

    /// A key went down somewhere on the system. The key itself still reaches the game.
    pub fn on_key(&self, key: Key, shift: bool) {
        let step = if shift { -1 } else { 1 };
        match key {
            Key::Menu => self.push(Action::ToggleMenu { activate: false }),
            _ if !self.menu_open.load(Ordering::Relaxed) || self.typing.load(Ordering::Relaxed) => {}
            Key::Quit => self.push(Action::Quit),
            Key::Color => self.update(|s| s.style.cycle_color(step)),
            Key::Size => self.update(|s| s.style.cycle_size(step)),
            Key::Shape => self.update(|s| s.style.shape = s.style.shape.cycle(step)),
            Key::Visibility => self.update(|s| s.visible = !s.visible),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letter_keys_only_work_while_the_menu_is_open() {
        let shared = Shared::new(Settings::default(), false);
        let shape = shared.settings().style.shape;
        shared.on_key(Key::Shape, false);
        assert_eq!(shared.settings().style.shape, shape, "menu closed: X must do nothing");
        shared.menu_open.store(true, Ordering::Relaxed);
        shared.on_key(Key::Shape, false);
        assert_eq!(shared.settings().style.shape, shape.cycle(1));
        shared.on_key(Key::Shape, true);
        assert_eq!(shared.settings().style.shape, shape);
        shared.typing.store(true, Ordering::Relaxed);
        shared.on_key(Key::Shape, false);
        assert_eq!(shared.settings().style.shape, shape, "typing: X must do nothing");
        shared.on_key(Key::Menu, false);
        assert_eq!(shared.take_actions(), vec![Action::ToggleMenu { activate: false }]);
    }

    #[test]
    fn remote_changes_are_not_echoed() {
        let shared = Shared::new(Settings::default(), false);
        let sent = Arc::new(AtomicU64::new(0));
        let counter = sent.clone();
        shared.set_forward(move |_| {
            counter.fetch_add(1, Ordering::Relaxed);
        });
        let mut s = shared.settings();
        s.visible = false;
        shared.apply_remote(s);
        assert_eq!(sent.load(Ordering::Relaxed), 0);
        shared.update(|s| s.visible = true);
        assert_eq!(sent.load(Ordering::Relaxed), 1);
        shared.update(|s| s.visible = true);
        assert_eq!(sent.load(Ordering::Relaxed), 1, "no change, nothing to send");
    }
}
