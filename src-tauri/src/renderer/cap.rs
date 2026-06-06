use crate::config::{CapShape, RgbaColor, TailConfig};
use skia_safe::{Canvas, Paint, Color, Rect, Path, ClipOp, BlendMode, Point, Shader, gradient_shader::GradientShaderColors, TileMode};

use crate::renderer::render::RenderLayout;

pub fn draw_cap_layer(canvas: &Canvas, config: &TailConfig, l: &RenderLayout) {
    if l.cap_h == 0 { return; }
    let (left, right, y_start, y_end) = (l.left, l.right, l.cap_start, l.cap_end);
    if left >= right { return; }

    let color = if config.cap.independent_settings { config.cap.color } else { config.global_color };
    let opacity = if config.cap.independent_settings { config.cap.opacity } else { config.global_opacity };

    match config.cap.shape {
        CapShape::Rect => draw_rect(canvas, left, right, y_start, y_end, color, opacity),
        CapShape::Ball => draw_ball(canvas, left, right, y_start, y_end, color, opacity),
        CapShape::Diamond => draw_diamond(canvas, left, right, y_start, y_end, color, opacity),
        CapShape::Gradient => draw_gradient(canvas, left, right, y_start, y_end, color, opacity),
    }
}

fn to_color(c: RgbaColor, opacity: u8) -> Color {
    let a = (c.a as u16 * opacity as u16 / 255) as u8;
    Color::from_argb(a, c.r, c.g, c.b)
}

fn solid_paint(c: RgbaColor, opacity: u8) -> Paint {
    let mut paint = Paint::default();
    paint.set_blend_mode(BlendMode::Src);
    paint.set_color(to_color(c, opacity));
    paint.set_anti_alias(true);
    paint
}

fn draw_rect(canvas: &Canvas, left: u32, right: u32, y_start: u32, y_end: u32, color: RgbaColor, opacity: u8) {
    let rect = Rect::from_xywh(left as f32, y_start as f32, (right - left) as f32, (y_end - y_start) as f32);
    canvas.draw_rect(rect, &solid_paint(color, opacity));
}

fn draw_ball(canvas: &Canvas, left: u32, right: u32, y_start: u32, y_end: u32, color: RgbaColor, opacity: u8) {
    let cx = (left + right) as f32 / 2.0;
    let cy = y_end as f32;
    let rx = (right - left) as f32 / 2.0;
    let ry = y_end.saturating_sub(y_start).max(1) as f32;

    let oval_rect = Rect::from_xywh(cx - rx, cy - ry, rx * 2.0, ry * 2.0);

    canvas.save();
    let clip_rect = Rect::from_xywh(left as f32, y_start as f32, (right - left) as f32, (y_end - y_start) as f32);
    canvas.clip_rect(clip_rect, ClipOp::Intersect, true);
    canvas.draw_oval(oval_rect, &solid_paint(color, opacity));
    canvas.restore();
}

fn draw_diamond(canvas: &Canvas, left: u32, right: u32, y_start: u32, y_end: u32, color: RgbaColor, opacity: u8) {
    let cx = (left + right) as f32 / 2.0;
    let top_y = y_start as f32;
    let bot_y = y_end as f32;

    let mut path = Path::new();
    path.move_to((cx, top_y));
    path.line_to((right as f32, bot_y));
    path.line_to((left as f32, bot_y));
    path.close();

    canvas.draw_path(&path, &solid_paint(color, opacity));
}

fn draw_gradient(canvas: &Canvas, left: u32, right: u32, y_start: u32, y_end: u32, color: RgbaColor, opacity: u8) {
    let a_top = to_color(color, 0);
    let a_bot = to_color(color, opacity);

    let colors = vec![a_top, a_bot];
    let positions = vec![0.0, 1.0];
    let start = Point::new(0.0, y_start as f32);
    let end = Point::new(0.0, (y_end - 1) as f32);

    let shader = Shader::linear_gradient(
        (start, end),
        GradientShaderColors::Colors(&colors),
        Some(&positions[..]),
        TileMode::Clamp,
        None,
        None,
    ).expect("Failed to create gradient");

    let mut paint = Paint::default();
    paint.set_blend_mode(BlendMode::Src);
    paint.set_shader(shader);
    paint.set_anti_alias(false);

    let rect = Rect::from_xywh(left as f32, y_start as f32, (right - left) as f32, (y_end - y_start) as f32);
    canvas.draw_rect(rect, &paint);
}
