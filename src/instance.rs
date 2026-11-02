//! One NeoCross at a time. Starting it again opens the menu of the running instance.

use crate::app::{Action, Shared};
use std::sync::Arc;

pub enum Instance {
    First(Guard),
    AlreadyRunning,
}

#[cfg(windows)]
pub struct Guard {
    _mutex: isize,
    event: isize,
}

#[cfg(windows)]
const MUTEX: &str = "Local\\NeoCross.T-GOW.Instance";
#[cfg(windows)]
const EVENT: &str = "Local\\NeoCross.T-GOW.ShowMenu";

#[cfg(windows)]
pub fn acquire() -> Instance {
    use crate::overlay::wide;
    use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};
    use windows_sys::Win32::System::Threading::{CreateEventW, CreateMutexW, EVENT_MODIFY_STATE, OpenEventW, SetEvent};
    unsafe {
        let mutex = CreateMutexW(std::ptr::null(), 0, wide(MUTEX).as_ptr());
        if mutex.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
            let event = OpenEventW(EVENT_MODIFY_STATE, 0, wide(EVENT).as_ptr());
            if !event.is_null() {
                SetEvent(event);
            }
            return Instance::AlreadyRunning;
        }
        let event = CreateEventW(std::ptr::null(), 0, 0, wide(EVENT).as_ptr());
        Instance::First(Guard { _mutex: mutex as isize, event: event as isize })
    }
}

#[cfg(windows)]
impl Guard {
    /// Waits for later starts in the background and opens the menu for them.
    pub fn listen(&self, shared: Arc<Shared>) {
        use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
        use windows_sys::Win32::System::Threading::{INFINITE, WaitForSingleObject};
        let event = self.event;
        if event == 0 {
            return;
        }
        std::thread::Builder::new()
            .name("instance".into())
            .spawn(move || unsafe {
                while WaitForSingleObject(event as _, INFINITE) == WAIT_OBJECT_0 {
                    shared.push(Action::ShowMenu);
                }
            })
            .expect("instance thread");
    }
}

#[cfg(unix)]
pub struct Guard {
    listener: std::os::unix::net::UnixListener,
    path: std::path::PathBuf,
}

#[cfg(unix)]
fn socket_path() -> std::path::PathBuf {
    let dir = std::env::var_os("XDG_RUNTIME_DIR").map(std::path::PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let user = std::env::var("USER").unwrap_or_default();
    dir.join(format!("neocross-{user}.sock"))
}

#[cfg(unix)]
pub fn acquire() -> Instance {
    use std::io::Write;
    use std::os::unix::net::{UnixListener, UnixStream};
    let path = socket_path();
    if let Ok(mut stream) = UnixStream::connect(&path) {
        let _ = stream.write_all(b"show\n");
        return Instance::AlreadyRunning;
    }
    // Nobody answers: a stale socket from a crash.
    let _ = std::fs::remove_file(&path);
    match UnixListener::bind(&path) {
        Ok(listener) => Instance::First(Guard { listener, path }),
        Err(_) => Instance::AlreadyRunning,
    }
}

#[cfg(unix)]
impl Guard {
    pub fn listen(&self, shared: Arc<Shared>) {
        let Ok(listener) = self.listener.try_clone() else {
            return;
        };
        std::thread::Builder::new()
            .name("instance".into())
            .spawn(move || {
                for stream in listener.incoming() {
                    if stream.is_ok() {
                        shared.push(Action::ShowMenu);
                    }
                }
            })
            .expect("instance thread");
    }
}

#[cfg(unix)]
impl Drop for Guard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
