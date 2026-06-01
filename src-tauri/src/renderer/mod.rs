mod cap;
mod body;

use cap::draw_cap;
use body::draw_body;

use crate::config::TailConfig;
use image::{ImageBuffer, Rgba, RgbaImage};

/// 预览最大行数
pub const PREVIEW_MAX_ROWS: u32 = 800;

/// 完整渲染（导出用）
pub fn render(config: &TailConfig) -> RgbaImage {
    let w = config.image.width;
    let h = config.image.height;
    let mut img: RgbaImage = ImageBuffer::from_pixel(w, h, Rgba([0, 0, 0, 0]));

    let cl = config.margin;
    let cr = w.saturating_sub(config.margin);
    let cap_h = config.cap_height();
    let cap_start = config.cap_start_y();
    let cap_end = config.cap_end_y();
    let body_start = cap_end;
    let body_h = config.body_height();

    // cap
    if cap_h > 0 {
        draw_cap(&mut img, config, cl, cr, cap_start, cap_end);
    }
    // body
    if body_h > 0 {
        draw_body(&mut img, config, cl, cr, body_start, body_h);
    }
    // 整体边框（cap + body 统一描边，从 cap_start 到图片底部）
    if config.body.border_enabled {
        draw_border(&mut img, config, cl, cr, cap_start, h);
    }
    // 全局透明度
    if config.global_opacity < 255 {
        for p in img.pixels_mut() {
            if p[3] > 0 {
                p[3] = (p[3] as u16 * config.global_opacity as u16 / 255) as u8;
            }
        }
    }
    img
}

/// 在 [left,right) × [y0,y1) 区域四周画边框（两轮：先满填充再覆边）
fn draw_border(img: &mut RgbaImage, config: &TailConfig, left: u32, right: u32, y0: u32, y1: u32) {
    let bw = config.body.border_width;
    if bw == 0 { return }
    let safe_bw = bw.min((right - left) / 2).min((y1.saturating_sub(y0)) / 2);
    if safe_bw == 0 { return }

    let bc = config.body.border_color;
    let bo = config.body.border_opacity;
    let a = (bc.a as u16 * bo as u16 / 255) as u8;
    let px = Rgba([bc.r, bc.g, bc.b, a]);

    // 上边
    for y in y0..(y0 + safe_bw).min(y1) { for x in left..right { img.put_pixel(x, y, px); } }
    // 下边
    let bot = y1.saturating_sub(safe_bw);
    for y in bot..y1 { for x in left..right { img.put_pixel(x, y, px); } }
    // 左边
    let mt = (y0 + safe_bw).min(y1);
    let mb = bot;
    for y in mt..mb { for x in left..(left + safe_bw).min(right) { img.put_pixel(x, y, px); } }
    // 右边
    let rs = right.saturating_sub(safe_bw);
    for y in mt..mb { for x in rs..right { img.put_pixel(x, y, px); } }
}

/// 预览：全宽，最多 PREVIEW_MAX_ROWS 行
pub fn render_preview(config: &TailConfig) -> RgbaImage {
    let full = render(config);
    let h = full.height().min(PREVIEW_MAX_ROWS);
    let w = full.width();
    ImageBuffer::from_fn(w, h, |x, y| *full.get_pixel(x, y))
}
