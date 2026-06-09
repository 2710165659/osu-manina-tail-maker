//! 暗化重复效果
use crate::config::TailConfig;
use tiny_skia::*;
use super::super::cap::draw_cap_layer;
use crate::renderer::render::RenderLayout;

pub fn draw_echo_layer(pixmap: &mut Pixmap, config: &TailConfig, l: &RenderLayout) {
    if !l.echo_enabled || l.cap_h == 0 { return; }
    let echo_cap_end = l.echo_cap_end.min(l.h);
    let right = l.right.min(l.w);
    if l.echo_start >= echo_cap_end || l.left >= right { return; }

    let mut echo_pixmap = Pixmap::new(l.w, l.h).unwrap();
    let echo_config = create_echo_config(config);
    let echo_layout = RenderLayout { cap_start: l.echo_start, cap_end: echo_cap_end, cap_h: echo_cap_end - l.echo_start, left: l.left, right, ..*l };
    draw_cap_layer(&mut echo_pixmap, &echo_config, &echo_layout);

    let echo_color = config.effect.echo_color;
    let a = (echo_color.a as u32 * config.effect.echo_opacity as u32 * config.global_opacity as u32 / 65025) as u8;
    let fill_color = Color::from_rgba8(echo_color.r, echo_color.g, echo_color.b, a);
    let fill_h = l.cap_end.min(l.h).saturating_sub(echo_cap_end);
    if fill_h > 0 {
        let rect = Rect::from_xywh(l.left as f32, echo_cap_end as f32, (right - l.left) as f32, fill_h as f32).unwrap();
        let mut paint = Paint::default();
        paint.blend_mode = BlendMode::Source;
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
