mod cap;
mod body;

use cap::draw_cap;
use body::draw_body;

use crate::config::TailConfig;
use image::{ImageBuffer, Rgba, RgbaImage};

/// 预览最大行数
pub const PREVIEW_MAX_ROWS: u32 = 500;

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

/// 在内容区边缘画边框，边框沿 cap 形状内蚀，挤占 body
/// 使用 4 方向距离扫描算法，O(W×H) 复杂度，与 border_width 无关
fn draw_border(img: &mut RgbaImage, config: &TailConfig, left: u32, right: u32, y0: u32, y1: u32) {
    let bw = config.body.border_width;
    if bw == 0 { return }

    let w = img.width() as usize;
    let h = img.height() as usize;
    let x0 = left as usize;
    let x1 = (right as usize).min(w);
    let y_top = y0 as usize;
    let y_bot = (y1 as usize).min(h);
    let ibw = bw as i32;

    // 用位图标记边框像素，避免干扰扫描检测
    let mut is_border = vec![false; w * h];
    // 直接读原始 RGBA 缓冲区，跳过 get_pixel 的边界检查
    let raw = img.as_raw(); // &[u8]

    // 辅助：计算像素索引，读取 alpha
    let alpha_at = |raw: &[u8], idx: usize| -> u8 { raw[idx * 4 + 3] };

    // ── 上方向扫描（top → bottom）────────────────────────
    for x in x0..x1 {
        let mut dist = 0i32; // 扫描区上边界视为紧邻透明像素
        for y in y_top..y_bot {
            let idx = y * w + x;
            if alpha_at(raw, idx) == 0 {
                dist = 0;
            } else {
                dist += 1;
                if dist <= ibw {
                    is_border[idx] = true;
                }
            }
        }
    }

    // ── 下方向扫描（bottom → top）────────────────────────
    for x in x0..x1 {
        let mut dist = 0i32;
        for y in (y_top..y_bot).rev() {
            let idx = y * w + x;
            if alpha_at(raw, idx) == 0 {
                dist = 0;
            } else {
                dist += 1;
                if dist <= ibw {
                    is_border[idx] = true;
                }
            }
        }
    }

    // ── 左方向扫描（left → right）────────────────────────
    for y in y_top..y_bot {
        let mut dist = 0i32;
        for x in x0..x1 {
            let idx = y * w + x;
            if alpha_at(raw, idx) == 0 {
                dist = 0;
            } else {
                dist += 1;
                if dist <= ibw {
                    is_border[idx] = true;
                }
            }
        }
    }

    // ── 右方向扫描（right → left）────────────────────────
    for y in y_top..y_bot {
        let mut dist = 0i32;
        for x in (x0..x1).rev() {
            let idx = y * w + x;
            if alpha_at(raw, idx) == 0 {
                dist = 0;
            } else {
                dist += 1;
                if dist <= ibw {
                    is_border[idx] = true;
                }
            }
        }
    }

    // ── 一次性写入边框颜色 ──────────────────────────────
    let bc = config.body.border_color;
    let bo = config.body.border_opacity;
    let a = (bc.a as u16 * bo as u16 / 255) as u8;
    let [r, g, b, _] = Rgba([bc.r, bc.g, bc.b, a]).0;

    let buf = img.as_mut_ptr();
    for y in y_top..y_bot {
        for x in x0..x1 {
            let idx = y * w + x;
            if is_border[idx] {
                unsafe {
                    let p = buf.add(idx * 4);
                    *p = r;
                    *p.add(1) = g;
                    *p.add(2) = b;
                    *p.add(3) = a;
                }
            }
        }
    }
}

/// 预览：全宽，最多 PREVIEW_MAX_ROWS 行
pub fn render_preview(config: &TailConfig) -> RgbaImage {
    let full = render(config);
    let h = full.height().min(PREVIEW_MAX_ROWS);
    let w = full.width();
    ImageBuffer::from_fn(w, h, |x, y| *full.get_pixel(x, y))
}
