use image::{GrayImage, Luma, RgbaImage, imageops};
use imageproc::contours::find_contours;
use imageproc::point::Point;
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

    // 打印文件名
    eprintln!("\n=====================================");
    eprintln!("解析图片: {:?}", image_path.file_name().unwrap_or_default());
    eprintln!("图片尺寸: {}x{}", width, height);
    eprintln!("=====================================");

    if width == 0 || height == 0 {
        return Err(ParseError::ImageTooSmall);
    }

    let mut warnings = Vec::new();

    // 1. 基本信息
    let filename = format!(
        "import-{}",
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

    // 4. 先检测 border 来确定内容宽度
    let (border_enabled, border_color, border_width, border_warnings) =
        detect_border(&rgba, &body_region);
    warnings.extend(border_warnings);

    // 计算去除边框后的实际内容宽度
    let content_width = if border_enabled {
        (body_region.x_end - body_region.x_start).saturating_sub(border_width * 2)
    } else {
        body_region.x_end - body_region.x_start
    };

    // 5. 检测 cap（传入实际内容宽度）
    let (cap_config, cap_warnings) = detect_cap(&rgba, &body_region, margin, content_width);
    warnings.extend(cap_warnings);

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

/// 形状检测结果
struct ShapeResult {
    shape: CapShape,
    aspect_ratio: f64, // 实际宽高比
}

/// 检测 cap 形状
fn detect_cap(rgba: &RgbaImage, body: &BodyRegion, _margin: u32, content_width: u32) -> (CapConfig, Vec<String>) {
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

    let cap_width = cap_x_end - cap_x_start;

    // 调试信息
    eprintln!("=== Cap 检测调试 ===");
    eprintln!("Cap 区域: y={}..{} (高度={}), x={}..{} (宽度={})",
        cap_y_start, cap_y_end, cap_height, cap_x_start, cap_x_end, cap_width);
    eprintln!("Body 区域: y={}..{}, x={}..{}",
        body.y_start, body.y_end, body.x_start, body.x_end);
    eprintln!("Content width (排除边框): {}", content_width);

    let shape_result = detect_shape(rgba, cap_y_start, cap_y_end, cap_x_start, cap_x_end);
    eprintln!("检测到的形状: {:?}", shape_result.shape);
    eprintln!("轮廓宽高比: {:.2}", shape_result.aspect_ratio);

    // 计算 scale
    // 矩形不需要缩放，只有球形和菱形才需要
    let scale = match shape_result.shape {
        CapShape::Rect => 100, // 矩形固定 100%
        CapShape::Ball => {
            // 球形：基于轮廓的实际宽高比计算
            // aspect_ratio = 实际宽度 / 实际高度
            // 如果是标准半圆，宽高比应该是 2:1
            // scale 表示相对于标准形状的拉伸比例
            if content_width > 0 && shape_result.aspect_ratio > 0.0 {
                // 期望的宽高比是 2:1 (半圆)
                let expected_ratio = 2.0;
                // 如果实际比例 < 2.0，说明被压扁了，需要拉伸
                // scale = (actual_height / expected_height) * 100
                // expected_height = content_width / expected_ratio
                // actual_height = cap_width / aspect_ratio
                let scale_factor = (cap_width as f64 / shape_result.aspect_ratio) / (content_width as f64 / expected_ratio);
                ((scale_factor * 100.0).round() as u32).clamp(10, 1000)
            } else {
                100
            }
        }
        CapShape::Diamond => {
            // 菱形：使用原来的计算方式
            if content_width > 0 {
                ((cap_height as f64 * 200.0 / content_width as f64).round() as u32).min(1000)
            } else {
                100
            }
        }
        CapShape::Gradient => {
            // 渐变：使用原来的计算方式
            if content_width > 0 {
                ((cap_height as f64 * 200.0 / content_width as f64).round() as u32).min(1000)
            } else {
                100
            }
        }
    };
    eprintln!("计算的缩放: {}%", scale);
    eprintln!("==================\n");

    (
        CapConfig {
            shape: shape_result.shape,
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

/// 检测形状（通过轮廓分析）
fn detect_shape(
    rgba: &RgbaImage,
    y_start: u32,
    y_end: u32,
    x_start: u32,
    x_end: u32,
) -> ShapeResult {
    let width = x_end - x_start;
    let height = y_end - y_start;

    if width == 0 || height == 0 {
        eprintln!("  [形状检测] 宽度或高度为0，返回矩形");
        return ShapeResult {
            shape: CapShape::Rect,
            aspect_ratio: 1.0,
        };
    }

    eprintln!("  [形状检测] Cap 原始尺寸: 宽={}, 高={}", width, height);

    // 1. 提取 cap 区域
    let cap_region = extract_region(rgba, y_start, y_end, x_start, x_end);

    // 2. 放大3倍以提高检测精度
    let scale_factor = 3;
    let upscaled = imageops::resize(
        &cap_region,
        width * scale_factor,
        height * scale_factor,
        imageops::FilterType::Lanczos3,
    );

    eprintln!("  [形状检测] 放大后尺寸: 宽={}, 高={}", upscaled.width(), upscaled.height());

    // 3. 转为二值图像
    let binary = binarize_rgba(&upscaled);

    // 4. 查找轮廓
    let contours = find_contours::<u32>(&binary);

    if contours.is_empty() {
        eprintln!("  [形状检测] 未找到轮廓，返回矩形");
        return ShapeResult {
            shape: CapShape::Rect,
            aspect_ratio: width as f64 / height as f64,
        };
    }

    // 找到最大的轮廓
    let main_contour = contours.iter()
        .max_by_key(|c| c.points.len())
        .unwrap();

    eprintln!("  [形状检测] 找到主轮廓，点数: {}", main_contour.points.len());

    // 5. 简化轮廓（RDP算法，epsilon也要按比例放大）
    let simplified = simplify_contour(&main_contour.points, 2.0 * scale_factor as f64);
    eprintln!("  [形状检测] 简化后点数: {}", simplified.len());

    // 6. 分析形状特征（使用原始尺寸）
    let (shape, aspect_ratio) = analyze_contour_shape(&simplified, width, height);

    eprintln!("  [形状检测] 判定为: {:?}, 宽高比: {:.2}", shape, aspect_ratio);

    ShapeResult {
        shape,
        aspect_ratio,
    }
}

/// 提取区域为 RgbaImage
fn extract_region(
    rgba: &RgbaImage,
    y_start: u32,
    y_end: u32,
    x_start: u32,
    x_end: u32,
) -> RgbaImage {
    let width = x_end - x_start;
    let height = y_end - y_start;

    let mut region = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let src_x = x_start + x;
            let src_y = y_start + y;

            if src_x < rgba.width() && src_y < rgba.height() {
                let pixel = rgba.get_pixel(src_x, src_y);
                region.put_pixel(x, y, *pixel);
            }
        }
    }

    region
}

/// 将 RgbaImage 二值化
fn binarize_rgba(rgba: &RgbaImage) -> GrayImage {
    let (width, height) = rgba.dimensions();
    let mut gray = GrayImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel = rgba.get_pixel(x, y);
            // 使用 alpha 通道进行二值化：alpha > 127 为白色（255），否则为黑色（0）
            let value = if pixel[3] > 127 { 255 } else { 0 };
            gray.put_pixel(x, y, Luma([value]));
        }
    }

    gray
}

/// Ramer-Douglas-Peucker 轮廓简化算法
fn simplify_contour(points: &[Point<u32>], epsilon: f64) -> Vec<(f64, f64)> {
    if points.is_empty() {
        return Vec::new();
    }

    // 转换为浮点坐标
    let pts: Vec<(f64, f64)> = points.iter()
        .map(|p| (p.x as f64, p.y as f64))
        .collect();

    if pts.len() <= 2 {
        return pts;
    }

    rdp_simplify(&pts, epsilon)
}

/// RDP 算法实现
fn rdp_simplify(points: &[(f64, f64)], epsilon: f64) -> Vec<(f64, f64)> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    // 找到距离线段最远的点
    let (start, end) = (points[0], points[points.len() - 1]);
    let mut max_dist = 0.0;
    let mut max_index = 0;

    for (i, &point) in points.iter().enumerate().skip(1).take(points.len() - 2) {
        let dist = perpendicular_distance(point, start, end);
        if dist > max_dist {
            max_dist = dist;
            max_index = i;
        }
    }

    // 如果最大距离大于阈值，递归简化
    if max_dist > epsilon {
        let left = rdp_simplify(&points[..=max_index], epsilon);
        let right = rdp_simplify(&points[max_index..], epsilon);

        let mut result = left;
        result.extend_from_slice(&right[1..]);
        result
    } else {
        vec![start, end]
    }
}

/// 计算点到线段的垂直距离
fn perpendicular_distance(point: (f64, f64), line_start: (f64, f64), line_end: (f64, f64)) -> f64 {
    let (x, y) = point;
    let (x1, y1) = line_start;
    let (x2, y2) = line_end;

    let dx = x2 - x1;
    let dy = y2 - y1;

    if dx == 0.0 && dy == 0.0 {
        return ((x - x1).powi(2) + (y - y1).powi(2)).sqrt();
    }

    let numerator = (dy * x - dx * y + x2 * y1 - y2 * x1).abs();
    let denominator = (dx.powi(2) + dy.powi(2)).sqrt();

    numerator / denominator
}

/// 分析轮廓形状，返回形状类型和实际宽高比
fn analyze_contour_shape(points: &[(f64, f64)], _width: u32, _height: u32) -> (CapShape, f64) {
    if points.len() < 4 {
        eprintln!("    [形状分析] 点数太少 ({}), 返回矩形", points.len());
        return (CapShape::Rect, 1.0);
    }

    // 计算轮廓的矩形度、圆形度、凸性等特征
    let (min_x, max_x, min_y, max_y) = bounding_box(points);
    let bbox_width = max_x - min_x;
    let bbox_height = max_y - min_y;

    // 计算实际宽高比
    let aspect_ratio = if bbox_height > 0.0 {
        bbox_width / bbox_height
    } else {
        1.0
    };

    eprintln!("    [形状分析] 边界框: 宽={:.1}, 高={:.1}, 宽高比={:.2}", bbox_width, bbox_height, aspect_ratio);

    // 计算轮廓面积（使用 Shoelace 公式）
    let contour_area = polygon_area(points);
    let bbox_area = bbox_width * bbox_height;
    let rectangularity = if bbox_area > 0.0 {
        contour_area / bbox_area
    } else {
        0.0
    };

    eprintln!("    [形状分析] 矩形度: {:.2} (>0.85为矩形)", rectangularity);

    // 矩形度高 -> 矩形
    if rectangularity > 0.85 {
        eprintln!("    [形状分析] 高矩形度，判定为矩形");
        return (CapShape::Rect, aspect_ratio);
    }

    // 计算圆形度（周长^2 / (4π * 面积)）
    let perimeter = polygon_perimeter(points);
    let circularity = if contour_area > 0.0 {
        (perimeter * perimeter) / (4.0 * std::f64::consts::PI * contour_area)
    } else {
        f64::MAX
    };

    eprintln!("    [形状分析] 圆形度: {:.2} (1.0为完美圆形，<1.5为椭圆)", circularity);

    // 圆形度接近1 -> 椭圆/球形
    if circularity < 1.5 {
        eprintln!("    [形状分析] 低圆形度值，判定为球形（椭圆）");
        return (CapShape::Ball, aspect_ratio);
    }

    // 检测是否为菱形（检查简化后的顶点数）
    let convex_vertices = detect_convex_vertices(points);
    eprintln!("    [形状分析] 凸顶点数: {}", convex_vertices);

    if convex_vertices == 4 && rectangularity < 0.7 && circularity > 1.5 {
        eprintln!("    [形状分析] 4个凸顶点且非矩形，判定为菱形");
        return (CapShape::Diamond, aspect_ratio);
    }

    // 默认根据形状特征判断
    let shape = if rectangularity > 0.7 {
        eprintln!("    [形状分析] 默认判定为矩形");
        CapShape::Rect
    } else if circularity < 2.0 {
        eprintln!("    [形状分析] 默认判定为球形");
        CapShape::Ball
    } else {
        eprintln!("    [形状分析] 默认判定为菱形");
        CapShape::Diamond
    };

    (shape, aspect_ratio)
}

/// 计算边界框
fn bounding_box(points: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;

    for &(x, y) in points {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }

    (min_x, max_x, min_y, max_y)
}

/// 计算多边形面积（Shoelace 公式）
fn polygon_area(points: &[(f64, f64)]) -> f64 {
    if points.len() < 3 {
        return 0.0;
    }

    let mut area = 0.0;
    let n = points.len();

    for i in 0..n {
        let j = (i + 1) % n;
        area += points[i].0 * points[j].1;
        area -= points[j].0 * points[i].1;
    }

    (area / 2.0).abs()
}

/// 计算多边形周长
fn polygon_perimeter(points: &[(f64, f64)]) -> f64 {
    if points.len() < 2 {
        return 0.0;
    }

    let mut perimeter = 0.0;
    let n = points.len();

    for i in 0..n {
        let j = (i + 1) % n;
        let dx = points[j].0 - points[i].0;
        let dy = points[j].1 - points[i].1;
        perimeter += (dx * dx + dy * dy).sqrt();
    }

    perimeter
}

/// 检测凸顶点数量
fn detect_convex_vertices(points: &[(f64, f64)]) -> usize {
    if points.len() < 3 {
        return points.len();
    }

    let n = points.len();
    let mut convex_count = 0;

    for i in 0..n {
        let prev = points[(i + n - 1) % n];
        let curr = points[i];
        let next = points[(i + 1) % n];

        // 计算叉积判断凸凹性
        let v1 = (curr.0 - prev.0, curr.1 - prev.1);
        let v2 = (next.0 - curr.0, next.1 - curr.1);
        let cross = v1.0 * v2.1 - v1.1 * v2.0;

        // 凸顶点的叉积为正（逆时针）或负（顺时针），取决于轮廓方向
        // 这里只统计显著的转角
        if cross.abs() > 0.1 {
            convex_count += 1;
        }
    }

    convex_count
}

/// 检测边框（检测左右两侧，取平均值）
fn detect_border(
    rgba: &RgbaImage,
    body: &BodyRegion,
) -> (bool, RgbaColor, u32, Vec<String>) {
    let mut warnings = Vec::new();

    let body_width = body.x_end - body.x_start;
    let detect_width = 100u32.min(body_width / 2);

    // 检测左侧边框
    let left_detect_x_start = body.x_start;
    let left_detect_x_end = left_detect_x_start + detect_width;
    let left_border_width = detect_border_width(
        rgba,
        body.y_start,
        body.y_end,
        left_detect_x_start,
        left_detect_x_end,
    );

    // 检测右侧边框
    let right_detect_x_end = body.x_end;
    let right_detect_x_start = right_detect_x_end.saturating_sub(detect_width);
    let right_border_width = detect_border_width_right(
        rgba,
        body.y_start,
        body.y_end,
        right_detect_x_start,
        right_detect_x_end,
    );

    // 取两侧的平均值或最大值
    let border_width = if left_border_width > 0 && right_border_width > 0 {
        (left_border_width + right_border_width) / 2
    } else {
        left_border_width.max(right_border_width)
    };

    if border_width > 0 {
        // 提取边框颜色（从左侧）
        let border_color = extract_border_color(
            rgba,
            body.y_start,
            body.y_end,
            left_detect_x_start,
            left_detect_x_end,
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

/// 检测边框宽度（从左侧，多位置采样）
fn detect_border_width(
    rgba: &RgbaImage,
    y_start: u32,
    y_end: u32,
    x_start: u32,
    x_end: u32,
) -> u32 {
    // 在多个垂直位置采样，取平均值
    let sample_positions = [
        y_start + (y_end - y_start) / 4,
        y_start + (y_end - y_start) / 2,
        y_start + (y_end - y_start) * 3 / 4,
    ];

    let mut border_widths = Vec::new();

    for &sample_y in &sample_positions {
        // 采样 body 内部颜色（距离边缘更远一些）
        let sample_x = (x_start + 10).min(x_end - 1);
        let body_color = sample_color_blur(rgba, sample_x, sample_y, 10);

        // 从左边缘向右扫描
        let mut border_end = x_start;

        for x in x_start..x_end {
            let color = sample_color_blur(rgba, x, sample_y, 5);

            // 检查颜色差异
            let diff = color_diff(body_color, color);
            if diff > 25 {
                // 降低阈值使其更敏感
                border_end = x + 1;
            } else {
                break;
            }
        }

        let width = border_end - x_start;
        if width > 0 && width < (x_end - x_start) / 2 {
            // 边框宽度应该合理（不超过一半宽度）
            border_widths.push(width);
        }
    }

    // 返回中位数
    if border_widths.is_empty() {
        0
    } else {
        border_widths.sort_unstable();
        border_widths[border_widths.len() / 2]
    }
}

/// 检测边框宽度（从右侧）
fn detect_border_width_right(
    rgba: &RgbaImage,
    y_start: u32,
    y_end: u32,
    x_start: u32,
    x_end: u32,
) -> u32 {
    // 在多个垂直位置采样，取平均值
    let sample_positions = [
        y_start + (y_end - y_start) / 4,
        y_start + (y_end - y_start) / 2,
        y_start + (y_end - y_start) * 3 / 4,
    ];

    let mut border_widths = Vec::new();

    for &sample_y in &sample_positions {
        // 采样 body 内部颜色（距离边缘更远一些）
        let sample_x = x_end.saturating_sub(10).max(x_start);
        let body_color = sample_color_blur(rgba, sample_x, sample_y, 10);

        // 从右边缘向左扫描
        let mut border_start = x_end;

        for x in (x_start..x_end).rev() {
            let color = sample_color_blur(rgba, x, sample_y, 5);

            // 检查颜色差异
            let diff = color_diff(body_color, color);
            if diff > 25 {
                border_start = x;
            } else {
                break;
            }
        }

        let width = x_end - border_start;
        if width > 0 && width < (x_end - x_start) / 2 {
            border_widths.push(width);
        }
    }

    // 返回中位数
    if border_widths.is_empty() {
        0
    } else {
        border_widths.sort_unstable();
        border_widths[border_widths.len() / 2]
    }
}

/// 模糊化采样（采样周围 N 个像素取平均）
fn sample_color_blur(rgba: &RgbaImage, x: u32, y: u32, radius: u32) -> [u8; 4] {
    let mut total_r = 0u64;
    let mut total_g = 0u64;
    let mut total_b = 0u64;
    let mut total_a = 0u64;
    let mut count = 0u64;

    let radius_i32 = radius as i32;

    for dy in -radius_i32..=radius_i32 {
        for dx in -radius_i32..=radius_i32 {
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

/// 提取边框颜色（多位置采样取平均）
fn extract_border_color(
    rgba: &RgbaImage,
    y_start: u32,
    y_end: u32,
    x_start: u32,
    _x_end: u32,
    border_width: u32,
) -> RgbaColor {
    // 在边框区域中心和多个垂直位置采样
    let sample_x = x_start + border_width / 2;
    let sample_positions = [
        y_start + (y_end - y_start) / 4,
        y_start + (y_end - y_start) / 2,
        y_start + (y_end - y_start) * 3 / 4,
    ];

    let mut total_r = 0u64;
    let mut total_g = 0u64;
    let mut total_b = 0u64;
    let mut total_a = 0u64;

    for &sample_y in &sample_positions {
        let color = sample_color_blur(rgba, sample_x, sample_y, 10);
        total_r += color[0] as u64;
        total_g += color[1] as u64;
        total_b += color[2] as u64;
        total_a += color[3] as u64;
    }

    let count = sample_positions.len() as u64;

    RgbaColor {
        r: (total_r / count) as u8,
        g: (total_g / count) as u8,
        b: (total_b / count) as u8,
        a: (total_a / count) as u8,
    }
}
