//! "Start with the system": the Run key on Windows, a LaunchAgent on macOS, an XDG autostart
//! entry on Linux. The state lives only there, so it is always what the system will really do.

use std::path::PathBuf;

#[cfg_attr(target_os = "macos", allow(dead_code))]
const NAME: &str = "NeoCross";
const ARG: &str = "--autostart";

fn exe() -> Option<PathBuf> {
    std::env::current_exe().ok()
}

#[cfg(windows)]
mod imp {
    use super::*;
    use crate::overlay::wide;
    use windows_sys::Win32::System::Registry::{
        HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
    };

    const RUN: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

    fn command() -> Option<String> {
        Some(format!("\"{}\" {ARG}", exe()?.display()))
    }

    fn read() -> Option<String> {
        let (key, name) = (wide(RUN), wide(NAME));
        let mut buf = vec![0u16; 1024];
        let mut size = (buf.len() * 2) as u32;
        let rc = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                buf.as_mut_ptr().cast(),
                &mut size,
            )
        };
        if rc != 0 {
            return None;
        }
        let len = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
        Some(String::from_utf16_lossy(&buf[..len]))
    }

    pub fn is_enabled() -> bool {
        read().is_some_and(|v| Some(v) == command())
    }

    pub fn set(on: bool) -> bool {
        let (key, name) = (wide(RUN), wide(NAME));
        let rc = unsafe {
            if on {
                let Some(cmd) = command() else { return false };
                let value = wide(&cmd);
                RegSetKeyValueW(
                    HKEY_CURRENT_USER,
                    key.as_ptr(),
                    name.as_ptr(),
                    REG_SZ,
                    value.as_ptr().cast(),
                    (value.len() * 2) as u32,
                )
            } else {
                RegDeleteKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr())
            }
        };
        rc == 0 || (!on && read().is_none())
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::*;

    fn plist() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join("Library/LaunchAgents/com.t-gow.neocross.plist"))
    }

    fn contents() -> Option<String> {
        let exe = exe()?;
        let exe = exe.to_string_lossy().replace('&', "&amp;").replace('<', "&lt;");
        Some(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>com.t-gow.neocross</string>
  <key>ProgramArguments</key><array><string>{exe}</string><string>{ARG}</string></array>
  <key>RunAtLoad</key><true/>
</dict>
</plist>
"#
        ))
    }

    pub fn is_enabled() -> bool {
        plist().is_some_and(|p| std::fs::read_to_string(p).ok() == contents())
    }

    pub fn set(on: bool) -> bool {
        let Some(path) = plist() else { return false };
        if on {
            let Some(text) = contents() else { return false };
            path.parent().is_some_and(|d| std::fs::create_dir_all(d).is_ok()) && std::fs::write(path, text).is_ok()
        } else {
            std::fs::remove_file(&path).is_ok() || !path.exists()
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod imp {
    use super::*;

    fn entry() -> Option<PathBuf> {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
        Some(base.join("autostart").join("neocross.desktop"))
    }

    fn contents() -> Option<String> {
        let exe = exe()?;
        Some(format!(
            "[Desktop Entry]\nType=Application\nName={NAME}\nComment=Crosshair overlay\nExec=\"{}\" {ARG}\nIcon=neocross\nTerminal=false\nX-GNOME-Autostart-enabled=true\n",
            exe.display()
        ))
    }

    pub fn is_enabled() -> bool {
        entry().is_some_and(|p| std::fs::read_to_string(p).ok() == contents())
    }

    pub fn set(on: bool) -> bool {
        let Some(path) = entry() else { return false };
        if on {
            let Some(text) = contents() else { return false };
            path.parent().is_some_and(|d| std::fs::create_dir_all(d).is_ok()) && std::fs::write(path, text).is_ok()
        } else {
            std::fs::remove_file(&path).is_ok() || !path.exists()
        }
    }
}

pub use imp::{is_enabled, set};

/// Whether this process was started by the autostart entry.
pub fn launched_at_login() -> bool {
    std::env::args().any(|a| a == ARG)
}
