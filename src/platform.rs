//! Small platform helpers.

/// Opens a web page in the default browser.
pub fn open_url(url: &str) {
    #[cfg(windows)]
    unsafe {
        use crate::overlay::wide;
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
        let (verb, target) = (wide("open"), wide(url));
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            target.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        );
    }
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}

#[cfg(windows)]
pub use win::*;

#[cfg(windows)]
mod win {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Dwm::{
        DWMWA_CAPTION_COLOR, DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        HWND_TOPMOST, SW_HIDE, SW_SHOW, SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetForegroundWindow,
        SetWindowPos, ShowWindow,
    };
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    /// Native handle of the menu window.
    pub fn hwnd(window: &impl HasWindowHandle) -> Option<HWND> {
        match window.window_handle().ok()?.as_raw() {
            RawWindowHandle::Win32(h) => Some(h.hwnd.get() as HWND),
            _ => None,
        }
    }

    fn set_attr(h: HWND, attr: i32, value: u32) {
        unsafe {
            DwmSetWindowAttribute(h, attr as u32, (&value as *const u32).cast(), 4);
        }
    }

    /// Shows or hides the menu. From a hotkey (`activate == false`) the game keeps keyboard
    /// focus, so Z, C and X keep working while you watch the change.
    pub fn show_or_hide(window: &impl HasWindowHandle, show: bool, activate: bool) {
        let Some(h) = hwnd(window) else { return };
        unsafe {
            if !show {
                ShowWindow(h, SW_HIDE);
            } else if activate {
                ShowWindow(h, SW_SHOW);
                SetForegroundWindow(h);
            } else {
                ShowWindow(h, SW_SHOWNOACTIVATE);
                SetWindowPos(h, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
            }
        }
    }

    /// Dark title bar to match the menu.
    pub fn dark_title_bar(window: &impl HasWindowHandle) {
        let Some(h) = hwnd(window) else { return };
        set_attr(h, DWMWA_USE_IMMERSIVE_DARK_MODE, 1);
        set_attr(h, DWMWA_CAPTION_COLOR, 0x00_12_0E_0B); // COLORREF 0x00BBGGRR of #0B0E12
    }
}
