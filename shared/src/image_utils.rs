/// 图像通用工具
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
