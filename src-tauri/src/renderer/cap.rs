use crate::config::{CapShape, RgbaColor, TailConfig};
use tiny_skia::*;

use crate::renderer::render::RenderLayout;

pub fn draw_cap_layer(pixmap: &mut Pixmap, config: &TailConfig, l: &RenderLayout) {
    if l.cap_h == 0 { return; }
    let (left, right, y_start, y_end) = (l.left, l.right, l.cap_start, l.cap_end);
    if left >= right { return; }

    let color = if config.cap.independent_settings { config.cap.color } else { config.global_color };
    let opacity = if config.cap.independent_settings { config.cap.opacity } else { config.global_opacity };

    match config.cap.shape {
        CapShape::Rect => draw_rect(&mut pixmap.as_mut(), left, right, y_start, y_end, color, opacity),
        CapShape::Ball => draw_ball(&mut pixmap.as_mut(), left, right, y_start, y_end, color, opacity),
        CapShape::Diamond => draw_diamond(&mut pixmap.as_mut(), left, right, y_start, y_end, color, opacity),
        CapShape::Gradient => draw_gradient(&mut pixmap.as_mut(), left, right, y_start, y_end, color, opacity),
    }
}

fn to_color(c: RgbaColor, opacity: u8) -> Color {
    let a = (c.a as u16 * opacity as u16 / 255) as u8;
    Color::from_rgba8(c.r, c.g, c.b, a)
}

fn solid_paint(c: RgbaColor, opacity: u8) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color(to_color(c, opacity));
    paint.anti_alias = true;
    paint
}

fn draw_rect(pixmap: &mut PixmapMut, left: u32, right: u32, y_start: u32, y_end: u32, color: RgbaColor, opacity: u8) {
    let rect = Rect::from_xywh(left as f32, y_start as f32, (right - left) as f32, (y_end - y_start) as f32).unwrap();
    pixmap.fill_rect(rect, &solid_paint(color, opacity), Transform::identity(), None);
}

fn draw_ball(pixmap: &mut PixmapMut, left: u32, right: u32, y_start: u32, y_end: u32, color: RgbaColor, opacity: u8) {
    // 上半椭圆：圆心在 y_end（底边），椭圆下半会溢出，用 ClipMask 裁剪到 cap 区域
    let cx = (left + right) as f32 / 2.0;
    let cy = y_end as f32;
    let rx = (right - left) as f32 / 2.0;
    let ry = y_end.saturating_sub(y_start).max(1) as f32;

    let oval_rect = Rect::from_xywh(cx - rx, cy - ry, rx * 2.0, ry * 2.0).unwrap();
    let mut pb = PathBuilder::new();
    pb.push_oval(oval_rect);
    let path = pb.finish().unwrap();

    // 裁剪矩形路径，只保留 cap 区域内的像素
    let clip_rect = Rect::from_xywh(left as f32, y_start as f32, (right - left) as f32, (y_end - y_start) as f32).unwrap();
    let mut clip_pb = PathBuilder::new();
    clip_pb.push_rect(clip_rect);
    let clip_path = clip_pb.finish().unwrap();
    let mut mask = Mask::new(pixmap.width(), pixmap.height()).unwrap();
    mask.fill_path(&clip_path, FillRule::Winding, false, Transform::identity());

    pixmap.fill_path(&path, &solid_paint(color, opacity), FillRule::Winding, Transform::identity(), Some(&mask));
}

fn draw_diamond(pixmap: &mut PixmapMut, left: u32, right: u32, y_start: u32, y_end: u32, color: RgbaColor, opacity: u8) {
    let cx = (left + right) as f32 / 2.0;
    let top_y = y_start as f32;
    let bot_y = y_end as f32;

    let mut pb = PathBuilder::new();
    pb.move_to(cx, top_y);
    pb.line_to(right as f32, bot_y);
    pb.line_to(left as f32, bot_y);
    pb.close();
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &solid_paint(color, opacity), FillRule::Winding, Transform::identity(), None);
}

fn draw_gradient(pixmap: &mut PixmapMut, left: u32, right: u32, y_start: u32, y_end: u32, color: RgbaColor, opacity: u8) {
    let a_top = to_color(color, 0);
    let a_bot = to_color(color, opacity);

    let gradient = LinearGradient::new(
        Point::from_xy(0.0, y_start as f32),
        Point::from_xy(0.0, (y_end - 1) as f32),
        vec![
            GradientStop::new(0.0, a_top),
            GradientStop::new(1.0, a_bot),
        ],
        SpreadMode::Pad,
        Transform::identity(),
    ).unwrap();

    let mut paint = Paint::default();
    paint.shader = gradient;
    paint.anti_alias = false;

    let rect = Rect::from_xywh(left as f32, y_start as f32, (right - left) as f32, (y_end - y_start) as f32).unwrap();
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
}
