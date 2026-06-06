//! 暗化重复效果
use crate::config::TailConfig;
use skia_safe::{Canvas, Paint, Color, Rect, BlendMode};
use super::super::cap::draw_cap_layer;
use crate::renderer::render::RenderLayout;

pub fn draw_echo_layer(canvas: &Canvas, config: &TailConfig, l: &RenderLayout) {
    if !l.echo_enabled || l.cap_h == 0 { return; }
    let echo_cap_end = l.echo_cap_end.min(l.h);
    let right = l.right.min(l.w);
    if l.echo_start >= echo_cap_end || l.left >= right { return; }

    let mut echo_surface = skia_safe::surfaces::raster_n32_premul((l.w as i32, l.h as i32))
        .expect("Failed to create echo surface");
    let echo_canvas = echo_surface.canvas();
    echo_canvas.clear(Color::TRANSPARENT);

    let echo_config = create_echo_config(config);
    let echo_layout = RenderLayout {
        cap_start: l.echo_start,
        cap_end: echo_cap_end,
        cap_h: echo_cap_end - l.echo_start,
        left: l.left,
        right,
        ..*l
    };
    draw_cap_layer(echo_canvas, &echo_config, &echo_layout);

    let echo_color = config.effect.echo_color;
    let a = (echo_color.a as u32 * config.effect.echo_opacity as u32 * config.global_opacity as u32 / 65025) as u8;
    let fill_color = Color::from_argb(a, echo_color.r, echo_color.g, echo_color.b);
    let fill_h = l.cap_end.min(l.h).saturating_sub(echo_cap_end);
    if fill_h > 0 {
        let rect = Rect::from_xywh(l.left as f32, echo_cap_end as f32, (right - l.left) as f32, fill_h as f32);
        let mut paint = Paint::default();
        paint.set_blend_mode(BlendMode::Src);
        paint.set_color(fill_color);
        echo_canvas.draw_rect(rect, &paint);
    }

    let echo_image = echo_surface.image_snapshot();
    let mut paint = Paint::default();
    paint.set_blend_mode(BlendMode::DstOver);
    canvas.draw_image(&echo_image, (0, 0), Some(&paint));
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
