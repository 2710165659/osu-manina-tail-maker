/// Key Image 修复（KeyImage# / KeyImage#D 共用）
///
/// 算法：
/// 1. 找到主体元素包围矩形（四向扫描）
/// 2. 新建透明图片：宽 = ColumnWidth × 1.6，高 = 原图高
/// 3. 主体元素和距底/左右留白统一按 scale_factor = new_w / old_w 等比缩放
/// 4. 主体等比缩放后放入新图，底部对齐

use image::RgbaImage;

use crate::image_utils;

/// Key image 修复（同时用于 KeyImage# 和 KeyImage#D）
///
/// # 参数
/// - `img`: 原始 Key 图片
/// - `column_width`: skin.ini 中对应 [Mania] 的 ColumnWidth 值
/// - `is_2x`: 图片是否为 @2x 版本，是则目标宽度额外 ×2
pub fn repair_key_image(img: &RgbaImage, column_width: u32, is_2x: bool) -> RgbaImage {
    let (old_w, old_h) = img.dimensions();
    if old_w == 0 || old_h == 0 {
        return RgbaImage::new(1, 1);
    }

    let mut new_w = ((column_width as f64) * 1.6).round() as u32;
    if is_2x {
        new_w *= 2;
    }
    let new_w = new_w.max(1);
    let new_h = old_h;

    let bbox = match image_utils::find_bounding_box(img) {
        Some(b) => b,
        None => return RgbaImage::new(new_w, new_h),
    };

    let (left, top, right, bottom) = bbox;
    let element_w = right - left;
    let element_h = bottom - top;
    if element_w == 0 || element_h == 0 {
        return RgbaImage::new(new_w, new_h);
    }

    let pad_left = left;
    let _pad_right = old_w - right;
    let pad_bottom = old_h - bottom;

    // 统一等比缩放系数
    let sf = new_w as f64 / old_w as f64;

    let new_pad_left = (pad_left as f64 * sf).round() as u32;
    let new_pad_bottom = (pad_bottom as f64 * sf).round() as u32;
    let new_element_w = (element_w as f64 * sf).round() as u32;
    let new_element_h = (element_h as f64 * sf).round() as u32;

    // 防 0
    if new_element_w == 0 || new_element_h == 0 {
        return RgbaImage::new(new_w, new_h);
    }

    // 缩放主体元素（Lanczos3 — 放大平滑，缩小不撕裂）
    let element = image::imageops::crop_imm(img, left, top, element_w, element_h).to_image();
    let scaled_element = image::imageops::resize(
        &element,
        new_element_w,
        new_element_h,
        image::imageops::FilterType::Lanczos3,
    );

    // 底部对齐放置
    let mut result = RgbaImage::new(new_w, new_h);
    let new_top = new_h.saturating_sub(new_pad_bottom + new_element_h);

    for y in 0..new_element_h {
        let dst_y = new_top + y;
        if dst_y >= new_h {
            break;
        }
        for x in 0..new_element_w {
            let dst_x = new_pad_left + x;
            if dst_x >= new_w {
                break;
            }
            result.put_pixel(dst_x, dst_y, *scaled_element.get_pixel(x, y));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_enlarge() {
        // 50×50, element at (10,20)-(40,45) → 宽大 → pad/元素等比放大
        let mut img = RgbaImage::new(50, 50);
        for y in 20..45 {
            for x in 10..40 {
                img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let result = repair_key_image(&img, 40, false);
        // new_w = 40*1.6 = 64, sf = 64/50 = 1.28
        assert_eq!(result.width(), 64);
        assert_eq!(result.height(), 50);
        let bbox = image_utils::find_bounding_box(&result).unwrap();
        // 元素宽 = 30*1.28 ≈ 38
        assert!(bbox.2 - bbox.0 >= 36 && bbox.2 - bbox.0 <= 40);
    }

    #[test]
    fn test_shrink() {
        // 100×100, element at (20,30)-(80,70) → 窄小 → pad/元素等比缩小
        let mut img = RgbaImage::new(100, 100);
        for y in 30..70 {
            for x in 20..80 {
                img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let result = repair_key_image(&img, 30, false);
        // new_w = 30*1.6 = 48, sf = 48/100 = 0.48
        assert_eq!(result.width(), 48);
        assert_eq!(result.height(), 100);
        let bbox = image_utils::find_bounding_box(&result).unwrap();
        // 元素宽 = 60*0.48 ≈ 29, 距底 = 30*0.48 ≈ 14
        let new_pad_bottom = 100 - bbox.3;
        assert!(new_pad_bottom >= 12 && new_pad_bottom <= 18);
    }

    #[test]
    fn test_transparent() {
        let img = RgbaImage::new(50, 50);
        let result = repair_key_image(&img, 40, false);
        assert_eq!(result.width(), 64);
        assert_eq!(result.height(), 50);
    }

    #[test]
    fn test_2x_enlarge() {
        // @2x 版本：50×50, element (10,20)-(40,45), cw=40
        // new_w = (40*1.6)*2 = 128, sf = 128/50 = 2.56
        let mut img = RgbaImage::new(50, 50);
        for y in 20..45 {
            for x in 10..40 {
                img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let result = repair_key_image(&img, 40, true);
        assert_eq!(result.width(), 128);
        assert_eq!(result.height(), 50);
        let bbox = image_utils::find_bounding_box(&result).unwrap();
        // 元素宽 = 30*2.56 ≈ 77
        assert!(bbox.2 - bbox.0 >= 74 && bbox.2 - bbox.0 <= 80);
    }
}
