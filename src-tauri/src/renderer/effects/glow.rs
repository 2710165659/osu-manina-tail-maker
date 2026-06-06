//! 外发光效果 - 基于 Alpha 通道的膨胀+模糊
use crate::config::TailConfig;
use skia_safe::{Surface, Paint, Color, image_filters, BlendMode, Matrix, ImageInfo, ColorType, AlphaType};

pub fn draw_glow_layer(surface: &mut Surface, config: &TailConfig) {
    if !config.effect.glow_enabled { return; }

    let spread = config.effect.glow_spread as f32;
    let size = config.effect.glow_size as f32;
    if spread == 0.0 && size == 0.0 { return; }

    let snapshot = surface.image_snapshot();
    let (w, h) = (snapshot.width(), snapshot.height());

    let alpha_mask = extract_alpha_mask(&snapshot, w, h);

    let mut glow_filter = None;

    if spread > 0.0 {
        glow_filter = image_filters::dilate((spread, spread), glow_filter, None);
    }

    if size > 0.0 {
        glow_filter = image_filters::blur((size, size), None, glow_filter, None);
    }

    if glow_filter.is_none() { return; }

    let mut glow_surface = skia_safe::surfaces::raster_n32_premul((w, h))
        .expect("Failed to create glow surface");
    let glow_canvas = glow_surface.canvas();
    glow_canvas.clear(Color::TRANSPARENT);

    let mut paint_glow = Paint::default();
    paint_glow.set_image_filter(glow_filter);

    let dx = config.effect.glow_dx as f32;
    let dy = config.effect.glow_dy as f32;
    let matrix = Matrix::translate((dx, dy));
    glow_canvas.concat(&matrix);

    glow_canvas.draw_image(&alpha_mask, (-dx, -dy), Some(&paint_glow));

    let glow_mask = glow_surface.image_snapshot();

    let mut glow_ring_surface = skia_safe::surfaces::raster_n32_premul((w, h))
        .expect("Failed to create glow ring surface");
    let glow_ring_canvas = glow_ring_surface.canvas();
    glow_ring_canvas.clear(Color::TRANSPARENT);
    glow_ring_canvas.draw_image(&glow_mask, (0, 0), None);

    let mut paint_mask_out = Paint::default();
    paint_mask_out.set_blend_mode(BlendMode::DstOut);
    glow_ring_canvas.draw_image(&alpha_mask, (0, 0), Some(&paint_mask_out));

    let glow_ring = glow_ring_surface.image_snapshot();

    let gc = config.effect.glow_color;
    let glow_opacity = config.effect.glow_opacity;
    let glow_color = Color::from_argb(glow_opacity, gc.r, gc.g, gc.b);

    let canvas = surface.canvas();

    let mut paint_color = Paint::default();
    paint_color.set_color(glow_color);
    paint_color.set_blend_mode(BlendMode::SrcIn);

    let mut paint_under = Paint::default();
    paint_under.set_blend_mode(BlendMode::DstOver);

    canvas.save_layer(&Default::default());
    canvas.draw_image(&glow_ring, (0, 0), None);
    canvas.draw_paint(&paint_color);
    canvas.restore();

    canvas.save_layer(&Default::default());
    canvas.draw_image(&snapshot, (0, 0), Some(&paint_under));
    canvas.restore();
}

fn extract_alpha_mask(image: &skia_safe::Image, w: i32, h: i32) -> skia_safe::Image {
    let info = ImageInfo::new(
        (w, h),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );

    let row_bytes = (w * 4) as usize;
    let mut pixels = vec![0u8; (w * h * 4) as usize];
    image.read_pixels(&info, &mut pixels, row_bytes, (0, 0), skia_safe::image::CachingHint::Allow);

    for i in (0..pixels.len()).step_by(4) {
        let a = if pixels[i + 3] > 0 { 255 } else { 0 };
        pixels[i] = 255;
        pixels[i + 1] = 255;
        pixels[i + 2] = 255;
        pixels[i + 3] = a;
    }

    let data = skia_safe::Data::new_copy(&pixels);
    skia_safe::images::raster_from_data(&info, data, row_bytes)
        .expect("Failed to create alpha mask image")
}
