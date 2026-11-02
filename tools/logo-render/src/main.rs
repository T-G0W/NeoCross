//! Renders the NeoCross logo SVGs into the PNG, ICO and ICNS files in `assets/`.
//! Run from the repository root: `cargo run --release --manifest-path tools/logo-render/Cargo.toml`

use resvg::{tiny_skia, usvg};
use std::path::Path;

fn render(svg: &Path, width: u32) -> tiny_skia::Pixmap {
    let mut opt = usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();
    opt.resources_dir = svg.parent().map(Path::to_path_buf);
    let data = std::fs::read(svg).expect("read SVG");
    let tree = usvg::Tree::from_data(&data, &opt).expect("parse SVG");
    let size = tree.size();
    let scale = width as f32 / size.width();
    let height = (size.height() * scale).round() as u32;
    let mut pixmap = tiny_skia::Pixmap::new(width, height).expect("pixmap");
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    pixmap
}

fn png(svg: &Path, size: u32) -> Vec<u8> {
    render(svg, size).encode_png().expect("PNG")
}

fn write_ico(path: &Path, images: &[(u32, Vec<u8>)]) {
    let mut out = Vec::new();
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&(images.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * images.len() as u32;
    for (size, png) in images {
        let dim = if *size >= 256 { 0u8 } else { *size as u8 };
        out.extend_from_slice(&[dim, dim, 0, 0]);
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&32u16.to_le_bytes());
        out.extend_from_slice(&(png.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += png.len() as u32;
    }
    for (_, png) in images {
        out.extend_from_slice(png);
    }
    std::fs::write(path, out).expect("write ICO");
}

/// Apple icon file with PNG payloads.
fn write_icns(path: &Path, entries: &[(&[u8; 4], Vec<u8>)]) {
    let mut body = Vec::new();
    for (kind, png) in entries {
        body.extend_from_slice(*kind);
        body.extend_from_slice(&(png.len() as u32 + 8).to_be_bytes());
        body.extend_from_slice(png);
    }
    let mut out = Vec::new();
    out.extend_from_slice(b"icns");
    out.extend_from_slice(&(body.len() as u32 + 8).to_be_bytes());
    out.extend_from_slice(&body);
    std::fs::write(path, out).expect("write ICNS");
}

fn main() {
    let assets = Path::new("assets");
    let full = assets.join("logo.svg");
    let small = assets.join("logo-small.svg");
    // Below 64 px the simplified mark stays crisp; above that the full logo.
    let pick = |size: u32| if size <= 48 { &small } else { &full };

    let ico: Vec<(u32, Vec<u8>)> = [16u32, 20, 24, 32, 40, 48, 64, 96, 128, 256].iter().map(|s| (*s, png(pick(*s), *s))).collect();
    write_ico(&assets.join("icon.ico"), &ico);

    write_icns(
        &assets.join("icon.icns"),
        &[
            (b"icp4", png(&small, 16)),
            (b"icp5", png(&small, 32)),
            (b"ic11", png(&small, 32)),
            (b"ic12", png(&full, 64)),
            (b"ic07", png(&full, 128)),
            (b"ic13", png(&full, 256)),
            (b"ic08", png(&full, 256)),
            (b"ic14", png(&full, 512)),
            (b"ic09", png(&full, 512)),
            (b"ic10", png(&full, 1024)),
        ],
    );

    for size in [512u32, 256, 128] {
        render(&full, size).save_png(assets.join(format!("logo-{size}.png"))).expect("save PNG");
    }
    render(&small, 64).save_png(assets.join("logo-64.png")).expect("save PNG");
    render(&assets.join("wordmark.svg"), 1200).save_png(assets.join("wordmark.png")).expect("save PNG");
    println!("Logo assets written to {}", assets.display());
}
