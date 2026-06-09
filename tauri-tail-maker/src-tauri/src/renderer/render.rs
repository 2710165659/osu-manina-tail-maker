use crate::renderer::cap::draw_cap_layer;
use crate::renderer::body::draw_body_layer;
use crate::renderer::border::draw_border_layer;
use crate::renderer::effects::{draw_echo_layer, draw_glow_layer};

use crate::config::{CapShape, TailConfig};
use image::{ImageBuffer, Rgba, RgbaImage};
use tiny_skia::*;

pub const PREVIEW_MAX_ROWS: u32 = 500;

pub(crate) struct RenderLayout {
    pub(crate) w: u32,
    pub(crate) h: u32,
    pub(crate) left: u32,
    pub(crate) right: u32,
    pub(crate) cap_h: u32,
    pub(crate) echo_enabled: bool,
    pub(crate) echo_start: u32,
    pub(crate) echo_cap_end: u32,
    pub(crate) cap_start: u32,
    pub(crate) cap_end: u32,
    pub(crate) body_start: u32,
    pub(crate) body_h: u32,
}

impl RenderLayout {
    fn new(config: &TailConfig) -> Self {
        let w = config.image.width;
        let h = config.image.height;
        let left = config.margin;
        let right = w.saturating_sub(config.margin);
        let cap_h = config.cap_height();
        let echo_enabled = config.effect.cap_echo_enabled && config.cap.shape != CapShape::Gradient;
        let echo_cap_h = if echo_enabled { cap_h } else { 0 };
        let echo_start = config.throw_length;
        let echo_cap_end = echo_start + echo_cap_h;
        let echo_rect_end = echo_cap_end + if echo_enabled { config.effect.echo_length } else { 0 };
        let cap_start = echo_rect_end;
        let cap_end = cap_start + cap_h;
        let body_start = cap_end;
        let body_h = config.body_height();
        Self { w, h, left, right, cap_h, echo_enabled, echo_start, echo_cap_end, cap_start, cap_end, body_start, body_h }
    }
}

pub fn render(config: &TailConfig) -> RgbaImage {
    let layout = RenderLayout::new(config);
    let mut pixmap = Pixmap::new(layout.w, layout.h).unwrap();

    draw_echo_layer(&mut pixmap, config, &layout);
    draw_cap_layer(&mut pixmap, config, &layout);
    draw_body_layer(&mut pixmap, config, &layout);
    draw_border_layer(&mut pixmap, config);
    draw_glow_layer(&mut pixmap, config);

    pixmap_to_image(pixmap)
}

/// tiny-skia 内部使用 premultiplied alpha，导出前需还原为 straight alpha
fn pixmap_to_image(pixmap: Pixmap) -> RgbaImage {
    let w = pixmap.width();
    let h = pixmap.height();
    let data = pixmap.data();
    ImageBuffer::from_fn(w, h, |x, y| {
        let i = ((y * w + x) * 4) as usize;
        let [r, g, b, a] = [data[i], data[i + 1], data[i + 2], data[i + 3]];
        if a == 0 {
            Rgba([0, 0, 0, 0])
        } else {
            // unpremultiply
            let scale = 255.0 / a as f32;
            Rgba([
                (r as f32 * scale).round().min(255.0) as u8,
                (g as f32 * scale).round().min(255.0) as u8,
                (b as f32 * scale).round().min(255.0) as u8,
                a,
            ])
        }
    })
}

pub fn render_preview(config: &TailConfig) -> RgbaImage {
    let full = render(config);
    let h = full.height().min(PREVIEW_MAX_ROWS);
    let w = full.width();
    ImageBuffer::from_fn(w, h, |x, y| *full.get_pixel(x, y))
}
