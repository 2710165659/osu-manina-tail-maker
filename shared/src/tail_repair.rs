/// Lazer 面尾修复
///
/// 核心算法：将面尾图片宽度缩放至 ColumnWidth × 1.6，高度等比缩放。
/// - 缩放后高 > 32800：裁切底部。
/// - 缩放后高 < 32800：取底部行平铺补足到 32800。

use crate::image_utils;
use image::RgbaImage;

/// 面尾图片修复的目标高度
pub const TAIL_TARGET_HEIGHT: u32 = 32800;

/// 单张面尾图片修复
///
/// # 参数
/// - `img`: 原始面尾图片
/// - `column_width`: skin.ini 中对应 [Mania] 的 ColumnWidth 值
///
/// # 返回
/// 修复后的面尾图片
pub fn repair_tail_image(img: &RgbaImage, column_width: u32) -> RgbaImage {
    let target_width = ((column_width as f64 * 1.6).round() as u32).max(1);

    // 等比缩放到目标宽度
    let scaled = image_utils::scale_to_width(img, target_width);
    let scaled_h = scaled.height();

    if scaled_h > TAIL_TARGET_HEIGHT {
        // 裁切底部
        image_utils::crop_to_height(&scaled, TAIL_TARGET_HEIGHT)
    } else if scaled_h < TAIL_TARGET_HEIGHT {
        // 取底部固定 1000 行平铺补足到 32800
        let tile_height = 1000.min(scaled_h);
        image_utils::tile_bottom_rows_to_height(&scaled, tile_height, TAIL_TARGET_HEIGHT)
    } else {
        // 正好 32800
        scaled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn test_repair_small_image() {
        // 创建一张 100×200 的小图片
        let img = RgbaImage::from_pixel(100, 200, image::Rgba([128, 128, 128, 255]));
        let result = repair_tail_image(&img, 60);
        // ColumnWidth 60 → target_width = 96
        assert_eq!(result.width(), 96);
        assert_eq!(result.height(), TAIL_TARGET_HEIGHT);
    }

    #[test]
    fn test_repair_height_exactly_32800() {
        // target_width = 101, scaled_h = 101/100 * 20000 = 20200 < 32800, should tile
        let img = RgbaImage::from_pixel(100, 20000, image::Rgba([128, 128, 128, 255]));
        // cw=63 → target_w=101, scaled_h=20200 → should tile to TAIL_TARGET_HEIGHT
        let result = repair_tail_image(&img, 63);
        assert_eq!(result.height(), TAIL_TARGET_HEIGHT);
    }
}
