mod cap;
mod body;

use cap::draw_cap;
use body::draw_body;

use crate::config::{CapShape, TailConfig};
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
    let echo_enabled = config.effect.cap_echo_enabled && config.cap.shape != CapShape::Gradient;
    let echo_rect_h = if echo_enabled { config.effect.echo_length } else { 0 };

    // 计算各区域位置
    // echo 关闭时不预留 echo cap 高度，让正常顶端紧跟 throw 之后
    let echo_cap_h = if echo_enabled { cap_h } else { 0 };
    let echo_start = config.throw_length;
    let echo_cap_end = echo_start + echo_cap_h; // 暗化顶端结束位置
    let echo_rect_end = echo_cap_end + echo_rect_h; // 矩形结束位置
    let cap_start = echo_rect_end; // 正常顶端开始位置
    let cap_end = cap_start + cap_h;
    let body_start = cap_end;
    let body_h = config.body_height();

    // echo（顶端暗化重复）—— 图层方式：先画完整 echo 图层，再叠正常内容
    if echo_enabled && cap_h > 0 {
        let echo_cap_end = echo_cap_end.min(h);
        let cr = cr.min(w);

        if echo_start < echo_cap_end && cl < cr {
            // 1. 创建 echo 图层，绘制完整 echo 区域（cap 形状 + 矩形延伸到正常顶端底部）
            let mut echo_layer: RgbaImage = ImageBuffer::from_pixel(w, h, Rgba([0, 0, 0, 0]));
            let echo_config = create_echo_config(config);
            draw_cap(&mut echo_layer, &echo_config, cl, cr, echo_start, echo_cap_end);
            // echo 矩形：从 cap 底部延伸到正常顶端底部，覆盖交界处不规则区域
            let echo_color = config.effect.echo_color;
            let echo_opacity = config.effect.echo_opacity;
            let a = (echo_color.a as u16 * echo_opacity as u16 / 255) as u8;
            let px = Rgba([echo_color.r, echo_color.g, echo_color.b, a]);
            let fill_end = cap_end.min(h);
            for y in echo_cap_end..fill_end {
                for x in cl..cr {
                    echo_layer.put_pixel(x, y, px);
                }
            }
            // 2. 合并到主图片：只在主图片透明处绘制 echo 像素
            for y in echo_start..fill_end {
                for x in cl..cr {
                    let ep = echo_layer.get_pixel(x, y);
                    if ep[3] > 0 && img.get_pixel(x, y)[3] == 0 {
                        img.put_pixel(x, y, *ep);
                    }
                }
            }
        }
    }

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

/// 创建 echo 效果的配置（使用 echo 的颜色和透明度）
fn create_echo_config(config: &TailConfig) -> TailConfig {
    let mut echo_config = config.clone();
    echo_config.cap.color = config.effect.echo_color;
    echo_config.cap.independent_opacity = true;
    echo_config.cap.opacity = config.effect.echo_opacity;
    // echo 的 scale 需要按比例缩放，使其高度等于 echo_length
    let cap_h = config.cap_height();
    if cap_h > 0 {
        let scale_factor = config.effect.echo_length as f64 / cap_h as f64;
        echo_config.cap.scale = (config.cap.scale as f64 * scale_factor) as u32;
        // 确保 scale 至少为 1
        if echo_config.cap.scale == 0 {
            echo_config.cap.scale = 1;
        }
    }
    echo_config
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
