//! The crosshair on macOS and Linux: a transparent, click-through, always-on-top egui viewport.

use crate::app::{Monitor, Shared};
use crate::crosshair;
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub fn viewport_id() -> egui::ViewportId {
    egui::ViewportId::from_hash_of("neocross-crosshair")
}

struct Texture {
    rev: u64,
    side: u32,
    handle: egui::TextureHandle,
}

static TEXTURE: Mutex<Option<Texture>> = Mutex::new(None);
static MONITORS_CHECKED: Mutex<Option<Instant>> = Mutex::new(None);

fn texture(ctx: &egui::Context, shared: &Shared) -> (egui::TextureId, u32) {
    let mut slot = TEXTURE.lock().unwrap_or_else(|e| e.into_inner());
    let rev = shared.rev();
    if slot.as_ref().is_none_or(|t| t.rev != rev) {
        let pm = crosshair::render(&shared.settings().style);
        let image = egui::ColorImage::from_rgba_premultiplied([pm.width() as usize, pm.height() as usize], pm.data());
        let handle = ctx.load_texture("crosshair", image, egui::TextureOptions::NEAREST);
        *slot = Some(Texture { rev, side: pm.width(), handle });
    }
    let t = slot.as_ref().expect("texture");
    (t.handle.id(), t.side)
}

fn monitors(window: &winit::window::Window) -> Vec<Monitor> {
    let primary = window.primary_monitor();
    let mut list: Vec<Monitor> = window
        .available_monitors()
        .enumerate()
        .map(|(i, m)| {
            let pos = m.position();
            let size = m.size();
            let name = m.name().unwrap_or_else(|| format!("{}", i + 1));
            Monitor {
                label: format!("{} {} – {name}", crate::t!("Bildschirm", "Display"), i + 1),
                id: name,
                x: pos.x,
                y: pos.y,
                width: size.width as i32,
                height: size.height as i32,
                scale: m.scale_factor() as f32,
                primary: primary.as_ref() == Some(&m),
            }
        })
        .collect();
    list.sort_by_key(|m| (!m.primary, m.x, m.y));
    list
}

/// Declares the crosshair viewport. Must run on every pass of the root viewport.
pub fn show(ctx: &egui::Context, frame: &eframe::Frame, shared: &Arc<Shared>) {
    if let Some(window) = frame.winit_window() {
        let mut checked = MONITORS_CHECKED.lock().unwrap_or_else(|e| e.into_inner());
        if checked.is_none_or(|t| t.elapsed() > Duration::from_secs(2)) {
            *checked = Some(Instant::now());
            shared.set_monitors(monitors(window));
        }
        ctx.request_repaint_after(Duration::from_secs(2));
    }
    let s = shared.settings();
    if !s.visible {
        return;
    }
    let list = shared.monitors.lock().map(|m| m.clone()).unwrap_or_default();
    let (_, side) = texture(ctx, shared);
    let (x, y, scale) = match Monitor::pick(&list, s.monitor.as_deref()) {
        Some(m) => {
            let half = (side / 2) as i32;
            (m.x + m.width / 2 - half + s.offset[0], m.y + m.height / 2 - half + s.offset[1], m.scale.max(0.5))
        }
        None => (0, 0, 1.0),
    };
    let builder = egui::ViewportBuilder::default()
        .with_title("NeoCross Crosshair")
        .with_app_id("neocross")
        .with_decorations(false)
        .with_transparent(true)
        .with_mouse_passthrough(true)
        .with_window_level(egui::WindowLevel::AlwaysOnTop)
        .with_taskbar(false)
        .with_active(false)
        .with_resizable(false)
        .with_has_shadow(false)
        .with_position([x as f32 / scale, y as f32 / scale])
        .with_inner_size([side as f32 / scale, side as f32 / scale]);
    let shared = shared.clone();
    ctx.show_viewport_deferred(viewport_id(), builder, move |ui, _| {
        let (id, _) = texture(ui.ctx(), &shared);
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        ui.painter().image(id, ui.ctx().viewport_rect(), uv, egui::Color32::WHITE);
    });
}
