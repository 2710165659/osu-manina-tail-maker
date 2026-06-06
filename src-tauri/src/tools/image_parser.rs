use image::RgbaImage;
use std::path::Path;

use crate::config::{
    BodyConfig, CapConfig, CapShape, EffectConfig, ImageConfig, RgbaColor, TailConfig,
};

/// 图片解析结果
#[derive(Debug)]
pub struct ParseResult {
    pub config: TailConfig,
    pub warnings: Vec<String>,
}

/// 解析错误
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("解析失败或非投皮：body 高度不足 5000px")]
    BodyTooShort,
    #[error("无法读取图片: {0}")]
    ImageReadError(#[from] image::ImageError),
    #[error("图片为空或尺寸过小")]
    ImageTooSmall,
}

/// 解析图片为 TailConfig
pub fn parse_image(image_path: &Path) -> Result<ParseResult, ParseError> {
    let img = image::open(image_path)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    if width == 0 || height == 0 {
        return Err(ParseError::ImageTooSmall);
    }

    let mut warnings = Vec::new();

    // 1. 基本信息
    let filename = format!(
        "upload-{}",
        image_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
    );

    let image_config = ImageConfig {
        width,
        height,
        filename,
    };

    // 2. 边缘检测
    let margin = detect_margin(&rgba);
    let throw_length = detect_throw_length(&rgba);

    if margin == 0 {
        warnings.push("未检测到左右边距，使用默认值 0".to_string());
    }
    if throw_length == 0 {
        warnings.push("未检测到投的长度，使用默认值 0".to_string());
    }

    // 3. 从下到上检测 body
    let body_region = detect_body(&rgba, margin)?;
    let (global_color, global_opacity) = extract_body_color(&rgba, &body_region);

    // 4. 检测 cap
    let (cap_config, cap_warnings) = detect_cap(&rgba, &body_region, margin);
    warnings.extend(cap_warnings);

    // 5. 检测 border
    let (border_enabled, border_color, border_width, border_warnings) =
        detect_border(&rgba, &body_region);
    warnings.extend(border_warnings);

    let config = TailConfig {
        image: image_config,
        margin,
        throw_length,
        global_color,
        global_opacity,
        cap: cap_config,
        body: BodyConfig {
            independent_settings: false,
            color: global_color,
            opacity: global_opacity,
            border_enabled,
            border_color,
            border_opacity: 255,
            border_opacity_independent: false,
            border_match_body: false,
            border_width,
        },
        effect: EffectConfig {
            cap_echo_enabled: false,
            echo_color: RgbaColor {
                r: 113,
                g: 113,
                b: 113,
                a: 76,
            },
            echo_opacity: 76,
            echo_length: 50,
            glow_enabled: false,
            glow_color: RgbaColor {
                r: 144,
                g: 238,
                b: 144,
                a: 255,
            },
            glow_opacity: 180,
            glow_size: 8,
            glow_spread: 4,
            glow_match_body: false,
            glow_opacity_independent: false,
        },
    };

    Ok(ParseResult { config, warnings })
}

/// 检测左右边距（扫描左右边缘透明像素）
fn detect_margin(rgba: &RgbaImage) -> u32 {
    let (width, height) = rgba.dimensions();
    let mut max_margin = 0u32;

    // 扫描左侧
    for x in 0..width / 2 {
        let mut all_transparent = true;
        for y in 0..height {
            if rgba.get_pixel(x, y)[3] > 0 {
                all_transparent = false;
                break;
            }
        }
        if all_transparent {
            max_margin = x + 1;
        } else {
            break;
        }
    }

    // 扫描右侧
    for x in (width / 2..width).rev() {
        let mut all_transparent = true;
        for y in 0..height {
            if rgba.get_pixel(x, y)[3] > 0 {
                all_transparent = false;
                break;
            }
        }
        if all_transparent {
            max_margin = max_margin.max(width - x);
        } else {
            break;
        }
    }

    max_margin
}

/// 检测投的长度（扫描顶部边缘透明像素）
fn detect_throw_length(rgba: &RgbaImage) -> u32 {
    let (width, height) = rgba.dimensions();
    let mut throw_length = 0u32;

    for y in 0..height {
        let mut all_transparent = true;
        for x in 0..width {
            if rgba.get_pixel(x, y)[3] > 0 {
                all_transparent = false;
                break;
            }
        }
        if all_transparent {
            throw_length = y + 1;
        } else {
            break;
        }
    }

    throw_length
}

/// Body 区域信息
struct BodyRegion {
    y_start: u32,
    y_end: u32,
    x_start: u32,
    x_end: u32,
}

/// 获取某一行非透明像素的宽度范围 (x_start, x_end)
fn row_width_range(rgba: &RgbaImage, y: u32) -> (u32, u32) {
    let w = rgba.width();
    let mut x_start = w;
    let mut x_end = 0u32;
    for x in 0..w {
        if rgba.get_pixel(x, y)[3] > 0 {
            x_start = x_start.min(x);
            x_end = x + 1;
        }
    }
    (x_start, x_end)
}

/// 检测 body 区域（从底部向上，用宽度一致性区分 body 和 cap）
fn detect_body(rgba: &RgbaImage, _margin: u32) -> Result<BodyRegion, ParseError> {
    let (_width, height) = rgba.dimensions();

    // 第一步：从底部找到第一个非透明行，确定 body 的参考宽度
    let mut bottom_y = height;
    for y in (0..height).rev() {
        let (xs, xe) = row_width_range(rgba, y);
        if xs < xe {
            bottom_y = y;
            break;
        }
    }
    if bottom_y == height {
        return Err(ParseError::BodyTooShort);
    }

    let (ref_xs, ref_xe) = row_width_range(rgba, bottom_y);
    let ref_width = ref_xe - ref_xs;
    if ref_width == 0 {
        return Err(ParseError::BodyTooShort);
    }

    // 允许的宽度偏差：10%
    let tolerance = (ref_width as f64 * 0.1).max(2.0) as u32;

    // 第二步：从底部向上扫描，body 区域宽度保持一致
    let body_y_end = bottom_y + 1;
    let mut body_y_start = bottom_y;

    for y in (0..bottom_y).rev() {
        let (xs, xe) = row_width_range(rgba, y);
        if xs >= xe {
            break; // 透明行，停止
        }
        let row_w = xe - xs;
        // 宽度偏差在容许范围内，认为还是 body
        let diff = if row_w > ref_width { row_w - ref_width } else { ref_width - row_w };
        if diff <= tolerance {
            body_y_start = y;
        } else {
            break; // 宽度变化，说明进入 cap 区域
        }
    }

    let body_height = body_y_end - body_y_start;
    if body_height < 5000 {
        return Err(ParseError::BodyTooShort);
    }

    Ok(BodyRegion {
        y_start: body_y_start,
        y_end: body_y_end,
        x_start: ref_xs,
        x_end: ref_xe,
    })
}

/// 提取 body 颜色（中心区域采样）
fn extract_body_color(rgba: &RgbaImage, body: &BodyRegion) -> (RgbaColor, u8) {
    let center_x = (body.x_start + body.x_end) / 2;
    let center_y = (body.y_start + body.y_end) / 2;

    // 在中心区域采样 10x10 像素
    let sample_size = 5i32;
    let mut total_r = 0u64;
    let mut total_g = 0u64;
    let mut total_b = 0u64;
    let mut total_a = 0u64;
    let mut count = 0u64;

    for dy in -sample_size..=sample_size {
        for dx in -sample_size..=sample_size {
            let x = (center_x as i32 + dx) as u32;
            let y = (center_y as i32 + dy) as u32;

            if x < rgba.width() && y < rgba.height() {
                let pixel = rgba.get_pixel(x, y);
                total_r += pixel[0] as u64;
                total_g += pixel[1] as u64;
                total_b += pixel[2] as u64;
                total_a += pixel[3] as u64;
                count += 1;
            }
        }
    }

    if count == 0 {
        return (
            RgbaColor {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            255,
        );
    }

    let r = (total_r / count) as u8;
    let g = (total_g / count) as u8;
    let b = (total_b / count) as u8;
    let a = (total_a / count) as u8;

    (RgbaColor { r, g, b, a }, a)
}

/// 检测 cap 形状
fn detect_cap(rgba: &RgbaImage, body: &BodyRegion, _margin: u32) -> (CapConfig, Vec<String>) {
    let mut warnings = Vec::new();
    let (img_width, _img_height) = rgba.dimensions();

    // cap 区域在 body 上方
    let cap_y_end = body.y_start;

    // 向上扫描找到 cap 的起始位置（扫描整个内容区宽度）
    let mut cap_y_start = cap_y_end;
    for y in (0..cap_y_end).rev() {
        let mut has_content = false;
        // 扫描整个内容区（包括 margin 区域）
        for x in 0..img_width {
            if rgba.get_pixel(x, y)[3] > 0 {
                has_content = true;
                break;
            }
        }
        if has_content {
            cap_y_start = y;
        } else {
            break;
        }
    }

    let cap_height = cap_y_end - cap_y_start;

    if cap_height == 0 {
        warnings.push("未检测到 cap，使用默认矩形，缩放100%".to_string());
        return (
            CapConfig {
                shape: CapShape::Rect,
                scale: 100,
                independent_settings: false,
                color: RgbaColor {
                    r: 113,
                    g: 113,
                    b: 113,
                    a: 255,
                },
                opacity: 255,
            },
            warnings,
        );
    }

    // 检测形状（使用 cap 的实际宽度范围）
    let mut cap_x_start = img_width;
    let mut cap_x_end = 0u32;
    for y in cap_y_start..cap_y_end {
        for x in 0..img_width {
            if rgba.get_pixel(x, y)[3] > 0 {
                cap_x_start = cap_x_start.min(x);
                cap_x_end = cap_x_end.max(x + 1);
            }
        }
    }

    let shape = detect_shape(rgba, cap_y_start, cap_y_end, cap_x_start, cap_x_end);

    // 计算 scale
    // scale = cap_height * 200 / content_width (对于非 gradient)
    let content_width = body.x_end - body.x_start;
    let scale = if content_width > 0 {
        ((cap_height as f64 * 200.0 / content_width as f64).round() as u32).min(1000)
    } else {
        100
    };

    (
        CapConfig {
            shape,
            scale,
            independent_settings: false,
            color: RgbaColor {
                r: 113,
                g: 113,
                b: 113,
                a: 255,
            },
            opacity: 255,
        },
        warnings,
    )
}

/// 检测形状（通过像素分布模式）
fn detect_shape(
    rgba: &RgbaImage,
    y_start: u32,
    y_end: u32,
    x_start: u32,
    x_end: u32,
) -> CapShape {
    let width = x_end - x_start;
    let height = y_end - y_start;

    if width == 0 || height == 0 {
        return CapShape::Rect;
    }

    // 检测是否为椭圆（球皮）
    if is_ellipse(rgba, y_start, y_end, x_start, x_end) {
        return CapShape::Ball;
    }

    // 检测是否为菱形
    if is_diamond(rgba, y_start, y_end, x_start, x_end) {
        return CapShape::Diamond;
    }

    // 默认矩形
    CapShape::Rect
}

/// 检测是否为椭圆
fn is_ellipse(
    rgba: &RgbaImage,
    y_start: u32,
    y_end: u32,
    x_start: u32,
    x_end: u32,
) -> bool {
    let width = x_end - x_start;
    let height = y_end - y_start;

    // 椭圆的宽高比应该接近 2:1（球皮）
    let aspect_ratio = width as f64 / height as f64;
    if aspect_ratio < 1.5 || aspect_ratio > 2.5 {
        return false;
    }

    // 检查每行的宽度是否符合椭圆曲线
    let center_y = (y_start + y_end) / 2;
    let mut match_count = 0;
    let mut total_count = 0;

    for y in y_start..y_end {
        let mut left = x_end;
        let mut right = x_start;

        for x in x_start..x_end {
            if rgba.get_pixel(x, y)[3] > 0 {
                left = left.min(x);
                right = right.max(x + 1);
            }
        }

        if left < right {
            total_count += 1;
            let row_width = right - left;

            // 椭圆方程: (y-h)^2/b^2 + (x-k)^2/a^2 = 1
            // 预期宽度 = 2a * sqrt(1 - (y-h)^2/b^2)
            let dy = (y as f64 - center_y as f64).abs();
            let expected_half_width =
                (width as f64 / 2.0) * (1.0 - (dy / (height as f64 / 2.0)).powi(2)).sqrt();
            let expected_width = (expected_half_width * 2.0) as u32;

            // 允许 10% 误差
            let diff = if row_width > expected_width {
                row_width - expected_width
            } else {
                expected_width - row_width
            };

            if diff < expected_width / 10 {
                match_count += 1;
            }
        }
    }

    // 80% 以上匹配则认为是椭圆
    total_count > 0 && match_count * 100 / total_count >= 80
}

/// 检测是否为菱形
fn is_diamond(
    rgba: &RgbaImage,
    y_start: u32,
    y_end: u32,
    x_start: u32,
    x_end: u32,
) -> bool {
    let width = x_end - x_start;
    let height = y_end - y_start;

    // 菱形的宽高比应该接近 2:1
    let aspect_ratio = width as f64 / height as f64;
    if aspect_ratio < 1.5 || aspect_ratio > 2.5 {
        return false;
    }

    // 检查每行的宽度是否符合菱形（线性变化）
    let center_y = (y_start + y_end) / 2;
    let mut match_count = 0;
    let mut total_count = 0;

    for y in y_start..y_end {
        let mut left = x_end;
        let mut right = x_start;

        for x in x_start..x_end {
            if rgba.get_pixel(x, y)[3] > 0 {
                left = left.min(x);
                right = right.max(x + 1);
            }
        }

        if left < right {
            total_count += 1;
            let row_width = right - left;

            // 菱形：宽度线性变化
            let dy = (y as f64 - center_y as f64).abs();
            let expected_width = (width as f64 * (1.0 - dy / (height as f64 / 2.0))) as u32;

            // 允许 10% 误差
            let diff = if row_width > expected_width {
                row_width - expected_width
            } else {
                expected_width - row_width
            };

            if diff < expected_width / 10 {
                match_count += 1;
            }
        }
    }

    // 80% 以上匹配则认为是菱形
    total_count > 0 && match_count * 100 / total_count >= 80
}

/// 检测边框（body 左侧 100px，模糊化采样 20 像素）
fn detect_border(
    rgba: &RgbaImage,
    body: &BodyRegion,
) -> (bool, RgbaColor, u32, Vec<String>) {
    let mut warnings = Vec::new();

    // 只检测 body 左侧 100px
    let detect_width = 100u32.min(body.x_end - body.x_start);
    let detect_x_start = body.x_start;
    let detect_x_end = detect_x_start + detect_width;

    // 检测边缘是否有不同颜色的像素带
    let border_width = detect_border_width(rgba, body.y_start, body.y_end, detect_x_start, detect_x_end);

    if border_width > 0 {
        // 提取边框颜色
        let border_color = extract_border_color(
            rgba,
            body.y_start,
            body.y_end,
            detect_x_start,
            detect_x_end,
            border_width,
        );

        (true, border_color, border_width, warnings)
    } else {
        warnings.push("未检测到边框".to_string());
        (
            false,
            RgbaColor {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            0,
            warnings,
        )
    }
}

/// 检测边框宽度（模糊化采样）
fn detect_border_width(
    rgba: &RgbaImage,
    y_start: u32,
    y_end: u32,
    x_start: u32,
    x_end: u32,
) -> u32 {
    // 采样 body 内部颜色（距离边缘 5px）
    let sample_x = (x_start + 5).min(x_end - 1);
    let center_y = (y_start + y_end) / 2;
    let body_color = sample_color_blur(rgba, sample_x, center_y, 20);

    // 从左边缘向右扫描
    let mut border_end = x_start;

    for x in x_start..x_end {
        let color = sample_color_blur(rgba, x, center_y, 20);

        // 检查颜色差异
        let diff = color_diff(body_color, color);
        if diff > 30 {
            // 阈值 30
            border_end = x + 1;
        } else {
            break;
        }
    }

    border_end - x_start
}

/// 模糊化采样（采样周围 N 个像素取平均）
fn sample_color_blur(rgba: &RgbaImage, x: u32, y: u32, radius: u32) -> [u8; 4] {
    let mut total_r = 0u64;
    let mut total_g = 0u64;
    let mut total_b = 0u64;
    let mut total_a = 0u64;
    let mut count = 0u64;

    for dy in -(radius as i32)..=(radius as i32) {
        for dx in -(radius as i32)..=(radius as i32) {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0 && ny >= 0 && (nx as u32) < rgba.width() && (ny as u32) < rgba.height() {
                let pixel = rgba.get_pixel(nx as u32, ny as u32);
                total_r += pixel[0] as u64;
                total_g += pixel[1] as u64;
                total_b += pixel[2] as u64;
                total_a += pixel[3] as u64;
                count += 1;
            }
        }
    }

    if count == 0 {
        return [0, 0, 0, 0];
    }

    [
        (total_r / count) as u8,
        (total_g / count) as u8,
        (total_b / count) as u8,
        (total_a / count) as u8,
    ]
}

/// 计算颜色差异
fn color_diff(c1: [u8; 4], c2: [u8; 4]) -> u32 {
    let dr = c1[0] as i32 - c2[0] as i32;
    let dg = c1[1] as i32 - c2[1] as i32;
    let db = c1[2] as i32 - c2[2] as i32;

    ((dr * dr + dg * dg + db * db) as f64).sqrt() as u32
}

/// 提取边框颜色
fn extract_border_color(
    rgba: &RgbaImage,
    y_start: u32,
    y_end: u32,
    x_start: u32,
    _x_end: u32,
    border_width: u32,
) -> RgbaColor {
    // 在边框区域中心采样
    let sample_x = x_start + border_width / 2;
    let center_y = (y_start + y_end) / 2;
    let color = sample_color_blur(rgba, sample_x, center_y, 20);

    RgbaColor {
        r: color[0],
        g: color[1],
        b: color[2],
        a: color[3],
    }
}
