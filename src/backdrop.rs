//! Backgrounds for the preview: a crosshair has to read well on dark, bright and busy scenes.

use crate::t;
use serde::{Deserialize, Serialize};
use tiny_skia::{
    Color, FillRule, GradientStop, LinearGradient, Paint, PathBuilder, Pixmap, Point, Rect, SpreadMode, Transform,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Backdrop {
    #[default]
    Scene,
    Dark,
    Light,
}

impl Backdrop {
    pub const ALL: [Backdrop; 3] = [Backdrop::Scene, Backdrop::Dark, Backdrop::Light];

    pub fn label(self) -> &'static str {
        match self {
            Backdrop::Scene => t!("Szene", "Scene"),
            Backdrop::Dark => t!("Dunkel", "Dark"),
            Backdrop::Light => t!("Hell", "Bright"),
        }
    }
}

fn rgb(hex: u32) -> Color {
    Color::from_rgba8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8, 255)
}

fn vertical(pm: &mut Pixmap, y0: f32, y1: f32, stops: &[(f32, u32)]) {
    let w = pm.width() as f32;
    let Some(rect) = Rect::from_xywh(0.0, y0, w, (y1 - y0).max(1.0)) else {
        return;
    };
    let stops = stops.iter().map(|(p, c)| GradientStop::new(*p, rgb(*c))).collect();
    let Some(shader) = LinearGradient::new(
        Point::from_xy(0.0, y0),
        Point::from_xy(0.0, y1),
        stops,
        SpreadMode::Pad,
        Transform::identity(),
    ) else {
        return;
    };
    let paint = Paint { shader, anti_alias: false, ..Paint::default() };
    pm.fill_rect(rect, &paint, Transform::identity(), None);
}

fn polygon(pm: &mut Pixmap, pts: &[(f32, f32)], color: u32, alpha: u8) {
    let mut pb = PathBuilder::new();
    pb.move_to(pts[0].0, pts[0].1);
    for p in &pts[1..] {
        pb.line_to(p.0, p.1);
    }
    pb.close();
    let Some(path) = pb.finish() else { return };
    let mut paint = Paint::default();
    let c = rgb(color);
    paint.set_color_rgba8((c.red() * 255.0) as u8, (c.green() * 255.0) as u8, (c.blue() * 255.0) as u8, alpha);
    paint.anti_alias = true;
    pm.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
}

fn rect(pm: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, color: u32, alpha: u8) {
    polygon(pm, &[(x, y), (x + w, y), (x + w, y + h), (x, y + h)], color, alpha);
}

/// Paints a `w`×`h` background. `scale` is the device pixel ratio, so details keep their size.
pub fn render(kind: Backdrop, w: u32, h: u32, scale: f32) -> Pixmap {
    let mut pm = Pixmap::new(w.max(1), h.max(1)).expect("backdrop");
    let (fw, fh) = (w as f32, h as f32);
    match kind {
        Backdrop::Dark => {
            vertical(&mut pm, 0.0, fh, &[(0.0, 0x1B2029), (1.0, 0x0C0F14)]);
            let step = 24.0 * scale;
            let mut x = (fw / 2.0) % step;
            while x < fw {
                rect(&mut pm, x.floor(), 0.0, 1.0, fh, 0xFFFFFF, 9);
                x += step;
            }
            let mut y = (fh / 2.0) % step;
            while y < fh {
                rect(&mut pm, 0.0, y.floor(), fw, 1.0, 0xFFFFFF, 9);
                y += step;
            }
        }
        Backdrop::Light => {
            vertical(&mut pm, 0.0, fh, &[(0.0, 0xF4F6F8), (0.55, 0xDDE3EA), (1.0, 0xB9C2CC)]);
            for (cx, cy, r) in [(0.18, 0.28, 0.12), (0.72, 0.22, 0.09), (0.86, 0.62, 0.14)] {
                let mut pb = PathBuilder::new();
                pb.push_circle(cx * fw, cy * fh, r * fw);
                if let Some(path) = pb.finish() {
                    let mut paint = Paint::default();
                    paint.set_color_rgba8(255, 255, 255, 120);
                    paint.anti_alias = true;
                    pm.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
                }
            }
        }
        Backdrop::Scene => {
            let horizon = fh * 0.58;
            vertical(&mut pm, 0.0, horizon, &[(0.0, 0x3D5A80), (0.7, 0x98B4CC), (1.0, 0xD8E2E8)]);
            // Far ridge, near ridge.
            polygon(
                &mut pm,
                &[
                    (0.0, horizon),
                    (0.0, horizon - fh * 0.10),
                    (fw * 0.16, horizon - fh * 0.19),
                    (fw * 0.30, horizon - fh * 0.09),
                    (fw * 0.47, horizon - fh * 0.22),
                    (fw * 0.66, horizon - fh * 0.08),
                    (fw * 0.83, horizon - fh * 0.17),
                    (fw, horizon - fh * 0.07),
                    (fw, horizon),
                ],
                0x5F7286,
                255,
            );
            polygon(
                &mut pm,
                &[
                    (0.0, horizon),
                    (0.0, horizon - fh * 0.04),
                    (fw * 0.22, horizon - fh * 0.09),
                    (fw * 0.40, horizon - fh * 0.03),
                    (fw * 0.58, horizon - fh * 0.07),
                    (fw * 0.77, horizon - fh * 0.02),
                    (fw, horizon - fh * 0.06),
                    (fw, horizon),
                ],
                0x48545F,
                255,
            );
            vertical(&mut pm, horizon, fh, &[(0.0, 0x6B6450), (1.0, 0x2E2B23)]);
            // A building and a few crates to aim at.
            let bx = fw * 0.60;
            rect(&mut pm, bx, horizon - fh * 0.30, fw * 0.22, fh * 0.30, 0x39404A, 255);
            rect(&mut pm, bx + fw * 0.22, horizon - fh * 0.30, fw * 0.03, fh * 0.30, 0x2A3038, 255);
            for row in 0..3 {
                for col in 0..4 {
                    let wx = bx + fw * (0.018 + 0.05 * col as f32);
                    let wy = horizon - fh * (0.265 - 0.085 * row as f32);
                    rect(&mut pm, wx, wy, fw * 0.03, fh * 0.045, 0xC9D3DA, if (row + col) % 3 == 0 { 200 } else { 70 });
                }
            }
            let crate_at = |pm: &mut Pixmap, x: f32, s: f32| {
                let y = fh * 0.86 - s;
                rect(pm, x, y, s, s, 0x8A6A3E, 255);
                rect(pm, x + s * 0.08, y + s * 0.08, s * 0.84, s * 0.84, 0x9C7A48, 255);
                rect(pm, x, y + s * 0.45, s, s * 0.10, 0x6E532F, 255);
            };
            crate_at(&mut pm, fw * 0.08, fh * 0.22);
            crate_at(&mut pm, fw * 0.24, fh * 0.15);
            // A soft vignette keeps the eye in the middle.
            for i in 0..6 {
                let inset = i as f32 * 2.0 * scale;
                let a = 18 - i * 3;
                rect(&mut pm, inset, inset, fw - 2.0 * inset, 2.0 * scale, 0x000000, a as u8);
                rect(&mut pm, inset, fh - inset - 2.0 * scale, fw - 2.0 * inset, 2.0 * scale, 0x000000, a as u8);
            }
        }
    }
    pm
}
