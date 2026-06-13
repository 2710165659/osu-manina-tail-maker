/// 图像通用工具
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use image::RgbaImage;

/// 从顶部开始查找第一个不透明像素所在行号（0-based）。
/// 全透明返回 None。
pub fn find_first_non_transparent_row(img: &RgbaImage) -> Option<u32> {
    let (w, h) = img.dimensions();
    for y in 0..h {
        for x in 0..w {
            if img.get_pixel(x, y)[3] > 0 {
                return Some(y);
            }
        }
    }
    None
}

/// 流式读取 PNG，从顶部逐行扫描，找到第一个不透明像素所在行号（0-based）。
/// 只在找到非透明行之前解码必要的行，避免解码整张大图。
/// 全透明返回 None。
pub fn find_first_non_transparent_row_streaming(path: &Path) -> Result<Option<u32>, String> {
    let file = File::open(path).map_err(|e| format!("打开文件失败: {}", e))?;
    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("读取 PNG 信息失败: {}", e))?;

    let info = reader.info();
    let color_type = info.color_type;
    let has_alpha = matches!(
        color_type,
        png::ColorType::Rgba | png::ColorType::GrayscaleAlpha
    );
    let bpp = info.bytes_per_pixel();

    let mut row = 0u32;

    while let Some(row_data) = reader
        .next_row()
        .map_err(|e| format!("读取 PNG 行失败: {}", e))?
    {
        let data = row_data.data();
        if has_alpha {
            let alpha_offset = bpp - 1;
            for x in (alpha_offset..data.len()).step_by(bpp) {
                if data[x] > 0 {
                    return Ok(Some(row));
                }
            }
        } else if bpp >= 3 {
            for chunk in data.chunks(bpp) {
                if chunk[0] > 0 || chunk[1] > 0 || chunk[2] > 0 {
                    return Ok(Some(row));
                }
            }
        } else {
            for &b in data {
                if b > 0 {
                    return Ok(Some(row));
                }
            }
        }
        row += 1;
    }

    Ok(None)
}

/// 流式读取 PNG 顶部 `max_rows` 行，返回 RgbaImage。
/// 用于生成预览图，避免解码全部行。
pub fn read_top_rows_streaming(path: &Path, max_rows: u32) -> Result<RgbaImage, String> {
    let file = File::open(path).map_err(|e| format!("打开文件失败: {}", e))?;
    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("读取 PNG 信息失败: {}", e))?;

    let info = reader.info();
    let width = info.width;
    let height = info.height;
    let crop_h = height.min(max_rows);
    let src_color = info.color_type;

    // Check for indexed color early
    if matches!(src_color, png::ColorType::Indexed) {
        drop(reader);
        let img = image::open(path)
            .map_err(|e| format!("读取索引颜色 PNG 失败: {}", e))?
            .to_rgba8();
        let (w, h) = img.dimensions();
        let crop_h = h.min(max_rows);
        let mut cropped = RgbaImage::new(w, crop_h);
        for y in 0..crop_h {
            for x in 0..w {
                cropped.put_pixel(x, y, *img.get_pixel(x, y));
            }
        }
        return Ok(cropped);
    }

    let mut img = RgbaImage::new(width, crop_h);
    let mut out_row = 0u32;

    while out_row < crop_h {
        match reader
            .next_row()
            .map_err(|e| format!("读取 PNG 行失败: {}", e))? {
            Some(row_data) => {
                let data = row_data.data();
                for x in 0..width as usize {
                    let pixel = match src_color {
                        png::ColorType::Rgba => {
                            let o = x * 4;
                            image::Rgba([data[o], data[o + 1], data[o + 2], data[o + 3]])
                        }
                        png::ColorType::Rgb => {
                            let o = x * 3;
                            image::Rgba([data[o], data[o + 1], data[o + 2], 255])
                        }
                        png::ColorType::GrayscaleAlpha => {
                            let o = x * 2;
                            image::Rgba([data[o], data[o], data[o], data[o + 1]])
                        }
                        png::ColorType::Grayscale => {
                            image::Rgba([data[x], data[x], data[x], 255])
                        }
                        png::ColorType::Indexed => unreachable!(),
                    };
                    img.put_pixel(x as u32, out_row, pixel);
                }
                out_row += 1;
            }
            None => break,
        }
    }

    Ok(img)
}

/// 找到图片中不透明主体元素的包围矩形 (left, top, right, bottom)，右下排他。
/// 全透明返回 None。
pub fn find_bounding_box(img: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let (w, h) = img.dimensions();
    let mut left = w;
    let mut top = h;
    let mut right = 0u32;
    let mut bottom = 0u32;
    let mut has_pixel = false;

    for y in 0..h {
        for x in 0..w {
            if img.get_pixel(x, y)[3] > 0 {
                has_pixel = true;
                if x < left {
                    left = x;
                }
                if y < top {
                    top = y;
                }
                if x + 1 > right {
                    right = x + 1;
                }
                if y + 1 > bottom {
                    bottom = y + 1;
                }
            }
        }
    }

    if has_pixel {
        Some((left, top, right, bottom))
    } else {
        None
    }
}

/// 等比缩放图片到目标宽度（高度等比变化）。
/// 使用 Lanczos3 滤波，放大平滑缩小不撕裂。
pub fn scale_to_width(img: &RgbaImage, target_width: u32) -> RgbaImage {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 || target_width == 0 {
        return RgbaImage::new(1, 1);
    }
    if w == target_width {
        return img.clone();
    }
    let target_height = (h as f64 * target_width as f64 / w as f64).round() as u32;
    image::imageops::resize(
        img,
        target_width,
        target_height.max(1),
        image::imageops::FilterType::Lanczos3,
    )
}

/// 从图片底部取 `tile_height` 行，平铺到指定总高度。
/// 如果 tile_height 为 0 则返回透明图。
/// 结果宽度与原图一致。
pub fn tile_bottom_rows_to_height(img: &RgbaImage, tile_height: u32, total_height: u32) -> RgbaImage {
    let (w, h) = img.dimensions();
    if w == 0 || total_height == 0 {
        return RgbaImage::new(1, 1);
    }

    let tile_h = tile_height.min(h);
    let mut result = RgbaImage::new(w, total_height);

    // 先复制原图的上半部分
    let copy_h = h.min(total_height);
    for y in 0..copy_h {
        for x in 0..w {
            result.put_pixel(x, y, *img.get_pixel(x, y));
        }
    }

    if tile_h == 0 || copy_h >= total_height {
        return result;
    }

    // 从底部 tile_h 行开始逐行平铺
    let tile_start = h - tile_h;
    let mut out_y = copy_h;
    while out_y < total_height {
        for ty in 0..tile_h {
            if out_y >= total_height {
                break;
            }
            let src_y = tile_start + ty;
            for x in 0..w {
                result.put_pixel(x, out_y, *img.get_pixel(x, src_y));
            }
            out_y += 1;
        }
    }

    result
}

/// 裁切图片到指定高度（从顶部保留）。
pub fn crop_to_height(img: &RgbaImage, target_height: u32) -> RgbaImage {
    let (w, h) = img.dimensions();
    if h <= target_height {
        return img.clone();
    }
    image::imageops::crop_imm(img, 0, 0, w, target_height).to_image()
}

/// 判断文件路径是否为 @2x 版本（文件名含 `@2x`）。
pub fn is_2x(path: &Path) -> bool {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.ends_with("@2x"))
        .unwrap_or(false)
}
