//! README images of every crosshair, drawn by the same renderer as the overlay.
//! `cargo run --release --features docs -- --docs docs/images`

use crate::backdrop::{self, Backdrop};
use crate::crosshair::{self, COLORS, Shape, Style};
use std::path::Path;
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, PixmapPaint, Transform};

/// The look used for the gallery, enlarged so details survive README scaling.
fn showcase(shape: Shape, color: [u8; 3], scale: f32) -> Style {
    let gap = (shape.uses_gap() && shape != Shape::XShape).then(|| (shape.auto_gap(14.0) * scale).round());
    Style {
        shape,
        color,
        size: 14.0 * scale,
        thickness: 2.0 * scale,
        gap,
        outline: true,
        outline_width: scale,
        outline_color: [0, 0, 0, 150],
        ..Style::default()
    }
}

fn rounded(pm: &Pixmap, radius: f32) -> Pixmap {
    let (w, h) = (pm.width() as f32, pm.height() as f32);
    let mut mask = Pixmap::new(pm.width(), pm.height()).expect("mask");
    let mut pb = PathBuilder::new();
    let r = radius;
    pb.move_to(r, 0.0);
    pb.line_to(w - r, 0.0);
    pb.quad_to(w, 0.0, w, r);
    pb.line_to(w, h - r);
    pb.quad_to(w, h, w - r, h);
    pb.line_to(r, h);
    pb.quad_to(0.0, h, 0.0, h - r);
    pb.line_to(0.0, r);
    pb.quad_to(0.0, 0.0, r, 0.0);
    pb.close();
    if let Some(path) = pb.finish() {
        let paint = Paint {
            shader: tiny_skia::Pattern::new(
                pm.as_ref(),
                tiny_skia::SpreadMode::Pad,
                tiny_skia::FilterQuality::Nearest,
                1.0,
                Transform::identity(),
            ),
            anti_alias: true,
            ..Paint::default()
        };
        mask.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
    }
    mask
}

fn tile(shape: Shape, color: [u8; 3], side: u32, scale: f32) -> Pixmap {
    let mut pm = backdrop::render(Backdrop::Dark, side, side, scale);
    let c = (side / 2) as i32;
    crosshair::draw_onto(&mut pm, c, c, &showcase(shape, color, scale));
    rounded(&pm, 18.0)
}

pub fn write_all(dir: &Path) {
    let shapes_dir = dir.join("crosshairs");
    let _ = std::fs::create_dir_all(&shapes_dir);
    let scale = 3.0;
    let side = 220;
    let palette: Vec<[u8; 3]> = COLORS.iter().take(8).map(|c| c.0).collect();

    // One image per shape, in the color it is shown with in the README.
    for (i, shape) in Shape::ALL.iter().enumerate() {
        let color = palette[i % palette.len()];
        let _ = tile(*shape, color, side, scale).save_png(shapes_dir.join(format!("{}.png", shape.id())));
    }

    // Overview sheet: 7 × 3.
    let (cols, gap, pad) = (7u32, 14u32, 18u32);
    let rows = (Shape::ALL.len() as u32).div_ceil(cols);
    let cell = 150u32;
    let w = pad * 2 + cols * cell + (cols - 1) * gap;
    let h = pad * 2 + rows * cell + (rows - 1) * gap;
    let mut sheet = Pixmap::new(w, h).expect("sheet");
    sheet.fill(tiny_skia::Color::from_rgba8(0x0B, 0x0E, 0x12, 255));
    for (i, shape) in Shape::ALL.iter().enumerate() {
        let (col, row) = (i as u32 % cols, i as u32 / cols);
        let t = tile(*shape, palette[i % palette.len()], cell, 2.0);
        let x = pad + col * (cell + gap);
        let y = pad + row * (cell + gap);
        sheet.draw_pixmap(x as i32, y as i32, t.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
    }
    let _ = sheet.save_png(dir.join("crosshairs.png"));

    // The same crosshair on the three preview backgrounds.
    let style = Style {
        shape: Shape::CrossGap,
        color: COLORS[0].0,
        size: 12.0 * 2.0,
        thickness: 2.0 * 2.0,
        gap: Some(8.0),
        dot: true,
        outline: true,
        outline_width: 2.0,
        outline_color: [0, 0, 0, 160],
        ..Style::default()
    };
    let (bw, bh) = (400u32, 240u32);
    let mut strip = Pixmap::new(bw * 3 + 2 * 12, bh).expect("strip");
    for (i, kind) in [Backdrop::Scene, Backdrop::Dark, Backdrop::Light].iter().enumerate() {
        let mut pm = backdrop::render(*kind, bw, bh, 2.0);
        crosshair::draw_onto(&mut pm, (bw / 2) as i32, (bh / 2) as i32, &style);
        let pm = rounded(&pm, 16.0);
        strip.draw_pixmap(
            (i as u32 * (bw + 12)) as i32,
            0,
            pm.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
    }
    let _ = strip.save_png(dir.join("backgrounds.png"));
}
