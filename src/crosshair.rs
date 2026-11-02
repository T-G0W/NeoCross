//! Crosshair model and renderer.
//!
//! The overlay, the menu previews and the README images all draw through [`render`], so what you
//! see in the menu is pixel for pixel what ends up on screen.

use crate::t;
use serde::{Deserialize, Serialize};
use std::f32::consts::{FRAC_1_SQRT_2, SQRT_2};
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, PixmapPaint, Stroke, Transform};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)] // X-Shape and T-Shape are the names users know
pub enum Shape {
    Plus,
    Dot,
    Circle,
    CircleDot,
    CrossGap,
    #[default]
    XShape,
    Triangle,
    TShape,
    Chevron,
    Corners,
    Square,
    Diamond,
    Hexagon,
    Scope,
    Reticle,
    Segments,
    MilDot,
    Star,
    TriDot,
    Brackets,
    Halo,
}

impl Shape {
    pub const ALL: [Shape; 21] = [
        Shape::Plus,
        Shape::Dot,
        Shape::Circle,
        Shape::CircleDot,
        Shape::CrossGap,
        Shape::XShape,
        Shape::Triangle,
        Shape::TShape,
        Shape::Chevron,
        Shape::Corners,
        Shape::Square,
        Shape::Diamond,
        Shape::Hexagon,
        Shape::Scope,
        Shape::Reticle,
        Shape::Segments,
        Shape::MilDot,
        Shape::Star,
        Shape::TriDot,
        Shape::Brackets,
        Shape::Halo,
    ];

    /// The seven shapes of NeoCross 1.x, in their original order (indices of the old settings file).
    pub const CLASSIC: [Shape; 7] =
        [Shape::Plus, Shape::Dot, Shape::Circle, Shape::CircleDot, Shape::CrossGap, Shape::XShape, Shape::Triangle];

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|s| *s == self).unwrap_or(0)
    }

    pub fn cycle(self, step: isize) -> Shape {
        let n = Self::ALL.len() as isize;
        Self::ALL[(self.index() as isize + step).rem_euclid(n) as usize]
    }

    pub fn id(self) -> &'static str {
        match self {
            Shape::Plus => "plus",
            Shape::Dot => "dot",
            Shape::Circle => "circle",
            Shape::CircleDot => "circle_dot",
            Shape::CrossGap => "cross_gap",
            Shape::XShape => "x_shape",
            Shape::Triangle => "triangle",
            Shape::TShape => "t_shape",
            Shape::Chevron => "chevron",
            Shape::Corners => "corners",
            Shape::Square => "square",
            Shape::Diamond => "diamond",
            Shape::Hexagon => "hexagon",
            Shape::Scope => "scope",
            Shape::Reticle => "reticle",
            Shape::Segments => "segments",
            Shape::MilDot => "mil_dot",
            Shape::Star => "star",
            Shape::TriDot => "tri_dot",
            Shape::Brackets => "brackets",
            Shape::Halo => "halo",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Shape::Plus => "Plus",
            Shape::Dot => t!("Punkt", "Dot"),
            Shape::Circle => t!("Kreis", "Circle"),
            Shape::CircleDot => t!("Kreis + Punkt", "Circle + Dot"),
            Shape::CrossGap => t!("Kreuz mit Lücke", "Cross Gap"),
            Shape::XShape => t!("X-Form", "X-Shape"),
            Shape::Triangle => t!("Dreieck", "Triangle"),
            Shape::TShape => t!("T-Form", "T-Shape"),
            Shape::Chevron => "Chevron",
            Shape::Corners => t!("Ecken", "Corners"),
            Shape::Square => t!("Quadrat", "Square"),
            Shape::Diamond => t!("Raute", "Diamond"),
            Shape::Hexagon => t!("Sechseck", "Hexagon"),
            Shape::Scope => t!("Zielfernrohr", "Scope"),
            Shape::Reticle => t!("Absehen", "Reticle"),
            Shape::Segments => t!("Segmente", "Segments"),
            Shape::MilDot => "Mil-Dot",
            Shape::Star => t!("Stern", "Star"),
            Shape::TriDot => t!("Drei Punkte", "Tri-Dot"),
            Shape::Brackets => t!("Klammern", "Brackets"),
            Shape::Halo => "Halo",
        }
    }

    /// Whether the gap slider changes this shape.
    pub fn uses_gap(self) -> bool {
        matches!(
            self,
            Shape::CrossGap
                | Shape::XShape
                | Shape::TShape
                | Shape::Chevron
                | Shape::Scope
                | Shape::MilDot
                | Shape::Star
        )
    }

    /// Gap used when "automatic" is selected; matches NeoCross 1.x where the shape existed there.
    pub fn auto_gap(self, size: f32) -> f32 {
        match self {
            Shape::CrossGap | Shape::TShape | Shape::Scope | Shape::MilDot => (size / 3.0).floor().max(3.0),
            Shape::Star => (size / 4.0).floor().max(2.0),
            _ => 0.0,
        }
    }

    fn has_own_dot(self) -> bool {
        matches!(self, Shape::Dot | Shape::CircleDot | Shape::Reticle)
    }
}

/// Color presets: the eight colors of NeoCross 1.x first, in their original order.
pub const COLORS: [([u8; 3], &str, &str); 10] = [
    ([0x00, 0xFF, 0x88], "Neongrün", "Neon Green"),
    ([0xFF, 0x44, 0x44], "Rot", "Red"),
    ([0x44, 0xAA, 0xFF], "Blau", "Blue"),
    ([0xFF, 0xD7, 0x00], "Gold", "Gold"),
    ([0xFF, 0x44, 0xFF], "Magenta", "Magenta"),
    ([0xFF, 0xFF, 0xFF], "Weiß", "White"),
    ([0xFF, 0x88, 0x00], "Orange", "Orange"),
    ([0x00, 0xFF, 0xFF], "Cyan", "Cyan"),
    ([0xB6, 0xFF, 0x00], "Limette", "Lime"),
    ([0x00, 0x00, 0x00], "Schwarz", "Black"),
];

pub fn color_name(rgb: [u8; 3]) -> String {
    match COLORS.iter().find(|c| c.0 == rgb) {
        Some((_, de, en)) => t!(*de, *en).to_string(),
        None => format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]),
    }
}

/// Size presets of NeoCross 1.x (arm length in pixels).
pub const SIZES: [(f32, &str); 6] = [(5.0, "XS"), (8.0, "S"), (12.0, "M"), (16.0, "L"), (22.0, "XL"), (30.0, "XXL")];

pub const SIZE_RANGE: std::ops::RangeInclusive<f32> = 2.0..=64.0;
pub const THICKNESS_RANGE: std::ops::RangeInclusive<f32> = 1.0..=8.0;
pub const GAP_RANGE: std::ops::RangeInclusive<f32> = 0.0..=24.0;
pub const OUTLINE_RANGE: std::ops::RangeInclusive<f32> = 1.0..=3.0;
pub const OPACITY_RANGE: std::ops::RangeInclusive<f32> = 0.1..=1.0;
pub const ROTATION_RANGE: std::ops::RangeInclusive<f32> = 0.0..=90.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Style {
    pub shape: Shape,
    pub color: [u8; 3],
    /// 0.1 – 1.0
    pub opacity: f32,
    /// Arm length / radius in physical pixels.
    pub size: f32,
    pub thickness: f32,
    /// `None` = automatic per shape.
    pub gap: Option<f32>,
    pub outline: bool,
    pub outline_width: f32,
    pub outline_color: [u8; 4],
    /// Adds a center dot to shapes that don't have one.
    pub dot: bool,
    /// Degrees, clockwise.
    pub rotation: f32,
}

impl Default for Style {
    /// The first-run look of NeoCross 1.x: X-Shape, Gold, size S, with a soft dark outline.
    fn default() -> Self {
        Style {
            shape: Shape::XShape,
            color: COLORS[3].0,
            opacity: 1.0,
            size: 8.0,
            thickness: 2.0,
            gap: None,
            outline: true,
            outline_width: 1.0,
            outline_color: [0, 0, 0, 110],
            dot: false,
            rotation: 0.0,
        }
    }
}

fn clamp(v: f32, r: &std::ops::RangeInclusive<f32>, fallback: f32) -> f32 {
    if v.is_finite() { v.clamp(*r.start(), *r.end()) } else { fallback }
}

impl Style {
    /// Brings values from a hand-edited or older settings file back into range.
    pub fn sanitize(&mut self) {
        let d = Style::default();
        self.opacity = clamp(self.opacity, &OPACITY_RANGE, d.opacity);
        self.size = clamp(self.size, &SIZE_RANGE, d.size).round();
        self.thickness = clamp(self.thickness, &THICKNESS_RANGE, d.thickness).round();
        self.gap = self.gap.map(|g| clamp(g, &GAP_RANGE, 0.0).round());
        self.outline_width = clamp(self.outline_width, &OUTLINE_RANGE, d.outline_width).round();
        self.rotation = clamp(self.rotation, &ROTATION_RANGE, 0.0).round();
    }

    pub fn gap_px(&self) -> f32 {
        self.gap.unwrap_or_else(|| self.shape.auto_gap(self.size))
    }

    /// Next/previous color preset; a custom color continues with the first preset.
    pub fn cycle_color(&mut self, step: isize) {
        let n = COLORS.len() as isize;
        self.color = match COLORS.iter().position(|c| c.0 == self.color) {
            Some(i) => COLORS[(i as isize + step).rem_euclid(n) as usize].0,
            None => COLORS[0].0,
        };
    }

    /// Next/previous size preset (XS … XXL), wrapping around like NeoCross 1.x.
    pub fn cycle_size(&mut self, step: isize) {
        let next = if step >= 0 {
            SIZES.iter().find(|s| s.0 > self.size + 0.5).unwrap_or(&SIZES[0])
        } else {
            SIZES.iter().rev().find(|s| s.0 < self.size - 0.5).unwrap_or(&SIZES[SIZES.len() - 1])
        };
        self.size = next.0;
    }
}

type P = [f32; 2];

enum Prim {
    Line(P, P),
    Path(Vec<P>, bool),
    Ring(f32),
    /// Radius, center angle and sweep in degrees.
    Arc(f32, f32, f32),
    Disc(P, f32),
}

const ORIGIN: P = [0.0, 0.0];
const CARDINAL: [P; 4] = [[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]];
const DIAGONAL: [P; 4] = [[1.0, 1.0], [-1.0, -1.0], [1.0, -1.0], [-1.0, 1.0]];

fn dot_radius(t: f32) -> f32 {
    (t * 0.5 + 0.75).max(1.25)
}

fn primitives(style: &Style) -> Vec<Prim> {
    let r = style.size.max(1.0);
    let t = style.thickness.max(0.5);
    let g = style.gap_px().max(0.0);
    // With an odd line width the aim point is the center of a pixel. Cardinal arms then start at
    // that pixel's edges, so both halves of the cross stay exactly the same length.
    let odd = (t.round() as i32) % 2 == 1 && style.rotation.rem_euclid(90.0) == 0.0;
    let h = if odd { 0.5 } else { 0.0 };
    let mut v = Vec::with_capacity(16);

    let arms = |v: &mut Vec<Prim>, from: f32, to: f32, dirs: &[P]| {
        let a = if from > 0.0 { from + h } else { -h };
        let b = to.max(from + 1.0) + h;
        for d in dirs {
            v.push(Prim::Line([d[0] * a, d[1] * a], [d[0] * b, d[1] * b]));
        }
    };
    let diagonals = |v: &mut Vec<Prim>, from: f32, to: f32| {
        for d in DIAGONAL {
            let (x, y) = (d[0] * FRAC_1_SQRT_2, d[1] * FRAC_1_SQRT_2);
            v.push(Prim::Line([x * from, y * from], [x * to, y * to]));
        }
    };

    match style.shape {
        Shape::Plus => arms(&mut v, 0.0, r, &CARDINAL),
        Shape::Dot => v.push(Prim::Disc(ORIGIN, (r / 3.0).max(2.0))),
        Shape::Circle => v.push(Prim::Ring(r)),
        Shape::CircleDot => {
            v.push(Prim::Ring(r));
            v.push(Prim::Disc(ORIGIN, (r / 4.0).max(2.0)));
        }
        Shape::CrossGap => arms(&mut v, g, r.max(g + 2.0), &CARDINAL),
        Shape::XShape => {
            if g <= 0.0 {
                v.push(Prim::Line([-r, -r], [r, r]));
                v.push(Prim::Line([r, -r], [-r, r]));
            } else {
                diagonals(&mut v, g, (r * SQRT_2).max(g + 2.0));
            }
        }
        Shape::Triangle => v.push(Prim::Path(vec![[0.0, -r], [-0.8 * r, 0.6 * r], [0.8 * r, 0.6 * r]], true)),
        Shape::TShape => arms(&mut v, g, r.max(g + 2.0), &CARDINAL[..3]),
        Shape::Chevron => {
            let k = r * 0.8;
            v.push(Prim::Path(vec![[-k, k + g], [0.0, g], [k, k + g]], false));
        }
        Shape::Corners => {
            let l = (r * 0.45).max(2.0);
            for d in DIAGONAL {
                let (sx, sy) = (d[0], d[1]);
                v.push(Prim::Path(vec![[sx * r, sy * (r - l)], [sx * r, sy * r], [sx * (r - l), sy * r]], false));
            }
        }
        Shape::Square => {
            let k = r * 0.8;
            v.push(Prim::Path(vec![[-k, -k], [k, -k], [k, k], [-k, k]], true));
        }
        Shape::Diamond => v.push(Prim::Path(vec![[0.0, -r], [r, 0.0], [0.0, r], [-r, 0.0]], true)),
        Shape::Hexagon => {
            let pts = (0..6)
                .map(|k| {
                    let a = (-90.0 + 60.0 * k as f32).to_radians();
                    [r * a.cos(), r * a.sin()]
                })
                .collect();
            v.push(Prim::Path(pts, true));
        }
        Shape::Scope => {
            v.push(Prim::Ring(r * 0.7));
            arms(&mut v, g, r.max(g + 2.0), &CARDINAL);
        }
        Shape::Reticle => {
            v.push(Prim::Ring(r));
            arms(&mut v, (r * 0.5).round(), r, &CARDINAL);
            v.push(Prim::Disc(ORIGIN, dot_radius(t)));
        }
        Shape::Segments => {
            for k in 0..4 {
                v.push(Prim::Arc(r, 45.0 + 90.0 * k as f32, 54.0));
            }
        }
        Shape::MilDot => {
            let end = r.max(g + 4.0);
            arms(&mut v, g, end, &CARDINAL);
            let dr = (t * 0.9).max(1.5);
            for d in CARDINAL {
                for f in [0.55, 1.0] {
                    let dist = g + (end - g) * f + h;
                    v.push(Prim::Disc([d[0] * dist, d[1] * dist], dr));
                }
            }
        }
        Shape::Star => {
            arms(&mut v, g, r.max(g + 2.0), &CARDINAL);
            diagonals(&mut v, g, (r * 0.75).max(g + 2.0));
        }
        Shape::TriDot => {
            let k = r * 0.75;
            let dr = (t * 0.9).max(1.5);
            for p in [[-k, 0.0], [k, 0.0], [0.0, k]] {
                v.push(Prim::Disc(p, dr));
            }
        }
        Shape::Brackets => {
            let w = r * 0.45;
            let hh = r * 0.75;
            for s in [-1.0, 1.0] {
                v.push(Prim::Path(vec![[s * (r - w), -hh], [s * r, -hh], [s * r, hh], [s * (r - w), hh]], false));
            }
        }
        Shape::Halo => {
            let dr = (t * 0.75).max(1.25);
            for k in 0..8 {
                let a = (45.0 * k as f32).to_radians();
                v.push(Prim::Disc([r * a.cos(), r * a.sin()], dr));
            }
        }
    }
    if style.dot && !style.shape.has_own_dot() {
        v.push(Prim::Disc(ORIGIN, dot_radius(t)));
    }
    v
}

fn len(p: P) -> f32 {
    (p[0] * p[0] + p[1] * p[1]).sqrt()
}

/// Largest distance from the aim point that any pixel can reach.
fn extent(prims: &[Prim], style: &Style) -> f32 {
    let mut m: f32 = 1.0;
    for p in prims {
        m = m.max(match p {
            Prim::Line(a, b) => len(*a).max(len(*b)),
            Prim::Path(pts, _) => pts.iter().map(|p| len(*p)).fold(0.0, f32::max),
            Prim::Ring(r) | Prim::Arc(r, _, _) => *r,
            Prim::Disc(c, r) => len(*c) + r,
        });
    }
    // Miter joins reach up to twice the line width past a corner.
    m + style.thickness * 2.0 + if style.outline { style.outline_width } else { 0.0 } + 2.0
}

/// Moves both ends of a segment outwards by `by` pixels.
fn extend(a: P, b: P, by: f32) -> (P, P) {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l = (dx * dx + dy * dy).sqrt();
    if by == 0.0 || l == 0.0 {
        return (a, b);
    }
    let (ux, uy) = (dx / l * by, dy / l * by);
    ([a[0] - ux, a[1] - uy], [b[0] + ux, b[1] + uy])
}

fn draw_pass(pm: &mut Pixmap, prims: &[Prim], ts: Transform, width: f32, grow: f32, rgba: [u8; 4]) {
    let mut paint = Paint::default();
    paint.set_color_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
    paint.anti_alias = true;
    let stroke = Stroke { width, miter_limit: 4.0, line_cap: LineCap::Butt, line_join: LineJoin::Miter, dash: None };
    for p in prims {
        let mut pb = PathBuilder::new();
        let fill = match p {
            Prim::Line(a, b) => {
                let (a, b) = extend(*a, *b, grow);
                pb.move_to(a[0], a[1]);
                pb.line_to(b[0], b[1]);
                false
            }
            Prim::Path(pts, closed) => {
                let mut pts = pts.clone();
                if !closed && pts.len() >= 2 {
                    let n = pts.len();
                    pts[0] = extend(pts[0], pts[1], grow).0;
                    pts[n - 1] = extend(pts[n - 2], pts[n - 1], grow).1;
                }
                pb.move_to(pts[0][0], pts[0][1]);
                for q in &pts[1..] {
                    pb.line_to(q[0], q[1]);
                }
                if *closed {
                    pb.close();
                }
                false
            }
            Prim::Ring(r) => {
                pb.push_circle(0.0, 0.0, *r);
                false
            }
            Prim::Arc(r, mid, sweep) => {
                let extra = (grow / r).to_degrees();
                let (a0, a1) = (mid - sweep / 2.0 - extra, mid + sweep / 2.0 + extra);
                let steps = ((a1 - a0) / 3.0).ceil().max(2.0) as usize;
                for i in 0..=steps {
                    let a = (a0 + (a1 - a0) * i as f32 / steps as f32).to_radians();
                    let (x, y) = (r * a.cos(), r * a.sin());
                    if i == 0 { pb.move_to(x, y) } else { pb.line_to(x, y) }
                }
                false
            }
            Prim::Disc(c, r) => {
                pb.push_circle(c[0], c[1], r + grow);
                true
            }
        };
        let Some(path) = pb.finish() else { continue };
        if fill {
            pm.fill_path(&path, &paint, FillRule::Winding, ts, None);
        } else {
            pm.stroke_path(&path, &paint, &stroke, ts, None);
        }
    }
}

/// Renders a crosshair onto a transparent, square canvas with an even side length.
///
/// The aim point is the canvas center: for even line widths it lies on the pixel boundary at
/// `(w/2, h/2)`, for odd ones in the middle of pixel `(w/2, h/2)`. Placing the canvas at
/// `screen_center - w/2` therefore always aims at the true screen center.
pub fn render(style: &Style) -> Pixmap {
    let prims = primitives(style);
    let half = extent(&prims, style).ceil() as u32 + 1;
    let side = half * 2;
    let mut pm = Pixmap::new(side, side).expect("canvas");
    let odd = (style.thickness.round() as i32) % 2 == 1 && style.rotation.rem_euclid(90.0) == 0.0;
    let c = half as f32 + if odd { 0.5 } else { 0.0 };
    let ts = Transform::from_translate(c, c).pre_rotate(style.rotation);
    let t = style.thickness;
    if style.outline {
        let ow = style.outline_width;
        draw_pass(&mut pm, &prims, ts, t + 2.0 * ow, ow, style.outline_color);
    }
    let [r, g, b] = style.color;
    draw_pass(&mut pm, &prims, ts, t, 0.0, [r, g, b, 255]);
    if style.opacity < 1.0 {
        let o = style.opacity.clamp(0.0, 1.0);
        for px in pm.data_mut() {
            *px = (*px as f32 * o).round() as u8;
        }
    }
    pm
}

/// Draws a crosshair centered on `(cx, cy)` of an existing image.
#[cfg_attr(not(feature = "docs"), allow(dead_code))]
pub fn draw_onto(target: &mut Pixmap, cx: i32, cy: i32, style: &Style) {
    let pm = render(style);
    let half = (pm.width() / 2) as i32;
    target.draw_pixmap(cx - half, cy - half, pm.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
}

/// A compact version of the style for the shape buttons in the menu.
pub fn thumbnail_style(base: &Style, shape: Shape) -> Style {
    Style {
        shape,
        size: 11.0,
        thickness: base.thickness.clamp(1.0, 2.0),
        gap: None,
        opacity: 1.0,
        rotation: 0.0,
        outline_width: 1.0,
        ..base.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shape_renders_centered_and_symmetric() {
        for shape in Shape::ALL {
            for thickness in [1.0, 2.0, 3.0] {
                let style = Style { shape, thickness, outline: false, ..Style::default() };
                let pm = render(&style);
                assert_eq!(pm.width() % 2, 0, "{shape:?}");
                assert!(pm.data().iter().any(|b| *b != 0), "{shape:?} draws nothing");
                // Nothing may touch the border: the canvas must be large enough.
                let w = pm.width() as usize;
                let alpha = |x: usize, y: usize| pm.data()[(y * w + x) * 4 + 3];
                for i in 0..w {
                    assert_eq!(alpha(i, 0), 0, "{shape:?} clipped at top");
                    assert_eq!(alpha(0, i), 0, "{shape:?} clipped at left");
                    assert_eq!(alpha(i, w - 1), 0, "{shape:?} clipped at bottom");
                    assert_eq!(alpha(w - 1, i), 0, "{shape:?} clipped at right");
                }
            }
        }
    }

    #[test]
    fn plus_is_pixel_exact() {
        for t in [1.0, 2.0, 3.0, 4.0] {
            let style = Style { shape: Shape::Plus, size: 10.0, thickness: t, outline: false, ..Style::default() };
            let pm = render(&style);
            let w = pm.width() as usize;
            let row: Vec<u8> = (0..w).map(|x| pm.data()[((w / 2) * w + x) * 4 + 3]).collect();
            // Fully opaque or fully transparent, nothing in between: crisp edges.
            assert!(row.iter().all(|a| *a == 0 || *a == 255), "t={t}: {row:?}");
            let lit: Vec<usize> = (0..w).filter(|x| row[*x] == 255).collect();
            let (first, last) = (lit[0], *lit.last().unwrap());
            let center = if t as i32 % 2 == 1 { w / 2 } else { w / 2 - 1 };
            assert_eq!(center - first, last - center - if t as i32 % 2 == 1 { 0 } else { 1 }, "t={t} asymmetric");
        }
    }

    #[test]
    fn cycling_wraps_like_v1() {
        let mut s = Style { size: 30.0, ..Style::default() };
        s.cycle_size(1);
        assert_eq!(s.size, 5.0);
        s.cycle_size(-1);
        assert_eq!(s.size, 30.0);
        s.color = COLORS[COLORS.len() - 1].0;
        s.cycle_color(1);
        assert_eq!(s.color, COLORS[0].0);
        assert_eq!(Shape::Halo.cycle(1), Shape::Plus);
        assert_eq!(Shape::Plus.cycle(-1), Shape::Halo);
    }
}
