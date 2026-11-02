//! The NeoCross menu.

mod theme;

use crate::app::{Action, Monitor, Shared};
use crate::backdrop::{self, Backdrop};
use crate::crosshair::{
    self, COLORS, GAP_RANGE, OUTLINE_RANGE, ROTATION_RANGE, SIZE_RANGE, SIZES, Shape, THICKNESS_RANGE,
};
use crate::i18n::{self, Language};
use crate::settings::{MenuKey, Settings};
use crate::{assets, autostart, paths, platform, t};
use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Layout, Margin, Pos2, Rect, RichText, Sense, Stroke, StrokeKind,
    TextureHandle, TextureOptions, Ui, Vec2, pos2, vec2,
};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use theme::{ACCENT, ACCENT_SOFT, BG, BORDER, CARD, CONTROL, CONTROL_HOVER, DIM, FAINT, TEXT, bold};

pub const WIDTH: f32 = 800.0;
pub const HEIGHT: f32 = 660.0;
const PAD: f32 = 16.0;
const GUTTER: f32 = 10.0;
const LEFT: f32 = 344.0;

pub enum Mode {
    /// macOS and Linux: one process with menu, tray, hotkeys and the crosshair viewport.
    #[cfg_attr(windows, allow(dead_code))]
    Standalone { visible_at_start: bool },
    /// Windows: the menu process the core starts while the menu is open.
    #[cfg_attr(not(windows), allow(dead_code))]
    Menu { activate: bool, position: Option<[f32; 2]> },
    /// README screenshots.
    #[cfg_attr(not(feature = "docs"), allow(dead_code))]
    Docs(std::path::PathBuf),
}

pub fn run(shared: Arc<Shared>, mode: Mode) -> eframe::Result<()> {
    let (rgba, w, h) = assets::rgba(assets::LOGO_256);
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(format!("NeoCross {} – by T-GOW", paths::VERSION))
        .with_app_id("neocross")
        .with_inner_size([WIDTH, HEIGHT])
        // Only the height can change, so small screens can still scroll to everything.
        .with_min_inner_size([WIDTH, 360.0])
        .with_max_inner_size([WIDTH, 4096.0])
        .with_resizable(true)
        .with_maximize_button(false)
        .with_window_level(egui::WindowLevel::AlwaysOnTop)
        .with_icon(egui::IconData { rgba, width: w, height: h });
    let mut centered = true;
    match &mode {
        Mode::Menu { activate, position } => {
            // Opened by the menu key in a game: show it, but leave the keyboard with the game.
            viewport = viewport.with_active(*activate);
            if let Some(p) = position {
                viewport = viewport.with_position(*p);
                centered = false;
            }
        }
        Mode::Standalone { .. } | Mode::Docs(_) => {}
    }
    #[cfg(windows)]
    if centered && let Some(m) = crate::overlay::monitors().into_iter().find(|m| m.primary) {
        // Top right like NeoCross 1.x, so the menu never covers the crosshair.
        let s = m.scale.max(0.5);
        viewport = viewport.with_position([(m.x + m.width) as f32 / s - WIDTH - 28.0, m.y as f32 / s + 28.0]);
        centered = false;
    }
    let options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Glow,
        centered,
        persist_window: false,
        ..Default::default()
    };
    eframe::run_native("NeoCross", options, Box::new(move |cc| Ok(Box::new(NeoApp::new(cc, shared, mode)))))
}

#[derive(Clone, PartialEq)]
struct ThumbKey {
    color: [u8; 3],
    thickness: f32,
    outline: bool,
    outline_color: [u8; 4],
    dot: bool,
    ppp: f32,
}

struct Preview {
    backdrop: (Backdrop, [u32; 2]),
    background: TextureHandle,
    rev: u64,
    crosshair: TextureHandle,
    side: u32,
}

pub struct NeoApp {
    shared: Arc<Shared>,
    menu_open: bool,
    quitting: bool,
    /// Frames still to paint before a menu that starts hidden is hidden.
    #[cfg(not(windows))]
    hide_after: Option<u32>,
    seen_rev: u64,
    logo: TextureHandle,
    preview: Option<Preview>,
    thumbs: Option<(ThumbKey, Vec<TextureHandle>)>,
    backdrop: Backdrop,
    zoom: u32,
    autostart: bool,
    typing: bool,
    #[cfg(target_os = "macos")]
    tray: Option<crate::tray::Tray>,
    #[cfg(target_os = "macos")]
    tray_lang: bool,
    #[cfg(not(windows))]
    hotkeys: Option<crate::hotkeys::Hotkeys>,
    docs: Option<docs::Job>,
    /// Content height the window was last sized for.
    fitted: f32,
    #[cfg(windows)]
    monitors_checked: Option<std::time::Instant>,
}

impl NeoApp {
    fn new(cc: &eframe::CreationContext<'_>, shared: Arc<Shared>, mode: Mode) -> Self {
        theme::install(&cc.egui_ctx);
        shared.attach_ui(&cc.egui_ctx);
        let (rgba, w, h) = assets::rgba(assets::LOGO_256);
        let logo = cc.egui_ctx.load_texture(
            "logo",
            egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba),
            TextureOptions::LINEAR,
        );
        let (visible, standalone, docs) = match mode {
            Mode::Standalone { visible_at_start } => (visible_at_start, true, None),
            Mode::Menu { .. } => (true, false, None),
            Mode::Docs(dir) => (true, false, Some(docs::Job::new(dir))),
        };
        #[cfg(target_os = "linux")]
        if standalone {
            crate::tray::spawn_linux(shared.clone());
        }
        let _ = standalone;
        shared.menu_open.store(visible, Ordering::Relaxed);
        NeoApp {
            menu_open: visible,
            quitting: false,
            #[cfg(not(windows))]
            hide_after: (!visible).then_some(2),
            seen_rev: 0,
            logo,
            preview: None,
            thumbs: None,
            backdrop: Backdrop::Scene,
            zoom: 1,
            autostart: autostart::is_enabled(),
            typing: false,
            #[cfg(target_os = "macos")]
            tray: if standalone { crate::tray::Tray::new(shared.clone()) } else { None },
            #[cfg(target_os = "macos")]
            tray_lang: i18n::is_en(),
            #[cfg(not(windows))]
            hotkeys: standalone.then(|| crate::hotkeys::Hotkeys::new(shared.clone())),
            docs,
            fitted: 0.0,
            #[cfg(windows)]
            monitors_checked: None,
            shared,
        }
    }

    fn set_open(&mut self, ctx: &egui::Context, frame: &eframe::Frame, open: bool, activate: bool) {
        #[cfg(windows)]
        {
            // The menu process only lives while the menu is open.
            if open {
                platform::show_or_hide(frame, true, activate);
            } else {
                self.quit(ctx);
            }
        }
        #[cfg(not(windows))]
        {
            let _ = frame;
            self.menu_open = open;
            self.shared.menu_open.store(open, Ordering::Relaxed);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(open));
            if open && activate {
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
            if open {
                self.autostart = autostart::is_enabled();
            }
            ctx.request_repaint();
        }
    }

    /// Sizes the window to its content once (and again when the content changes, e.g. the
    /// language), but never taller than the screen; anything beyond that scrolls.
    fn fit_height(&mut self, ctx: &egui::Context, needed: f32) {
        if (needed - self.fitted).abs() < 4.0 {
            return;
        }
        self.fitted = needed;
        let screen = ctx.input(|i| i.viewport().monitor_size).map_or(800.0, |m| m.y);
        let height = needed.ceil().min(screen - 90.0).max(360.0);
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(vec2(WIDTH, height)));
    }

    fn quit(&mut self, ctx: &egui::Context) {
        if self.quitting {
            return;
        }
        self.quitting = true;
        #[cfg(windows)]
        if let Some(r) = ctx.input(|i| i.viewport().outer_rect) {
            crate::ipc::to_core(crate::ipc::Msg::Position([r.min.x, r.min.y]));
        }
        #[cfg(not(windows))]
        self.shared.save_now();
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

impl eframe::App for NeoApp {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        #[cfg(not(windows))]
        if let Some(n) = self.hide_after {
            if n == 0 {
                self.hide_after = None;
                self.set_open(ctx, frame, false, false);
            } else {
                self.hide_after = Some(n - 1);
                ctx.request_repaint();
            }
        }
        #[cfg(windows)]
        {
            if self.seen_rev == 0 {
                platform::dark_title_bar(frame);
            }
            // The menu process has its own view of the displays for the display list.
            if self.monitors_checked.is_none_or(|t| t.elapsed() > std::time::Duration::from_secs(2)) {
                self.monitors_checked = Some(std::time::Instant::now());
                self.shared.set_monitors(crate::overlay::monitors());
            }
        }

        if ctx.input(|i| i.viewport().close_requested()) && !self.quitting {
            if cfg!(windows) {
                self.quit(ctx);
            } else {
                // The close button only hides the menu; NeoCross keeps running in the tray.
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.set_open(ctx, frame, false, false);
            }
        }
        for action in self.shared.take_actions() {
            match action {
                Action::ToggleMenu { activate } => {
                    let open = !self.menu_open;
                    self.set_open(ctx, frame, open, activate);
                }
                Action::ShowMenu => self.set_open(ctx, frame, true, true),
                Action::Quit => self.quit(ctx),
            }
        }

        let rev = self.shared.rev();
        if rev != self.seen_rev {
            self.seen_rev = rev;
            #[cfg(target_os = "macos")]
            {
                if self.tray_lang != i18n::is_en() && self.tray.is_some() {
                    self.tray_lang = i18n::is_en();
                    self.tray = None;
                    self.tray = crate::tray::Tray::new(self.shared.clone());
                }
                if let Some(tray) = &self.tray {
                    tray.sync(&self.shared.settings());
                }
            }
            #[cfg(not(windows))]
            if !self.menu_open {
                // Lets the root pass run once so the crosshair viewport follows the change.
                ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaintWhileHidden);
            }
        }
        #[cfg(not(windows))]
        if let Some(hk) = &mut self.hotkeys {
            hk.sync(self.shared.settings().menu_key, self.menu_open);
        }
    }

    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        #[cfg(not(windows))]
        crate::overlay::show(ui.ctx(), frame, &self.shared);
        let _ = &frame;

        let mut s = self.shared.settings();
        let before = s.clone();
        egui::CentralPanel::no_frame().frame(Frame::new().fill(BG).inner_margin(Margin::same(PAD as i8))).show(
            ui,
            |ui| {
                self.header(ui, &mut s);
                ui.add_space(10.0);
                let footer_h = 26.0;
                let out = egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - footer_h)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = GUTTER;
                            let h = ui.available_height();
                            ui.allocate_ui_with_layout(vec2(LEFT, h), Layout::top_down(Align::LEFT), |ui| {
                                ui.set_width(LEFT);
                                ui.spacing_mut().item_spacing.y = GUTTER;
                                self.preview_card(ui, &mut s);
                                self.shape_card(ui, &mut s);
                                self.position_card(ui, &mut s);
                            });
                            let right = WIDTH - 2.0 * PAD - LEFT - GUTTER;
                            ui.allocate_ui_with_layout(vec2(right, h), Layout::top_down(Align::LEFT), |ui| {
                                ui.set_width(right);
                                ui.spacing_mut().item_spacing.y = GUTTER;
                                self.color_card(ui, &mut s);
                                self.size_card(ui, &mut s);
                                self.details_card(ui, &mut s);
                                self.general_card(ui, &mut s);
                            });
                        });
                    });
                // Scroll area top (header and margin) + its content + footer + bottom margin.
                self.fit_height(ui.ctx(), out.inner_rect.top() + out.content_size.y + footer_h + PAD + 8.0);
                self.footer(ui, &s);
            },
        );
        if s != before {
            if s.language != before.language {
                i18n::set_language(s.language);
            }
            self.shared.update(|x| *x = s);
        }
        let typing = ui.ctx().egui_wants_keyboard_input();
        if typing != self.typing {
            self.typing = typing;
            self.shared.typing.store(typing, Ordering::Relaxed);
            #[cfg(windows)]
            crate::ipc::to_core(crate::ipc::Msg::Typing(typing));
        }

        if let Some(job) = &mut self.docs
            && job.step(ui.ctx(), &self.shared)
        {
            self.docs = None;
            self.quit(ui.ctx());
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Transparent, for the crosshair viewport on macOS and Linux. The menu paints its own background.
        [0.0; 4]
    }
}

// ---------------------------------------------------------------------------------------------
// Building blocks

fn card<R>(ui: &mut Ui, title: &str, key: Option<&str>, add: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(10)
        .inner_margin(Margin::symmetric(13, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(title.to_uppercase()).font(bold(10.5)).color(FAINT).extra_letter_spacing(1.4));
                if let Some(k) = key {
                    keycap(ui, k);
                }
            });
            ui.add_space(4.0);
            add(ui)
        })
        .inner
}

fn keycap(ui: &mut Ui, key: &str) {
    Frame::new()
        .fill(CONTROL)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(4)
        .inner_margin(Margin::symmetric(5, 0))
        .show(ui, |ui| {
            ui.label(RichText::new(key).font(bold(10.0)).color(DIM));
        });
}

/// A label column of fixed width, then the control.
fn row(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(vec2(92.0, 22.0), Layout::left_to_right(Align::Center), |ui| {
            ui.set_min_width(92.0);
            ui.label(RichText::new(label).color(DIM));
        });
        add(ui);
    });
}

fn slider(ui: &mut Ui, value: &mut f32, range: std::ops::RangeInclusive<f32>, suffix: &str) -> egui::Response {
    ui.spacing_mut().slider_width = (ui.available_width() - 74.0).max(60.0);
    ui.add(egui::Slider::new(value, range).step_by(1.0).suffix(suffix).max_decimals(0))
}

fn chip(ui: &mut Ui, text: &str, selected: bool, min_width: f32) -> egui::Response {
    let (fill, stroke, color) = if selected {
        (ACCENT_SOFT, Stroke::new(1.0, ACCENT), ACCENT)
    } else {
        (CONTROL, Stroke::new(1.0, BORDER), TEXT)
    };
    ui.add(
        egui::Button::new(RichText::new(text).font(bold(11.5)).color(color))
            .fill(fill)
            .stroke(stroke)
            .corner_radius(6)
            .min_size(vec2(min_width, 24.0)),
    )
}

fn switch(ui: &mut Ui, on: &mut bool) -> egui::Response {
    let (rect, mut resp) = ui.allocate_exact_size(vec2(36.0, 20.0), Sense::click());
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    let t = ui.ctx().animate_bool_responsive(resp.id, *on);
    let off = if resp.hovered() { CONTROL_HOVER } else { CONTROL };
    let fill = lerp_color(off, Color32::from_rgb(0x0B, 0x7A, 0x4A), t);
    let stroke = Stroke::new(1.0, lerp_color(BORDER, ACCENT, t));
    ui.painter().rect(rect, CornerRadius::same(10), fill, stroke, StrokeKind::Inside);
    let x = egui::lerp(rect.left() + 10.0..=rect.right() - 10.0, t);
    ui.painter().circle_filled(pos2(x, rect.center().y), 7.0, lerp_color(DIM, Color32::WHITE, t));
    resp
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

fn texture(ctx: &egui::Context, name: &str, pm: &tiny_skia::Pixmap, options: TextureOptions) -> TextureHandle {
    let image = egui::ColorImage::from_rgba_premultiplied([pm.width() as usize, pm.height() as usize], pm.data());
    ctx.load_texture(name, image, options)
}

/// Places a texture so that its pixels land exactly on screen pixels, centered on `center`.
fn pixel_rect(center: Pos2, side_px: f32, ppp: f32, zoom: f32) -> Rect {
    let (cx, cy) = ((center.x * ppp).round(), (center.y * ppp).round());
    let half = side_px * zoom / 2.0;
    Rect::from_min_size(pos2((cx - half) / ppp, (cy - half) / ppp), Vec2::splat(side_px * zoom / ppp))
}

const FULL_UV: Rect = Rect { min: Pos2 { x: 0.0, y: 0.0 }, max: Pos2 { x: 1.0, y: 1.0 } };

// ---------------------------------------------------------------------------------------------
// Sections

impl NeoApp {
    fn header(&mut self, ui: &mut Ui, s: &mut Settings) {
        ui.horizontal(|ui| {
            ui.add(egui::Image::new(&self.logo).fit_to_exact_size(vec2(32.0, 32.0)));
            ui.add_space(2.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.horizontal(|ui| {
                    ui.label(RichText::new("NeoCross").font(bold(20.0)).color(TEXT));
                    ui.label(RichText::new(paths::VERSION).font(bold(11.0)).color(FAINT));
                });
                ui.label(RichText::new(t!("Fadenkreuz-Overlay", "Crosshair overlay")).size(11.5).color(DIM));
            });
            ui.add_space(6.0);
            Frame::new()
                .fill(ACCENT_SOFT)
                .stroke(Stroke::new(1.0, Color32::from_rgb(0x0B, 0x7A, 0x4A)))
                .corner_radius(4)
                .inner_margin(Margin::symmetric(6, 2))
                .show(ui, |ui| {
                    ui.label(RichText::new("BY T-GOW").font(bold(9.5)).color(ACCENT).extra_letter_spacing(1.0));
                });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                switch(ui, &mut s.visible).on_hover_text(t!("Fadenkreuz ein/aus (V)", "Crosshair on/off (V)"));
                let state =
                    if s.visible { t!("Fadenkreuz an", "Crosshair on") } else { t!("Fadenkreuz aus", "Crosshair off") };
                ui.label(RichText::new(state).color(if s.visible { TEXT } else { DIM }));
            });
        });
    }

    fn preview_card(&mut self, ui: &mut Ui, s: &mut Settings) {
        card(ui, t!("Vorschau", "Preview"), None, |ui| {
            let ctx = ui.ctx().clone();
            let ppp = ctx.pixels_per_point();
            let size = vec2(ui.available_width(), 152.0);
            let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
            let px = [(size.x * ppp).round() as u32, (size.y * ppp).round() as u32];
            let rev = self.shared.rev();
            let stale_bg = self.preview.as_ref().is_none_or(|p| p.backdrop != (self.backdrop, px));
            let stale_ch = self.preview.as_ref().is_none_or(|p| p.rev != rev);
            if stale_bg || stale_ch {
                let background = match (&self.preview, stale_bg) {
                    (Some(p), false) => p.background.clone(),
                    _ => texture(
                        &ctx,
                        "preview-bg",
                        &backdrop::render(self.backdrop, px[0], px[1], ppp),
                        TextureOptions::LINEAR,
                    ),
                };
                let pm = crosshair::render(&s.style);
                let crosshair = texture(&ctx, "preview-crosshair", &pm, TextureOptions::NEAREST);
                self.preview =
                    Some(Preview { backdrop: (self.backdrop, px), background, rev, crosshair, side: pm.width() });
            }
            let Some(p) = &self.preview else { return };
            ui.put(rect, egui::Image::new((p.background.id(), size)).corner_radius(8));
            if s.visible {
                let r = pixel_rect(rect.center(), p.side as f32, ppp, self.zoom as f32);
                ui.painter().with_clip_rect(rect).image(p.crosshair.id(), r, FULL_UV, Color32::WHITE);
            } else {
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    t!("Fadenkreuz ausgeblendet", "Crosshair hidden"),
                    bold(13.0),
                    Color32::from_white_alpha(200),
                );
            }
            ui.painter().rect_stroke(rect, 8, Stroke::new(1.0, BORDER), StrokeKind::Inside);

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                for b in Backdrop::ALL {
                    if chip(ui, b.label(), self.backdrop == b, 0.0).clicked() {
                        self.backdrop = b;
                    }
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    for z in [4, 2, 1] {
                        if chip(ui, &format!("{z}×"), self.zoom == z, 30.0)
                            .on_hover_text(t!("Vergrößerte Vorschau", "Magnified preview"))
                            .clicked()
                        {
                            self.zoom = z;
                        }
                    }
                });
            });
        });
    }

    fn shape_card(&mut self, ui: &mut Ui, s: &mut Settings) {
        card(ui, t!("Form", "Shape"), Some("X"), |ui| {
            let ctx = ui.ctx().clone();
            let ppp = ctx.pixels_per_point();
            let key = ThumbKey {
                color: s.style.color,
                thickness: s.style.thickness.clamp(1.0, 2.0),
                outline: s.style.outline,
                outline_color: s.style.outline_color,
                dot: s.style.dot,
                ppp,
            };
            if self.thumbs.as_ref().is_none_or(|(k, _)| *k != key) {
                let list = Shape::ALL
                    .iter()
                    .map(|shape| {
                        let mut st = crosshair::thumbnail_style(&s.style, *shape);
                        st.size = (st.size * ppp).round();
                        st.thickness = (st.thickness * ppp).round().max(1.0);
                        texture(&ctx, shape.id(), &crosshair::render(&st), TextureOptions::NEAREST)
                    })
                    .collect();
                self.thumbs = Some((key, list));
            }
            let Some((_, thumbs)) = &self.thumbs else {
                return;
            };
            let gap = 5.0;
            let per_row = 7;
            let cell = ((ui.available_width() - gap * (per_row as f32 - 1.0)) / per_row as f32).floor();
            for chunk in Shape::ALL.chunks(per_row) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for shape in chunk {
                        let (rect, resp) = ui.allocate_exact_size(Vec2::splat(cell), Sense::click());
                        let selected = s.style.shape == *shape;
                        let (fill, stroke) = if selected {
                            (ACCENT_SOFT, Stroke::new(1.5, ACCENT))
                        } else if resp.hovered() {
                            (CONTROL_HOVER, Stroke::new(1.0, Color32::from_rgb(0x34, 0x3C, 0x47)))
                        } else {
                            (CONTROL, Stroke::new(1.0, BORDER))
                        };
                        ui.painter().rect(rect, 7, fill, stroke, StrokeKind::Inside);
                        let tex = &thumbs[shape.index()];
                        let side = tex.size()[0] as f32;
                        ui.painter().image(
                            tex.id(),
                            pixel_rect(rect.center(), side, ppp, 1.0),
                            FULL_UV,
                            Color32::WHITE,
                        );
                        let resp = resp.on_hover_text(shape.name());
                        if resp.clicked() {
                            s.style.shape = *shape;
                        }
                    }
                });
                ui.add_space(gap - ui.spacing().item_spacing.y);
            }
            ui.horizontal(|ui| {
                ui.label(RichText::new(s.style.shape.name()).font(bold(12.5)).color(TEXT));
                ui.label(
                    RichText::new(format!("{} / {}", s.style.shape.index() + 1, Shape::ALL.len()))
                        .size(11.5)
                        .color(FAINT),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(t!("Shift+X = zurück", "Shift+X = back")).size(11.0).color(FAINT));
                });
            });
        });
    }

    fn position_card(&mut self, ui: &mut Ui, s: &mut Settings) {
        card(ui, t!("Position", "Position"), None, |ui| {
            let monitors = self.shared.monitors.lock().map(|m| m.clone()).unwrap_or_default();
            let current = Monitor::pick(&monitors, s.monitor.as_deref());
            let describe = |m: &Monitor| {
                let primary = if m.primary { t!(" · Haupt", " · primary") } else { "" };
                format!("{} · {}×{}{primary}", m.label, m.width, m.height)
            };
            row(ui, t!("Bildschirm", "Display"), |ui| {
                let text = match (&s.monitor, current) {
                    (None, Some(m)) => format!("{} · {}×{}", t!("Hauptbildschirm", "Primary"), m.width, m.height),
                    (_, Some(m)) => describe(m),
                    _ => t!("Hauptbildschirm", "Primary display").to_string(),
                };
                egui::ComboBox::from_id_salt("monitor")
                    .width(ui.available_width())
                    .truncate()
                    .selected_text(text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut s.monitor,
                            None,
                            t!("Immer der Hauptbildschirm", "Always the primary display"),
                        );
                        for m in &monitors {
                            ui.selectable_value(&mut s.monitor, Some(m.id.clone()), describe(m));
                        }
                    });
            });
            row(ui, t!("Feinjustierung", "Fine-tune"), |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.label(RichText::new("X").color(FAINT));
                ui.add(egui::DragValue::new(&mut s.offset[0]).range(-2000..=2000).suffix(" px").speed(0.2));
                ui.add_space(4.0);
                ui.label(RichText::new("Y").color(FAINT));
                ui.add(egui::DragValue::new(&mut s.offset[1]).range(-2000..=2000).suffix(" px").speed(0.2));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let centered = s.offset == [0, 0];
                    if ui
                        .add_enabled(!centered, egui::Button::new(t!("Mitte", "Center")).small())
                        .on_hover_text(t!("Genau auf die Bildschirmmitte", "Exactly at the screen center"))
                        .clicked()
                    {
                        s.offset = [0, 0];
                    }
                });
            });
        });
    }

    fn color_card(&mut self, ui: &mut Ui, s: &mut Settings) {
        card(ui, t!("Farbe", "Color"), Some("Z"), |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 5.0;
                for (rgb, de, en) in COLORS {
                    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
                    let c = rect.center();
                    let selected = s.style.color == rgb;
                    if selected {
                        ui.painter().circle_stroke(c, 11.0, Stroke::new(2.0, Color32::WHITE));
                    } else if resp.hovered() {
                        ui.painter().circle_stroke(c, 11.0, Stroke::new(1.0, DIM));
                    }
                    ui.painter().circle_filled(c, 8.0, Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
                    if rgb == [0, 0, 0] {
                        ui.painter().circle_stroke(c, 8.0, Stroke::new(1.0, Color32::from_gray(70)));
                    }
                    if resp.on_hover_text(t!(de, en)).clicked() {
                        s.style.color = rgb;
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new(crosshair::color_name(s.style.color)).size(11.5).color(DIM));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::color_picker::color_edit_button_srgb(ui, &mut s.style.color)
                        .on_hover_text(t!("Eigene Farbe wählen", "Pick a custom color"));
                    let custom = !COLORS.iter().any(|c| c.0 == s.style.color);
                    ui.label(RichText::new(t!("Eigene Farbe", "Custom color")).size(11.5).color(if custom {
                        TEXT
                    } else {
                        FAINT
                    }));
                });
            });
        });
    }

    fn size_card(&mut self, ui: &mut Ui, s: &mut Settings) {
        card(ui, t!("Größe", "Size"), Some("C"), |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 5.0;
                let w = ((ui.available_width() - 5.0 * 5.0) / 6.0).floor();
                for (px, label) in SIZES {
                    if chip(ui, label, (s.style.size - px).abs() < 0.5, w).on_hover_text(format!("{px} px")).clicked() {
                        s.style.size = px;
                    }
                }
            });
            row(ui, t!("Länge", "Length"), |ui| {
                slider(ui, &mut s.style.size, SIZE_RANGE, " px");
            });
        });
    }

    fn details_card(&mut self, ui: &mut Ui, s: &mut Settings) {
        card(ui, t!("Feinschliff", "Details"), None, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            let st = &mut s.style;
            row(ui, t!("Dicke", "Thickness"), |ui| {
                slider(ui, &mut st.thickness, THICKNESS_RANGE, " px");
            });
            row(ui, t!("Lücke", "Gap"), |ui| {
                let uses = st.shape.uses_gap();
                let mut auto = st.gap.is_none();
                if ui
                    .add_enabled(uses, egui::Checkbox::new(&mut auto, "Auto"))
                    .on_disabled_hover_text(t!("Diese Form hat keine Lücke", "This shape has no gap"))
                    .changed()
                {
                    st.gap = if auto { None } else { Some(st.shape.auto_gap(st.size)) };
                }
                let mut g = st.gap_px();
                ui.add_enabled_ui(uses && !auto, |ui| {
                    if slider(ui, &mut g, GAP_RANGE, " px").changed() {
                        st.gap = Some(g);
                    }
                });
            });
            row(ui, t!("Deckkraft", "Opacity"), |ui| {
                let mut pct = (st.opacity * 100.0).round();
                if slider(ui, &mut pct, 10.0..=100.0, " %").changed() {
                    st.opacity = pct / 100.0;
                }
            });
            row(ui, t!("Drehung", "Rotation"), |ui| {
                slider(ui, &mut st.rotation, ROTATION_RANGE, "°");
            });
            row(ui, t!("Kontur", "Outline"), |ui| {
                switch(ui, &mut st.outline)
                    .on_hover_text(t!("Dunkler Rand für helle Szenen", "Dark edge for bright scenes"));
                ui.add_enabled_ui(st.outline, |ui| {
                    ui.spacing_mut().slider_width = (ui.available_width() - 120.0).max(40.0);
                    ui.add(
                        egui::Slider::new(&mut st.outline_width, OUTLINE_RANGE)
                            .step_by(1.0)
                            .suffix(" px")
                            .max_decimals(0),
                    );
                    let mut c = Color32::from_rgba_unmultiplied(
                        st.outline_color[0],
                        st.outline_color[1],
                        st.outline_color[2],
                        st.outline_color[3],
                    );
                    if egui::color_picker::color_edit_button_srgba(ui, &mut c, egui::color_picker::Alpha::OnlyBlend)
                        .on_hover_text(t!("Farbe und Deckkraft der Kontur", "Outline color and opacity"))
                        .changed()
                    {
                        st.outline_color = c.to_srgba_unmultiplied();
                    }
                });
            });
            row(ui, t!("Mittelpunkt", "Center dot"), |ui| {
                let own = matches!(st.shape, Shape::Dot | Shape::CircleDot | Shape::Reticle);
                ui.add_enabled_ui(!own, |ui| {
                    switch(ui, &mut st.dot);
                });
                if own {
                    ui.label(RichText::new(t!("hat die Form schon", "built into this shape")).size(11.5).color(FAINT));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(egui::Button::new(t!("Stil zurücksetzen", "Reset style")).small())
                        .on_hover_text(t!(
                            "Form, Farbe und Regler auf Standard",
                            "Shape, color and sliders back to default"
                        ))
                        .clicked()
                    {
                        *st = crosshair::Style::default();
                    }
                });
            });
        });
    }

    fn general_card(&mut self, ui: &mut Ui, s: &mut Settings) {
        card(ui, t!("Allgemein", "General"), None, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            row(ui, t!("Menütaste", "Menu key"), |ui| {
                egui::ComboBox::from_id_salt("menu-key").width(110.0).selected_text(s.menu_key.label()).show_ui(
                    ui,
                    |ui| {
                        for k in MenuKey::ALL {
                            ui.selectable_value(&mut s.menu_key, k, k.label());
                        }
                    },
                );
                ui.label(
                    RichText::new(t!("öffnet/schließt das Menü", "opens/closes this menu")).size(11.5).color(FAINT),
                );
            });
            row(ui, t!("Sprache", "Language"), |ui| {
                egui::ComboBox::from_id_salt("language").width(160.0).selected_text(s.language.label()).show_ui(
                    ui,
                    |ui| {
                        for l in Language::ALL {
                            ui.selectable_value(&mut s.language, l, l.label());
                        }
                    },
                );
            });
            row(ui, "Autostart", |ui| {
                let mut on = self.autostart;
                if switch(ui, &mut on).changed() {
                    autostart::set(on);
                    self.autostart = autostart::is_enabled();
                    self.shared.mark_dirty();
                }
                let text = if cfg!(windows) {
                    t!("Mit Windows starten", "Start with Windows")
                } else {
                    t!("Beim Anmelden starten", "Start at login")
                };
                ui.label(RichText::new(text).color(if self.autostart { TEXT } else { DIM }));
            });
        });
    }

    fn footer(&mut self, ui: &mut Ui, s: &Settings) {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let items = [
                (s.menu_key.label(), t!("Menü", "Menu")),
                ("X", t!("Form", "Shape")),
                ("Z", t!("Farbe", "Color")),
                ("C", t!("Größe", "Size")),
                ("V", t!("Ein/Aus", "On/Off")),
                ("F3", t!("Beenden", "Quit")),
            ];
            for (i, (key, what)) in items.iter().enumerate() {
                if i > 0 {
                    ui.label(RichText::new("·").color(FAINT));
                }
                keycap(ui, key);
                ui.label(RichText::new(*what).size(11.5).color(DIM));
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(format!("v{}", paths::VERSION)).size(11.5).color(FAINT));
                if ui.link(RichText::new("GitHub").size(11.5)).clicked() {
                    platform::open_url(paths::REPO_URL);
                }
                ui.label(RichText::new(t!("Keine Telemetrie ·", "No telemetry ·")).size(11.5).color(FAINT));
            });
        });
    }
}

// ---------------------------------------------------------------------------------------------
// README screenshots (`--docs <dir>`, only with the `docs` feature)

mod docs {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Mutex;

    pub struct Job {
        dir: PathBuf,
        frame: u32,
        shot: usize,
        pending: Arc<Mutex<Option<Arc<egui::ColorImage>>>>,
    }

    /// A menu state captured for the README: file, language, style tweak.
    type Shot = (&'static str, Language, fn(&mut Settings));

    fn shots() -> Vec<Shot> {
        vec![
            ("menu-en.png", Language::En, |s| {
                s.style = crosshair::Style {
                    shape: Shape::CrossGap,
                    color: COLORS[0].0,
                    size: 12.0,
                    thickness: 2.0,
                    gap: Some(4.0),
                    dot: true,
                    ..crosshair::Style::default()
                }
            }),
            ("menu-de.png", Language::De, |s| {
                s.style = crosshair::Style {
                    shape: Shape::Hexagon,
                    color: COLORS[2].0,
                    size: 14.0,
                    dot: true,
                    ..crosshair::Style::default()
                }
            }),
        ]
    }

    impl Job {
        pub fn new(dir: PathBuf) -> Job {
            Job { dir, frame: 0, shot: 0, pending: Arc::default() }
        }

        /// Returns `true` when all screenshots are written.
        pub fn step(&mut self, ctx: &egui::Context, shared: &Shared) -> bool {
            let list = shots();
            ctx.request_repaint();
            if let Some(img) = self.pending.lock().ok().and_then(|mut p| p.take()) {
                let (file, ..) = list[self.shot];
                let [w, h] = img.size;
                let mut rgba = Vec::with_capacity(w * h * 4);
                for p in &img.pixels {
                    rgba.extend_from_slice(&[p.r(), p.g(), p.b(), 255]);
                }
                if let Some(pm) =
                    tiny_skia::IntSize::from_wh(w as u32, h as u32).and_then(|s| tiny_skia::Pixmap::from_vec(rgba, s))
                {
                    let _ = std::fs::create_dir_all(&self.dir);
                    let _ = pm.save_png(self.dir.join(file));
                }
                self.shot += 1;
                self.frame = 0;
            }
            if self.shot >= list.len() {
                return true;
            }
            self.frame += 1;
            match self.frame {
                1 => {
                    let (_, lang, tweak) = list[self.shot];
                    i18n::set_language(lang);
                    shared.update(|s| {
                        s.language = lang;
                        s.visible = true;
                        tweak(s);
                    });
                }
                // Give fonts, textures and animations time to settle.
                20 => {
                    let slot = self.pending.clone();
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::ScreenshotCallback::new(
                        move |img| {
                            if let Ok(mut s) = slot.lock() {
                                *s = Some(img);
                            }
                        },
                    )));
                }
                _ => {}
            }
            false
        }
    }
}
