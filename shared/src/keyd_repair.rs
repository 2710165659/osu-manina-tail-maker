/// Key Image 修复（KeyImage# / KeyImage#D 共用）
///
/// 算法：
/// 1. 找到主体元素包围矩形（四向扫描），得到 pad_left / pad_right / pad_bottom
/// 2. 新建透明图片：宽 = ColumnWidth × 1.6，主体等比缩放
/// 3. pad_left / pad_bottom 保持原像素值不变；主体区域宽高 × scale_factor
/// 4. 主体从 (pad_left, 0) 开始放置，底部自然留出 pad_bottom
/// 5. 画布高度 = 缩放后主体高度 + pad_bottom（顶部无留白，主体完整可见）

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

    let bbox = match image_utils::find_bounding_box(img) {
        Some(b) => b,
        None => {
            // 全透明：保持高度，宽度按目标
            return RgbaImage::new(new_w, old_h);
        }
    };

    let (left, top, right, bottom) = bbox;
    let element_w = right - left;
    let element_h = bottom - top;
    if element_w == 0 || element_h == 0 {
        return RgbaImage::new(new_w, old_h);
    }

    // 留白保持原像素值不变
    let pad_left = left;
    let pad_bottom = old_h - bottom;

    // 主体等比缩放系数
    let sf = new_w as f64 / old_w as f64;

    let new_element_w = (element_w as f64 * sf).round() as u32;
    let new_element_h = (element_h as f64 * sf).round() as u32;

    // 防 0
    if new_element_w == 0 || new_element_h == 0 {
        return RgbaImage::new(new_w, old_h);
    }

    // 画布高度 = 缩放后主体高度 + 底部留白（顶部无留白，主体完整可见）
    let new_h = new_element_h + pad_bottom;

    // 缩放主体元素（Lanczos3 — 放大平滑，缩小不撕裂）
    let element = image::imageops::crop_imm(img, left, top, element_w, element_h).to_image();
    let scaled_element = image::imageops::resize(
        &element,
        new_element_w,
        new_element_h,
        image::imageops::FilterType::Lanczos3,
    );

    // 放置：左上角 (pad_left, 0)，底部自然留出 pad_bottom
    let mut result = RgbaImage::new(new_w, new_h);
    for y in 0..new_element_h {
        let dst_y = y;
        for x in 0..new_element_w {
            let dst_x = pad_left + x;
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
        // old_w=50 old_h=50, element (10,20)-(40,45), cw=40
        // new_w=64, sf=1.28
        // pad_left=10 pad_bottom=5, element_w=30 element_h=25
        // new_element_w=38 new_element_h=32, new_h=32+5=37
        let mut img = RgbaImage::new(50, 50);
        for y in 20..45 {
            for x in 10..40 {
                img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let result = repair_key_image(&img, 40, false);
        assert_eq!(result.width(), 64);
        assert_eq!(result.height(), 37);
        let bbox = image_utils::find_bounding_box(&result).unwrap();
        // 元素宽 = 30*1.28 ≈ 38; pad_left = 10
        assert_eq!(bbox.0, 10, "pad_left 应保持 10");
        assert!(bbox.2 - bbox.0 >= 36 && bbox.2 - bbox.0 <= 40);
        // 底部留白 = 5
        assert_eq!(result.height() - bbox.3, 5, "pad_bottom 应保持 5");
    }

    #[test]
    fn test_shrink() {
        // old_w=100 old_h=100, element (20,30)-(80,70), cw=30
        // new_w=48, sf=0.48
        // pad_left=20 pad_bottom=30, element_w=60 element_h=40
        // new_element_w=29 new_element_h=19, new_h=19+30=49
        let mut img = RgbaImage::new(100, 100);
        for y in 30..70 {
            for x in 20..80 {
                img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let result = repair_key_image(&img, 30, false);
        assert_eq!(result.width(), 48);
        assert_eq!(result.height(), 49);
        let bbox = image_utils::find_bounding_box(&result).unwrap();
        // pad_left=20 不变
        assert_eq!(bbox.0, 20, "pad_left 应保持 20");
        // 元素宽 ≈ 29（因 20+29=49>48 右侧被裁 1px，bbox 右边界 = 48）
        assert!(bbox.2 - bbox.0 >= 27 && bbox.2 - bbox.0 <= 29);
        // pad_bottom=30 不变
        assert_eq!(result.height() - bbox.3, 30, "pad_bottom 应保持 30");
    }

    #[test]
    fn test_enlarge_no_top_padding() {
        // old_w=50 old_h=100, element (0,0)-(40,90), cw=40
        // new_w=64, sf=1.28
        // pad_left=0 pad_bottom=10, element_w=40 element_h=90
        // new_element_w=51 new_element_h=115, new_h=115+10=125
        let mut img = RgbaImage::new(50, 100);
        for y in 0..90 {
            for x in 0..40 {
                img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let result = repair_key_image(&img, 40, false);
        assert_eq!(result.width(), 64);
        assert_eq!(result.height(), 125);

        let bbox = image_utils::find_bounding_box(&result).unwrap();
        // pad_left=0 不变
        assert_eq!(bbox.0, 0, "pad_left 应保持 0");
        // 元素从顶部开始，完整可见，无裁切
        assert_eq!(bbox.1, 0, "元素应从顶部开始");
        // pad_bottom=10 不变
        assert_eq!(result.height() - bbox.3, 10, "pad_bottom 应保持 10");
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
        // @2x: old_w=50 old_h=50, element (10,20)-(40,45), cw=40
        // new_w=(40*1.6)*2=128, sf=2.56
        // pad_left=10 pad_bottom=5, element_w=30 element_h=25
        // new_element_w=77 new_element_h=64, new_h=64+5=69
        let mut img = RgbaImage::new(50, 50);
        for y in 20..45 {
            for x in 10..40 {
                img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let result = repair_key_image(&img, 40, true);
        assert_eq!(result.width(), 128);
        assert_eq!(result.height(), 69);
        let bbox = image_utils::find_bounding_box(&result).unwrap();
        // 元素宽 = 30*2.56 ≈ 77; pad_left=10 不变
        assert_eq!(bbox.0, 10, "pad_left 应保持 10");
        assert!(bbox.2 - bbox.0 >= 74 && bbox.2 - bbox.0 <= 80);
        // pad_bottom=5 不变
        assert_eq!(result.height() - bbox.3, 5, "pad_bottom 应保持 5");
    }
}
