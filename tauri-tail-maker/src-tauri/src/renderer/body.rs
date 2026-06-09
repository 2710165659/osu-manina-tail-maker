use crate::config::{RgbaColor, TailConfig};
use tiny_skia::*;

use crate::renderer::render::RenderLayout;

pub fn draw_body_layer(pixmap: &mut Pixmap, config: &TailConfig, l: &RenderLayout) {
    if l.body_h == 0 { return; }
    let (left, right, y_start, body_height) = (l.left, l.right, l.body_start, l.body_h);
    if left >= right { return; }
    let (fc, fo) = if config.body.independent_settings {
        (config.body.color, config.body.opacity)
    } else {
        (config.global_color, config.global_opacity)
    };
    let rect = Rect::from_xywh(left as f32, y_start as f32, (right - left) as f32, body_height as f32).unwrap();
    let mut paint = Paint::default();
    paint.blend_mode = BlendMode::Source;
    paint.set_color(solid_color(fc, fo));
    pixmap.as_mut().fill_rect(rect, &paint, Transform::identity(), None);
}

fn solid_color(c: RgbaColor, opacity: u8) -> Color {
    let a = (c.a as u16 * opacity as u16 / 255) as u8;
    Color::from_rgba8(c.r, c.g, c.b, a)
}
