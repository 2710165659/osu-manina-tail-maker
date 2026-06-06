mod cap;
mod body;
mod morph;

use cap::draw_cap;
use body::draw_body;
use morph::erode_mask;

use crate::config::{CapShape, RgbaColor, TailConfig};
use image::{ImageBuffer, Rgba, RgbaImage};
use tiny_skia::*;

pub const PREVIEW_MAX_ROWS: u32 = 500;

struct RenderLayout {
    w: u32,
    h: u32,
    left: u32,
    right: u32,
    cap_h: u32,
    echo_enabled: bool,
    echo_start: u32,
    echo_cap_end: u32,
    cap_start: u32,
    cap_end: u32,
    body_start: u32,
    body_h: u32,
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

    pixmap_to_image(pixmap)
}

fn draw_echo_layer(pixmap: &mut Pixmap, config: &TailConfig, l: &RenderLayout) {
    if !l.echo_enabled || l.cap_h == 0 { return; }
    let echo_cap_end = l.echo_cap_end.min(l.h);
    let right = l.right.min(l.w);
    if l.echo_start >= echo_cap_end || l.left >= right { return; }

    let mut echo_pixmap = Pixmap::new(l.w, l.h).unwrap();
    let echo_config = create_echo_config(config);
    draw_cap(&mut echo_pixmap.as_mut(), &echo_config, l.left, right, l.echo_start, echo_cap_end);

    let echo_color = config.effect.echo_color;
    let a = (echo_color.a as u32 * config.effect.echo_opacity as u32 * config.global_opacity as u32 / 65025) as u8;
    let fill_color = Color::from_rgba8(echo_color.r, echo_color.g, echo_color.b, a);
    let fill_h = l.cap_end.min(l.h).saturating_sub(echo_cap_end);
    if fill_h > 0 {
        let rect = Rect::from_xywh(l.left as f32, echo_cap_end as f32, (right - l.left) as f32, fill_h as f32).unwrap();
        let mut paint = Paint::default();
        paint.set_color(fill_color);
        echo_pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }

    // 合并 echo 层到主 pixmap（仅在主 pixmap 透明处绘制）
    let pw = l.w as usize;
    let y_end = l.cap_end.min(l.h) as usize;
    let data = pixmap.data_mut();
    let echo_data = echo_pixmap.data();
    for y in l.echo_start as usize..y_end {
        for x in l.left as usize..right as usize {
            let i = (y * pw + x) * 4;
            if echo_data[i + 3] > 0 && data[i + 3] == 0 {
                data[i..i + 4].copy_from_slice(&echo_data[i..i + 4]);
            }
        }
    }
}

fn draw_cap_layer(pixmap: &mut Pixmap, config: &TailConfig, l: &RenderLayout) {
    if l.cap_h > 0 {
        draw_cap(&mut pixmap.as_mut(), config, l.left, l.right, l.cap_start, l.cap_end);
    }
}

fn draw_body_layer(pixmap: &mut Pixmap, config: &TailConfig, l: &RenderLayout) {
    if l.body_h > 0 {
        draw_body(&mut pixmap.as_mut(), config, l.left, l.right, l.body_start, l.body_h);
    }
}

fn draw_border_layer(pixmap: &mut Pixmap, config: &TailConfig) {
    if !config.body.border_enabled || config.body.border_width == 0 { return; }

    let w = pixmap.width();
    let h = pixmap.height();
    let border_width = config.body.border_width;
    let (br, bg, bb) = border_rgb(config);

    let data = pixmap.data();
    let len = (w * h) as usize;
    let mask: Vec<u8> = (0..len).map(|i| if data[i * 4 + 3] > 0 { 255 } else { 0 }).collect();
    let orig_alpha: Vec<u8> = (0..len).map(|i| data[i * 4 + 3]).collect();
    let _ = data; // 释放不可变借用，后面需要 data_mut

    let mut eroded = mask.clone();
    erode_mask(&mut eroded, w, h, border_width);

    let border_opacity = config.body.border_opacity;
    let independent = config.body.border_opacity_independent;
    let data = pixmap.data_mut();
    for i in 0..len {
        if mask[i] == 0 || eroded[i] != 0 { continue; }
        let alpha = if independent { border_opacity } else { orig_alpha[i] };
        data[i * 4] = br;
        data[i * 4 + 1] = bg;
        data[i * 4 + 2] = bb;
        data[i * 4 + 3] = alpha;
    }
}

fn border_rgb(config: &TailConfig) -> (u8, u8, u8) {
    let bc: RgbaColor = if config.body.border_match_body {
        if config.body.independent_settings { config.body.color } else { config.global_color }
    } else {
        config.body.border_color
    };
    (bc.r, bc.g, bc.b)
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

fn create_echo_config(config: &TailConfig) -> TailConfig {
    let mut echo_config = config.clone();
    echo_config.cap.color = config.effect.echo_color;
    echo_config.cap.independent_settings = true;
    echo_config.cap.opacity = (config.effect.echo_opacity as u16 * config.global_opacity as u16 / 255) as u8;
    let cap_h = config.cap_height();
    if cap_h > 0 {
        let scale_factor = config.effect.echo_length as f64 / cap_h as f64;
        echo_config.cap.scale = ((config.cap.scale as f64 * scale_factor) as u32).max(1);
    }
    echo_config
}

pub fn render_preview(config: &TailConfig) -> RgbaImage {
    let full = render(config);
    let h = full.height().min(PREVIEW_MAX_ROWS);
    let w = full.width();
    ImageBuffer::from_fn(w, h, |x, y| *full.get_pixel(x, y))
}
