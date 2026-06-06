use crate::config::{RgbaColor, TailConfig};
use tiny_skia::*;

pub fn draw_body(
    pixmap: &mut PixmapMut,
    config: &TailConfig,
    left: u32,
    right: u32,
    y_start: u32,
    body_height: u32,
) {
    if body_height == 0 || left >= right { return; }
    let (fc, fo) = if config.body.independent_settings {
        (config.body.color, config.body.opacity)
    } else {
        (config.global_color, config.global_opacity)
    };
    let rect = Rect::from_xywh(left as f32, y_start as f32, (right - left) as f32, body_height as f32).unwrap();
    let mut paint = Paint::default();
    paint.set_color(solid_color(fc, fo));
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
}

fn solid_color(c: RgbaColor, opacity: u8) -> Color {
    let a = (c.a as u16 * opacity as u16 / 255) as u8;
    Color::from_rgba8(c.r, c.g, c.b, a)
}
