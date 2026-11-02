//! The crosshair on Windows, and the always-running core around it.
//!
//! The crosshair is a layered, click-through, topmost tool window, drawn on the CPU and handed to
//! the compositor with `UpdateLayeredWindow` (per-pixel alpha). It needs no GPU context and costs
//! nothing while it doesn't change. The same thread reads the keyboard through Raw Input (no hook,
//! no injected code, every key still reaches the game), owns the tray icon and starts the menu
//! process on demand.

use crate::app::{Action, Key, Monitor, Shared};
use crate::ipc::Host;
use crate::tray::Tray;
use crate::{crosshair, i18n};
use std::cell::RefCell;
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::sync::Arc;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION, CreateCompatibleDC,
    CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, EnumDisplayMonitors, GetDC, GetMonitorInfoW, HDC,
    HMONITOR, MONITORINFO, MONITORINFOEXW, ReleaseDC, SelectObject,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows_sys::Win32::UI::Input::{
    GetRawInputData, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER, RID_INPUT, RIDEV_INPUTSINK, RIM_TYPEKEYBOARD,
    RegisterRawInputDevices,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, HTTRANSPARENT, HWND_TOPMOST,
    MA_NOACTIVATE, MONITORINFOF_PRIMARY, MSG, PostMessageW, PostQuitMessage, RI_KEY_BREAK, RegisterClassExW, SW_HIDE,
    SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetTimer, SetWindowPos, ShowWindow, TranslateMessage,
    ULW_ALPHA, UpdateLayeredWindow, WM_APP, WM_DESTROY, WM_DISPLAYCHANGE, WM_DPICHANGED, WM_INPUT, WM_MOUSEACTIVATE,
    WM_NCHITTEST, WM_TIMER, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_EX_TRANSPARENT, WS_POPUP,
};

const WM_REFRESH: u32 = WM_APP + 1;
const TIMER_WATCH: usize = 1;

const VK_SHIFT: usize = 0x10;
const VK_F3: u16 = 0x72;

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

struct State {
    shared: Arc<Shared>,
    hwnd: HWND,
    /// Position and settings revision currently on screen.
    shown: Option<(i32, i32, u64)>,
    image: Option<(u64, tiny_skia::Pixmap)>,
    keys: [bool; 256],
    menu: Host,
    tray: Option<Tray>,
    /// Settings revision and language the tray menu shows.
    tray_seen: (u64, bool),
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

/// Runs NeoCross until it quits. `open_menu`: start with the menu open.
pub fn run(shared: Arc<Shared>, open_menu: bool) {
    unsafe {
        let hinstance = GetModuleHandleW(null());
        let class = wide("NeoCrossOverlay");
        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: hinstance,
            lpszClassName: class.as_ptr(),
            ..zeroed()
        };
        RegisterClassExW(&wc);
        let title = wide("NeoCross Crosshair");
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class.as_ptr(),
            title.as_ptr(),
            WS_POPUP,
            0,
            0,
            1,
            1,
            null_mut(),
            null_mut(),
            hinstance,
            null(),
        );
        if hwnd.is_null() {
            return;
        }

        // Keyboard as Raw Input, delivered even while a game has focus. Nothing is blocked.
        let device = RAWINPUTDEVICE { usUsagePage: 0x01, usUsage: 0x06, dwFlags: RIDEV_INPUTSINK, hwndTarget: hwnd };
        RegisterRawInputDevices(&device, 1, size_of::<RAWINPUTDEVICE>() as u32);
        // Picks up monitor changes and keeps the window above games that grab topmost themselves.
        SetTimer(hwnd, TIMER_WATCH, 2000, None);

        let target = hwnd as isize;
        shared.set_notify(move || {
            PostMessageW(target as HWND, WM_REFRESH, 0, 0);
        });
        let menu = Host::new(shared.clone());
        let tray = Tray::new(shared.clone());
        STATE.with(|s| {
            *s.borrow_mut() = Some(State {
                shared: shared.clone(),
                hwnd,
                shown: None,
                image: None,
                keys: [false; 256],
                menu,
                tray,
                tray_seen: (shared.rev(), i18n::is_en()),
            })
        });
        shared.set_monitors(monitors());
        apply();
        if open_menu {
            with_state(|st| st.menu.open(true));
        }

        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        STATE.with(|s| s.borrow_mut().take());
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_INPUT => {
                on_raw_input(lparam);
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_REFRESH => {
                refresh();
                0
            }
            WM_TIMER => {
                with_state(|st| st.shared.set_monitors(monitors()));
                apply();
                keep_on_top(hwnd);
                0
            }
            WM_DISPLAYCHANGE | WM_DPICHANGED => {
                with_state(|st| st.shared.set_monitors(monitors()));
                apply();
                0
            }
            WM_MOUSEACTIVATE => MA_NOACTIVATE as LRESULT,
            WM_NCHITTEST => HTTRANSPARENT as LRESULT,
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

/// Settings changed or an action is waiting.
fn refresh() {
    apply();
    let actions = with_state(|st| st.shared.take_actions()).unwrap_or_default();
    for action in actions {
        match action {
            Action::ToggleMenu { activate } => {
                with_state(|st| st.menu.toggle(activate));
            }
            Action::ShowMenu => {
                with_state(|st| st.menu.open(true));
            }
            Action::Quit => {
                let hwnd = with_state(|st| {
                    st.menu.close();
                    st.shared.save_now();
                    st.hwnd
                });
                if let Some(hwnd) = hwnd {
                    unsafe { DestroyWindow(hwnd) };
                }
                return;
            }
        }
    }
    with_state(|st| {
        let rev = st.shared.rev();
        if st.tray_seen.0 == rev {
            return;
        }
        let s = st.shared.settings();
        i18n::set_language(s.language);
        if st.tray_seen.1 != i18n::is_en() {
            st.tray = None;
            st.tray = Tray::new(st.shared.clone());
        }
        st.tray_seen = (rev, i18n::is_en());
        if let Some(tray) = &st.tray {
            tray.sync(&s);
        }
    });
}

fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> Option<R> {
    STATE.with(|s| s.borrow_mut().as_mut().map(f))
}

unsafe fn keep_on_top(hwnd: HWND) {
    let visible = with_state(|st| st.shown.is_some()).unwrap_or(false);
    if visible {
        unsafe { SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE) };
    }
}

unsafe fn on_raw_input(lparam: LPARAM) {
    unsafe {
        let mut raw: RAWINPUT = zeroed();
        let mut size = size_of::<RAWINPUT>() as u32;
        let read = GetRawInputData(
            lparam as HRAWINPUT,
            RID_INPUT,
            &mut raw as *mut RAWINPUT as *mut c_void,
            &mut size,
            size_of::<RAWINPUTHEADER>() as u32,
        );
        if read == u32::MAX || raw.header.dwType != RIM_TYPEKEYBOARD {
            return;
        }
        let kb = raw.data.keyboard;
        let vk = kb.VKey as usize;
        if vk == 0 || vk >= 255 {
            return;
        }
        let down = kb.Flags as u32 & RI_KEY_BREAK == 0;
        let pressed = with_state(|st| {
            let was = std::mem::replace(&mut st.keys[vk], down);
            (down && !was).then(|| (st.shared.clone(), st.keys[VK_SHIFT]))
        })
        .flatten();
        if let Some((shared, shift)) = pressed {
            let menu_vk = shared.settings().menu_key.vk();
            let key = match kb.VKey {
                v if v == menu_vk => Key::Menu,
                VK_F3 => Key::Quit,
                0x5A => Key::Color,      // Z
                0x43 => Key::Size,       // C
                0x58 => Key::Shape,      // X
                0x56 => Key::Visibility, // V
                _ => return,
            };
            shared.on_key(key, shift);
        }
    }
}

/// Renders the current style and puts it on the chosen monitor. Does nothing if neither the
/// style nor the position changed.
fn apply() {
    with_state(|st| unsafe {
        let s = st.shared.settings();
        if !s.visible {
            if st.shown.take().is_some() {
                ShowWindow(st.hwnd, SW_HIDE);
            }
            return;
        }
        let list = st.shared.monitors.lock().map(|m| m.clone()).unwrap_or_default();
        let Some(mon) = Monitor::pick(&list, s.monitor.as_deref()).cloned() else {
            return;
        };
        let rev = st.shared.rev();
        if st.image.as_ref().is_none_or(|(r, _)| *r != rev) {
            st.image = Some((rev, crosshair::render(&s.style)));
        }
        let Some((_, pm)) = st.image.as_ref() else {
            return;
        };
        let half = (pm.width() / 2) as i32;
        let x = mon.x + mon.width / 2 - half + s.offset[0];
        let y = mon.y + mon.height / 2 - half + s.offset[1];
        if st.shown == Some((x, y, rev)) {
            return;
        }
        present(st.hwnd, pm, x, y);
        if st.shown.is_none() {
            ShowWindow(st.hwnd, SW_SHOWNOACTIVATE);
            SetWindowPos(st.hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        }
        st.shown = Some((x, y, rev));
    });
}

/// Copies the premultiplied RGBA image into a DIB and hands it to the compositor.
unsafe fn present(hwnd: HWND, pm: &tiny_skia::Pixmap, x: i32, y: i32) {
    unsafe {
        let (w, h) = (pm.width() as i32, pm.height() as i32);
        let screen: HDC = GetDC(null_mut());
        let mem = CreateCompatibleDC(screen);
        let mut info: BITMAPINFO = zeroed();
        info.bmiHeader = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            ..zeroed()
        };
        let mut bits: *mut c_void = null_mut();
        let bitmap = CreateDIBSection(screen, &info, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
        if !bitmap.is_null() && !bits.is_null() {
            let dst = std::slice::from_raw_parts_mut(bits as *mut u8, (w * h * 4) as usize);
            for (d, s) in dst.chunks_exact_mut(4).zip(pm.data().chunks_exact(4)) {
                d.copy_from_slice(&[s[2], s[1], s[0], s[3]]);
            }
            let old = SelectObject(mem, bitmap);
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            let pos = POINT { x, y };
            let size = SIZE { cx: w, cy: h };
            let src = POINT { x: 0, y: 0 };
            UpdateLayeredWindow(hwnd, screen, &pos, &size, mem, &src, 0, &blend, ULW_ALPHA);
            SelectObject(mem, old);
            DeleteObject(bitmap);
        }
        DeleteDC(mem);
        ReleaseDC(null_mut(), screen);
    }
}

/// All monitors in physical pixels, primary first, then left to right.
pub fn monitors() -> Vec<Monitor> {
    unsafe extern "system" fn collect(hmon: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> i32 {
        unsafe {
            let list = &mut *(data as *mut Vec<Monitor>);
            let mut info: MONITORINFOEXW = zeroed();
            info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
            if GetMonitorInfoW(hmon, &mut info as *mut MONITORINFOEXW as *mut MONITORINFO) == 0 {
                return 1;
            }
            let r = info.monitorInfo.rcMonitor;
            let len = info.szDevice.iter().position(|c| *c == 0).unwrap_or(info.szDevice.len());
            let id = String::from_utf16_lossy(&info.szDevice[..len]);
            let (mut dx, mut dy) = (96u32, 96u32);
            GetDpiForMonitor(hmon, MDT_EFFECTIVE_DPI, &mut dx, &mut dy);
            let number: String = id.chars().filter(|c| c.is_ascii_digit()).collect();
            list.push(Monitor {
                label: format!(
                    "{} {}",
                    crate::t!("Bildschirm", "Display"),
                    if number.is_empty() { "?" } else { &number }
                ),
                id,
                x: r.left,
                y: r.top,
                width: r.right - r.left,
                height: r.bottom - r.top,
                scale: dx as f32 / 96.0,
                primary: info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0,
            });
            1
        }
    }
    let mut list: Vec<Monitor> = Vec::new();
    unsafe {
        EnumDisplayMonitors(null_mut(), null(), Some(collect), &mut list as *mut Vec<Monitor> as LPARAM);
    }
    list.sort_by_key(|m| (!m.primary, m.x, m.y));
    list
}
