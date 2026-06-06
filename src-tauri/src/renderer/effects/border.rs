//! 边框效果 - 基于二值 Shape Mask 的形态学腐蚀
use crate::config::{RgbaColor, TailConfig};
use skia_safe::{Surface, Paint, Color, image_filters, BlendMode, ImageInfo, ColorType, AlphaType};

pub fn draw_border_layer(surface: &mut Surface, config: &TailConfig) {
    if !config.body.border_enabled || config.body.border_width == 0 { return; }

    let snapshot = surface.image_snapshot();
    let (w, h) = (snapshot.width(), snapshot.height());
    let border_width = config.body.border_width as f32;

    let info = ImageInfo::new((w, h), ColorType::RGBA8888, AlphaType::Premul, None);
    let row_bytes = (w * 4) as usize;
    let mut src_pixels = vec![0u8; (w * h * 4) as usize];
    snapshot.read_pixels(&info, &mut src_pixels, row_bytes, (0, 0), skia_safe::image::CachingHint::Allow);

    // 1. 二值化 Alpha → Shape Mask
    let mut binary_pixels = vec![0u8; src_pixels.len()];
    for i in (0..src_pixels.len()).step_by(4) {
        let a = if src_pixels[i + 3] > 0 { 255u8 } else { 0u8 };
        binary_pixels[i] = 255;
        binary_pixels[i + 1] = 255;
        binary_pixels[i + 2] = 255;
        binary_pixels[i + 3] = a;
    }

    let binary_data = skia_safe::Data::new_copy(&binary_pixels);
    let binary_mask = skia_safe::images::raster_from_data(&info, binary_data, row_bytes)
        .expect("Failed to create binary mask");

    // 2. 对二值 Mask 做腐蚀 → eroded_mask
    let erode_filter = image_filters::erode((border_width, border_width), None, None)
        .expect("Failed to create erode filter");

    let mut eroded_surface = skia_safe::surfaces::raster_n32_premul((w, h))
        .expect("Failed to create eroded surface");
    eroded_surface.canvas().clear(Color::TRANSPARENT);

    let mut paint_erode = Paint::default();
    paint_erode.set_image_filter(erode_filter);
    eroded_surface.canvas().draw_image(&binary_mask, (0, 0), Some(&paint_erode));

    let eroded_mask = eroded_surface.image_snapshot();

    // 3. 边框区域 = binary_mask − eroded_mask
    let mut border_mask_surface = skia_safe::surfaces::raster_n32_premul((w, h))
        .expect("Failed to create border mask surface");
    let border_mask_canvas = border_mask_surface.canvas();
    border_mask_canvas.clear(Color::TRANSPARENT);
    border_mask_canvas.draw_image(&binary_mask, (0, 0), None);

    let mut paint_dst_out = Paint::default();
    paint_dst_out.set_blend_mode(BlendMode::DstOut);
    border_mask_canvas.draw_image(&eroded_mask, (0, 0), Some(&paint_dst_out));

    // 4. 根据设置决定透明度模式
    if !config.body.border_opacity_independent {
        // 沿用原图 alpha：用原图 alpha 调制边框遮罩
        let border_mask_snapshot = border_mask_surface.image_snapshot();
        let mut final_surface = skia_safe::surfaces::raster_n32_premul((w, h))
            .expect("Failed to create final surface");
        let final_canvas = final_surface.canvas();
        final_canvas.clear(Color::TRANSPARENT);
        final_canvas.draw_image(&border_mask_snapshot, (0, 0), None);

        let mut paint_alpha = Paint::default();
        paint_alpha.set_blend_mode(BlendMode::DstIn);
        let alpha_data = skia_safe::Data::new_copy(&src_pixels);
        if let Some(alpha_img) = skia_safe::images::raster_from_data(&info, alpha_data, row_bytes) {
            final_canvas.draw_image(&alpha_img, (0, 0), Some(&paint_alpha));
        }

        let (br, bg, bb) = border_rgb(config);
        let border_color = Color::from_argb(255, br, bg, bb);

        let canvas = surface.canvas();
        let mut paint_color = Paint::default();
        paint_color.set_color(border_color);
        paint_color.set_blend_mode(BlendMode::SrcIn);

        canvas.save_layer(&Default::default());
        canvas.draw_image(&final_surface.image_snapshot(), (0, 0), None);
        canvas.draw_paint(&paint_color);
        canvas.restore();
    } else {
        // 独立透明度：直接使用配置的 border_opacity
        let border_mask = border_mask_surface.image_snapshot();
        let (br, bg, bb) = border_rgb(config);
        let border_color = Color::from_argb(config.body.border_opacity, br, bg, bb);

        let canvas = surface.canvas();
        let mut paint_color = Paint::default();
        paint_color.set_color(border_color);
        paint_color.set_blend_mode(BlendMode::SrcIn);

        canvas.save_layer(&Default::default());
        canvas.draw_image(&border_mask, (0, 0), None);
        canvas.draw_paint(&paint_color);
        canvas.restore();
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
