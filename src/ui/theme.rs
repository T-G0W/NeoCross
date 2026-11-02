use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle, Visuals,
};
use std::sync::Arc;

pub const BG: Color32 = Color32::from_rgb(0x0B, 0x0E, 0x12);
pub const CARD: Color32 = Color32::from_rgb(0x12, 0x16, 0x1C);
pub const CONTROL: Color32 = Color32::from_rgb(0x19, 0x1E, 0x25);
pub const CONTROL_HOVER: Color32 = Color32::from_rgb(0x22, 0x28, 0x31);
pub const BORDER: Color32 = Color32::from_rgb(0x20, 0x26, 0x2E);
pub const TEXT: Color32 = Color32::from_rgb(0xE9, 0xEC, 0xF0);
pub const DIM: Color32 = Color32::from_rgb(0x95, 0x9D, 0xA9);
pub const FAINT: Color32 = Color32::from_rgb(0x5B, 0x63, 0x6F);
pub const ACCENT: Color32 = Color32::from_rgb(0x00, 0xFF, 0x88);
pub const ACCENT_SOFT: Color32 = Color32::from_rgba_premultiplied(0x00, 0x2E, 0x18, 0x2E);

pub const BOLD: &str = "bold";

pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(BOLD.into()))
}

/// Segoe UI on Windows, the system UI font on macOS, egui's own font elsewhere.
fn system_fonts() -> (Option<Vec<u8>>, Option<Vec<u8>>) {
    let read = |p: &str| std::fs::read(p).ok();
    if cfg!(windows) {
        let dir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
        let f = |n: &str| read(&format!("{dir}\\Fonts\\{n}"));
        (f("segoeui.ttf"), f("seguisb.ttf"))
    } else if cfg!(target_os = "macos") {
        (read("/System/Library/Fonts/SFNS.ttf").or_else(|| read("/System/Library/Fonts/Helvetica.ttc")), None)
    } else {
        let regular = ["/usr/share/fonts/truetype/inter/Inter-Regular.ttf", "/usr/share/fonts/TTF/Inter-Regular.ttf"];
        let semi = ["/usr/share/fonts/truetype/inter/Inter-SemiBold.ttf", "/usr/share/fonts/TTF/Inter-SemiBold.ttf"];
        (regular.iter().find_map(|p| read(p)), semi.iter().find_map(|p| read(p)))
    }
}

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let (regular, semibold) = system_fonts();
    if let Some(bytes) = regular {
        fonts.font_data.insert("ui".into(), Arc::new(FontData::from_owned(bytes)));
        fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "ui".into());
    }
    let proportional = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let mut bold = Vec::new();
    if let Some(bytes) = semibold {
        fonts.font_data.insert("ui-bold".into(), Arc::new(FontData::from_owned(bytes)));
        bold.push("ui-bold".to_string());
    }
    bold.extend(proportional);
    fonts.families.insert(FontFamily::Name(BOLD.into()), bold);
    ctx.set_fonts(fonts);

    let mut v = Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = CARD;
    v.window_stroke = Stroke::new(1.0, BORDER);
    v.extreme_bg_color = Color32::from_rgb(0x0E, 0x11, 0x16);
    v.faint_bg_color = CARD;
    v.override_text_color = None;
    v.hyperlink_color = ACCENT;
    v.selection.bg_fill = Color32::from_rgb(0x0B, 0x5E, 0x3A);
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.slider_trailing_fill = true;
    v.window_corner_radius = CornerRadius::same(10);
    v.menu_corner_radius = CornerRadius::same(8);
    let r = CornerRadius::same(6);
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = CARD;
    w.noninteractive.weak_bg_fill = CARD;
    w.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    w.noninteractive.fg_stroke = Stroke::new(1.0, DIM);
    w.noninteractive.corner_radius = r;
    w.inactive.bg_fill = CONTROL;
    w.inactive.weak_bg_fill = CONTROL;
    w.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    w.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    w.inactive.corner_radius = r;
    w.hovered.bg_fill = CONTROL_HOVER;
    w.hovered.weak_bg_fill = CONTROL_HOVER;
    w.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0x34, 0x3C, 0x47));
    w.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    w.hovered.corner_radius = r;
    w.active.bg_fill = Color32::from_rgb(0x0B, 0x5E, 0x3A);
    w.active.weak_bg_fill = Color32::from_rgb(0x0B, 0x5E, 0x3A);
    w.active.bg_stroke = Stroke::new(1.0, ACCENT);
    w.active.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    w.active.corner_radius = r;
    w.open = w.active;
    ctx.set_visuals_of(egui::Theme::Dark, v);
    ctx.set_theme(egui::Theme::Dark);

    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 7.0);
        style.spacing.button_padding = egui::vec2(9.0, 4.0);
        style.spacing.interact_size.y = 24.0;
        style.spacing.slider_rail_height = 4.0;
        style.text_styles.insert(TextStyle::Body, FontId::proportional(13.5));
        style.text_styles.insert(TextStyle::Button, FontId::proportional(13.0));
        style.text_styles.insert(TextStyle::Small, FontId::proportional(11.0));
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    });
}
