use crate::config::TailConfig;
use image::{Rgba, RgbaImage};

/// 绘制身体区域（纯填充，边框在渲染管线统一处理）
/// 独立设置关闭时，使用投皮头的颜色和透明度
pub fn draw_body(
    img: &mut RgbaImage,
    config: &TailConfig,
    left: u32,
    right: u32,
    y_start: u32,
    body_height: u32,
) {
    let y_end = y_start + body_height;
    let (fc, fo) = if config.body.independent_fill {
        (config.body.fill_color, config.body.fill_opacity)
    } else {
        (config.cap.color, 255)
    };
    let a = (fc.a as u16 * fo as u16 / 255) as u8;
    let px = Rgba([fc.r, fc.g, fc.b, a]);
    for y in y_start..y_end {
        for x in left..right {
            img.put_pixel(x, y, px);
        }
    }
}
