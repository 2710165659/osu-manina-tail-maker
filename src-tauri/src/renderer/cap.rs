use crate::config::{CapShape, TailConfig};
use image::{Rgba, RgbaImage};

/// 超采样倍率（用于抗锯齿）
const SSAA: u32 = 4;
const SSAA2: u32 = SSAA * SSAA;

/// 绘制 Cap 区域
pub fn draw_cap(
    img: &mut RgbaImage,
    config: &TailConfig,
    left: u32,
    right: u32,
    y_start: u32,
    y_end: u32,
) {
    let content_w = right - left;
    let cap_h = y_end - y_start;

    let cap_color = config.cap.color;
    let cap_opacity = if config.cap.independent_opacity {
        config.cap.opacity
    } else {
        255
    };

    match config.cap.shape {
        CapShape::Rect => {
            // 矩形：填满整个内容区
            for y in y_start..y_end {
                for x in left..right {
                    img.put_pixel(x, y, rgba_pixel(cap_color, cap_opacity));
                }
            }
        }
        CapShape::Ball => {
            // 上半椭圆：圆心在 Cap 底部中心
            let cx = (left + right) as f64 / 2.0;
            let cy = y_end as f64; // 圆心在 Cap 区域下方边界
            let rx = content_w as f64 / 2.0;
            let ry = cap_h as f64;

            for y in y_start..y_end {
                for x in left..right {
                    let coverage = ellipse_coverage(x, y, cx, cy, rx, ry);
                    if coverage > 0.0 {
                        let alpha = (coverage * cap_opacity as f64).round() as u8;
                        img.put_pixel(x, y, rgba_pixel(cap_color, alpha));
                    }
                }
            }
        }
        CapShape::Diamond => {
            // 上半菱形：顶点在 Cap 起始线中心，底部宽度 = 内容区宽度
            let cx = (left + right) as f64 / 2.0;
            for y in y_start..y_end {
                let t = (y - y_start) as f64 / cap_h as f64; // 0(顶) → 1(底)
                let half_w = (content_w as f64 / 2.0) * t; // 从 0 扩展到 content_w/2
                let left_bound = cx - half_w;
                let right_bound = cx + half_w;

                for x in left..right {
                    let coverage = diamond_coverage(
                        x, y,
                        left_bound, right_bound,
                    );
                    if coverage > 0.0 {
                        let alpha = (coverage * cap_opacity as f64).round() as u8;
                        img.put_pixel(x, y, rgba_pixel(cap_color, alpha));
                    }
                }
            }
        }
        CapShape::Gradient => {
            // 矩形 + 透明度渐变：从上到下透明度递增
            for y in y_start..y_end {
                let t = (y - y_start) as f64 / cap_h as f64; // 0(顶,全透明) → 1(底,目标透明度)
                let row_alpha = (t * cap_opacity as f64).round() as u8;

                for x in left..right {
                    img.put_pixel(x, y, rgba_pixel(cap_color, row_alpha));
                }
            }
        }
    }
}

/// 椭圆覆盖率：对像素 (px,py) 进行超采样，计算落在椭圆内的子采样比例
/// 椭圆方程：(x-cx)²/rx² + (y-cy)²/ry² ≤ 1.0
fn ellipse_coverage(px: u32, py: u32, cx: f64, cy: f64, rx: f64, ry: f64) -> f64 {
    let mut count = 0u32;
    for si in 0..SSAA {
        let sy = py as f64 + (si as f64 + 0.5) / SSAA as f64;
        for sj in 0..SSAA {
            let sx = px as f64 + (sj as f64 + 0.5) / SSAA as f64;
            let dx = (sx - cx) / rx;
            let dy = (sy - cy) / ry;
            if dx * dx + dy * dy <= 1.0 {
                count += 1;
            }
        }
    }
    count as f64 / SSAA2 as f64
}

/// 菱形覆盖率：像素在菱形左右边界之间的比例
fn diamond_coverage(px: u32, _py: u32, left: f64, right: f64) -> f64 {
    let px_f = px as f64;
    // 像素中心在边界内
    if px_f + 0.5 <= left || px_f - 0.5 >= right {
        return 0.0;
    }
    if px_f - 0.5 >= left && px_f + 0.5 <= right {
        return 1.0;
    }
    // 像素部分覆盖（边缘抗锯齿）
    let overlap_left = if px_f - 0.5 < left {
        (px_f + 0.5 - left).max(0.0).min(1.0)
    } else {
        1.0
    };
    let overlap_right = if px_f + 0.5 > right {
        (right - (px_f - 0.5)).max(0.0).min(1.0)
    } else {
        1.0
    };
    overlap_left.min(overlap_right).max(0.0)
}

/// 将 RgbaColor + opacity 转为 Rgba<u8> 像素
fn rgba_pixel(color: crate::config::RgbaColor, opacity: u8) -> Rgba<u8> {
    let a = (color.a as u16 * opacity as u16 / 255) as u8;
    Rgba([color.r, color.g, color.b, a])
}
