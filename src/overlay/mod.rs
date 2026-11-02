#[cfg(windows)]
mod win32;
#[cfg(windows)]
pub use win32::{monitors, run, wide};

#[cfg(not(windows))]
mod viewport;
#[cfg(not(windows))]
pub use viewport::{show, viewport_id};
