//! Windows: the menu is a separate process that only exists while it is open.
//!
//! The always-running part of NeoCross (crosshair, keyboard, tray) needs no GPU and only a few
//! megabytes. The menu uses OpenGL, whose driver keeps its memory until the process ends, so it
//! gets its own process and gives everything back when you close it. Core and menu talk in JSON
//! lines over the menu's stdin and stdout: anonymous pipes, nothing anyone else can reach.

use crate::app::{Action, Shared};
use crate::settings::Settings;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::process::{ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Msg {
    Settings(Settings),
    /// Core → menu: come to the front.
    Focus,
    /// Core → menu: close.
    Close,
    /// Menu → core: a text field has keyboard focus, letter hotkeys pause.
    Typing(bool),
    /// Menu → core: where the window was when it closed (points), to reopen it there.
    Position([f32; 2]),
}

fn send(w: &mut impl Write, msg: &Msg) -> bool {
    match serde_json::to_string(msg) {
        Ok(line) => writeln!(w, "{line}").and_then(|_| w.flush()).is_ok(),
        Err(_) => false,
    }
}

/// All messages until the pipe closes; lines that aren't ours are skipped.
fn messages(r: impl BufRead) -> impl Iterator<Item = Msg> {
    r.lines().map_while(Result::ok).filter_map(|l| serde_json::from_str(&l).ok())
}

// ---------------------------------------------------------------------------------------------
// Core side

pub struct Host {
    shared: Arc<Shared>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    open: Arc<AtomicBool>,
    position: Arc<Mutex<Option<[f32; 2]>>>,
}

impl Host {
    pub fn new(shared: Arc<Shared>) -> Host {
        let stdin: Arc<Mutex<Option<ChildStdin>>> = Arc::default();
        let pipe = stdin.clone();
        // Hotkey and tray changes go to the open menu, so it always shows the truth.
        shared.set_forward(move |s| {
            if let Ok(mut w) = pipe.lock()
                && let Some(w) = w.as_mut()
            {
                send(w, &Msg::Settings(s.clone()));
            }
        });
        Host { shared, stdin, open: Arc::default(), position: Arc::default() }
    }

    pub fn is_open(&self) -> bool {
        self.open.load(Ordering::Acquire)
    }

    fn send(&self, msg: Msg) {
        if let Ok(mut w) = self.stdin.lock()
            && let Some(w) = w.as_mut()
        {
            send(w, &msg);
        }
    }

    pub fn toggle(&self, activate: bool) {
        if self.is_open() { self.close() } else { self.open(activate) }
    }

    pub fn close(&self) {
        self.send(Msg::Close);
    }

    pub fn open(&self, activate: bool) {
        if self.is_open() {
            self.send(Msg::Focus);
            return;
        }
        let Ok(exe) = std::env::current_exe() else { return };
        let mut cmd = Command::new(exe);
        cmd.arg("--menu");
        if !activate {
            cmd.arg("--no-activate");
        }
        if let Some([x, y]) = self.position.lock().ok().and_then(|p| *p) {
            cmd.arg("--pos").arg(format!("{x},{y}"));
        }
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
        let Ok(mut child) = cmd.spawn() else { return };
        let (Some(mut stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
            let _ = child.kill();
            return;
        };
        send(&mut stdin, &Msg::Settings(self.shared.settings()));
        if let Ok(mut slot) = self.stdin.lock() {
            *slot = Some(stdin);
        }
        self.open.store(true, Ordering::Release);
        self.shared.menu_open.store(true, Ordering::Relaxed);

        let (shared, pipe, open, position) =
            (self.shared.clone(), self.stdin.clone(), self.open.clone(), self.position.clone());
        std::thread::Builder::new()
            .name("menu-link".into())
            .spawn(move || {
                for msg in messages(BufReader::new(stdout)) {
                    match msg {
                        Msg::Settings(s) => shared.apply_remote(s),
                        Msg::Typing(on) => shared.typing.store(on, Ordering::Relaxed),
                        Msg::Position(p) => {
                            if let Ok(mut slot) = position.lock() {
                                *slot = Some(p);
                            }
                        }
                        Msg::Focus | Msg::Close => {}
                    }
                }
                // The menu closed (or crashed): the crosshair simply keeps running.
                let _ = child.wait();
                if let Ok(mut slot) = pipe.lock() {
                    *slot = None;
                }
                open.store(false, Ordering::Release);
                shared.menu_open.store(false, Ordering::Relaxed);
                shared.typing.store(false, Ordering::Relaxed);
                // Autostart may have been switched in the menu: let the tray look again.
                shared.mark_dirty();
            })
            .expect("menu link thread");
    }
}

// ---------------------------------------------------------------------------------------------
// Menu side

static CLIENT: AtomicBool = AtomicBool::new(false);

/// Reads the settings the core sends first. `None` if this process wasn't started by the core.
pub fn first_settings() -> Option<Settings> {
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line).ok()?;
    match serde_json::from_str(&line).ok()? {
        Msg::Settings(s) => Some(s),
        _ => None,
    }
}

/// Connects the menu's state to the core.
pub fn attach_client(shared: &Arc<Shared>) {
    CLIENT.store(true, Ordering::Relaxed);
    shared.set_forward(|s| to_core(Msg::Settings(s.clone())));
    let shared = shared.clone();
    std::thread::Builder::new()
        .name("core-link".into())
        .spawn(move || {
            for msg in messages(std::io::stdin().lock()) {
                match msg {
                    Msg::Settings(s) => shared.apply_remote(s),
                    Msg::Focus => shared.push(Action::ShowMenu),
                    Msg::Close => shared.push(Action::Quit),
                    Msg::Typing(_) | Msg::Position(_) => {}
                }
            }
            // The core is gone, so is the reason for this menu.
            shared.clear_forward();
            shared.push(Action::Quit);
        })
        .expect("core link thread");
}

pub fn to_core(msg: Msg) {
    if CLIENT.load(Ordering::Relaxed) {
        send(&mut std::io::stdout().lock(), &msg);
    }
}
