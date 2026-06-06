use crate::renderer::cap::draw_cap_layer;
use crate::renderer::body::draw_body_layer;
use crate::renderer::effects::{draw_echo_layer, draw_border_layer, draw_glow_layer};
use crate::renderer::gpu::{detect_gpu_backend, create_surface};

use crate::config::{CapShape, TailConfig};
use image::{ImageBuffer, Rgba, RgbaImage};
use skia_safe::Surface;

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
    let backend = detect_gpu_backend();
    let mut surface = create_surface(layout.w as i32, layout.h as i32, &backend)
        .expect("Failed to create surface");

    let canvas = surface.canvas();
    canvas.clear(skia_safe::Color::TRANSPARENT);

    draw_echo_layer(canvas, config, &layout);
    draw_cap_layer(canvas, config, &layout);
    draw_body_layer(canvas, config, &layout);
    draw_border_layer(&mut surface, config);
    draw_glow_layer(&mut surface, config);

    surface_to_image(&mut surface)
}

fn surface_to_image(surface: &mut Surface) -> RgbaImage {
    let image = surface.image_snapshot();
    let (w, h) = (image.width() as u32, image.height() as u32);

    let info = skia_safe::ImageInfo::new(
        (w as i32, h as i32),
        skia_safe::ColorType::RGBA8888,
        skia_safe::AlphaType::Premul,
        None,
    );

    let mut pixels = vec![0u8; (w * h * 4) as usize];
    image.read_pixels(&info, &mut pixels, (w * 4) as usize, (0, 0), skia_safe::image::CachingHint::Allow);

    ImageBuffer::from_fn(w, h, |x, y| {
        let i = ((y * w + x) * 4) as usize;
        let r = pixels[i];
        let g = pixels[i + 1];
        let b = pixels[i + 2];
        let a = pixels[i + 3];

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
