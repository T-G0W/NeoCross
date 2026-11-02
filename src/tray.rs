//! Tray icon (notification area / menu bar) with quick access to the most used settings.

use crate::app::{Action, Shared};
use crate::crosshair::{COLORS, SIZES, Shape};
use crate::settings::Settings;
use crate::{assets, autostart, paths, platform, t};
use std::collections::HashMap;
use std::sync::Arc;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

#[derive(Clone, Copy)]
enum Cmd {
    OpenMenu,
    ToggleVisible,
    Shape(Shape),
    Color(usize),
    Size(usize),
    Autostart,
    GitHub,
    Quit,
}

pub struct Tray {
    _icon: TrayIcon,
    visible: CheckMenuItem,
    shapes: Vec<(Shape, CheckMenuItem)>,
    colors: Vec<CheckMenuItem>,
    sizes: Vec<CheckMenuItem>,
    autostart: CheckMenuItem,
}

fn autostart_label() -> &'static str {
    if cfg!(windows) {
        t!("Mit Windows starten", "Start with Windows")
    } else {
        t!("Beim Anmelden starten", "Start at login")
    }
}

impl Tray {
    pub fn new(shared: Arc<Shared>) -> Option<Tray> {
        let s = shared.settings();
        let key = s.menu_key.label();
        let mut ids: HashMap<MenuId, Cmd> = HashMap::new();
        let mut item = |text: String, cmd: Cmd| {
            let i = MenuItem::new(text, true, None);
            ids.insert(i.id().clone(), cmd);
            i
        };
        let open = item(format!("{} ({key})", t!("Menü öffnen", "Open menu")), Cmd::OpenMenu);
        let github = item(t!("NeoCross auf GitHub", "NeoCross on GitHub").into(), Cmd::GitHub);
        let quit = item(format!("{} (F3)", t!("Beenden", "Quit")), Cmd::Quit);

        let mut check = |text: String, cmd: Cmd| {
            let i = CheckMenuItem::new(text, true, false, None);
            ids.insert(i.id().clone(), cmd);
            i
        };
        let visible = check(format!("{} (V)", t!("Fadenkreuz anzeigen", "Show crosshair")), Cmd::ToggleVisible);
        let autostart = check(autostart_label().into(), Cmd::Autostart);
        let shapes: Vec<(Shape, CheckMenuItem)> =
            Shape::ALL.iter().map(|sh| (*sh, check(sh.name().into(), Cmd::Shape(*sh)))).collect();
        let colors: Vec<CheckMenuItem> =
            COLORS.iter().enumerate().map(|(i, c)| check(t!(c.1, c.2).into(), Cmd::Color(i))).collect();
        let sizes: Vec<CheckMenuItem> =
            SIZES.iter().enumerate().map(|(i, sz)| check(format!("{} · {} px", sz.1, sz.0), Cmd::Size(i))).collect();

        let shape_menu = Submenu::new(format!("{} (X)", t!("Form", "Shape")), true);
        for (_, i) in &shapes {
            let _ = shape_menu.append(i);
        }
        let color_menu = Submenu::new(format!("{} (Z)", t!("Farbe", "Color")), true);
        for i in &colors {
            let _ = color_menu.append(i);
        }
        let size_menu = Submenu::new(format!("{} (C)", t!("Größe", "Size")), true);
        for i in &sizes {
            let _ = size_menu.append(i);
        }

        let menu = Menu::new();
        let _ = menu.append_items(&[
            &open,
            &visible,
            &PredefinedMenuItem::separator(),
            &shape_menu,
            &color_menu,
            &size_menu,
            &PredefinedMenuItem::separator(),
            &autostart,
            &github,
            &PredefinedMenuItem::separator(),
            &quit,
        ]);

        let (rgba, w, h) = assets::rgba(assets::LOGO_64);
        let icon = TrayIconBuilder::new()
            .with_icon(Icon::from_rgba(rgba, w, h).ok()?)
            .with_tooltip(format!("NeoCross {} – {key}: {}", paths::VERSION, t!("Menü", "menu")))
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(!cfg!(windows))
            .build()
            .ok()?;

        let ids = Arc::new(ids);
        let sh = shared.clone();
        MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
            if let Some(cmd) = ids.get(&e.id).copied() {
                run(&sh, cmd);
            }
        }));
        let sh = shared.clone();
        TrayIconEvent::set_event_handler(Some(move |e: TrayIconEvent| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = e {
                sh.push(Action::ToggleMenu { activate: true });
            }
        }));

        let tray = Tray { _icon: icon, visible, shapes, colors, sizes, autostart };
        tray.sync(&s);
        Some(tray)
    }

    /// Mirrors the settings into the check marks.
    pub fn sync(&self, s: &Settings) {
        self.visible.set_checked(s.visible);
        for (shape, item) in &self.shapes {
            item.set_checked(*shape == s.style.shape);
        }
        for (i, item) in self.colors.iter().enumerate() {
            item.set_checked(COLORS[i].0 == s.style.color);
        }
        for (i, item) in self.sizes.iter().enumerate() {
            item.set_checked((SIZES[i].0 - s.style.size).abs() < 0.5);
        }
        self.autostart.set_checked(autostart::is_enabled());
    }
}

fn run(shared: &Shared, cmd: Cmd) {
    match cmd {
        Cmd::OpenMenu => shared.push(Action::ShowMenu),
        Cmd::ToggleVisible => shared.update(|s| s.visible = !s.visible),
        Cmd::Shape(sh) => shared.update(|s| s.style.shape = sh),
        Cmd::Color(i) => shared.update(|s| s.style.color = COLORS[i].0),
        Cmd::Size(i) => shared.update(|s| s.style.size = SIZES[i].0),
        Cmd::Autostart => {
            autostart::set(!autostart::is_enabled());
            shared.mark_dirty();
        }
        Cmd::GitHub => platform::open_url(paths::REPO_URL),
        Cmd::Quit => shared.push(Action::Quit),
    }
}

/// On Linux the tray lives in its own GTK thread and polls the settings from there.
#[cfg(target_os = "linux")]
pub fn spawn_linux(shared: Arc<Shared>) {
    std::thread::Builder::new()
        .name("tray".into())
        .spawn(move || {
            if gtk::init().is_err() {
                return;
            }
            let Some(tray) = Tray::new(shared.clone()) else {
                return;
            };
            let mut seen = 0;
            gtk::glib::timeout_add_local(std::time::Duration::from_millis(400), move || {
                let rev = shared.rev();
                if rev != seen {
                    seen = rev;
                    tray.sync(&shared.settings());
                }
                gtk::glib::ControlFlow::Continue
            });
            gtk::main();
        })
        .expect("tray thread");
}
