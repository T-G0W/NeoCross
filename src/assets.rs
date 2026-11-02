//! Images compiled into the executable.

pub const LOGO_64: &[u8] = include_bytes!("../assets/logo-64.png");
pub const LOGO_256: &[u8] = include_bytes!("../assets/logo-256.png");

/// Decodes a PNG into straight (not premultiplied) RGBA.
pub fn rgba(png: &[u8]) -> (Vec<u8>, u32, u32) {
    match tiny_skia::Pixmap::decode_png(png) {
        Ok(pm) => {
            let (w, h) = (pm.width(), pm.height());
            let data = pm
                .pixels()
                .iter()
                .flat_map(|p| {
                    let c = p.demultiply();
                    [c.red(), c.green(), c.blue(), c.alpha()]
                })
                .collect();
            (data, w, h)
        }
        Err(_) => (vec![0; 4], 1, 1),
    }
}
