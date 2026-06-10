/// 投长度修改
///
/// 投长度 = 从顶部开始到第一个不透明像素的行数。
///
/// 算法：
/// - 增大投长度：顶部增加透明行，内容下移。底部取原图底部行平铺补足，总高度不变。
/// - 减小投长度：顶部移除行，内容上移。底部取原图底部行平铺补足，总高度不变。

use crate::image_utils;
use image::RgbaImage;

/// 获取图片的当前投长度（首个不透明行距顶部距离）。
/// 全透明图片返回图片高度。
pub fn find_throw_length(img: &RgbaImage) -> u32 {
    image_utils::find_first_non_transparent_row(img).unwrap_or(img.height())
}

/// 校验面尾图片是否可用于修改投长度。
/// 规则：图片高度必须 > 5000。
/// 返回 (是否合规, 高度)
pub fn validate_tail_image(img: &RgbaImage) -> (bool, u32) {
    let h = img.height();
    (h > 5000, h)
}

/// 修改图片的投长度为目标值，总高度保持不变。
///
/// # 参数
/// - `img`: 原始图片
/// - `target_throw`: 目标投长度（像素行数）
///
/// # 返回
/// 修改后的图片，宽高不变。
pub fn modify_throw_length(img: &RgbaImage, target_throw: u32) -> RgbaImage {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return RgbaImage::new(1, 1);
    }

    let current_throw = find_throw_length(img);

    if target_throw == current_throw || h == 0 {
        return img.clone();
    }

    let mut result = RgbaImage::new(w, h);

    if target_throw > current_throw {
        // 增大投长度：顶部加透明行
        let shift = target_throw - current_throw;
        let content_rows = h.saturating_sub(shift);

        // 新图顶部 shift 行保持透明（已初始化为透明）
        // 从原图 current_throw 行开始，取 content_rows 行放到新图 target_throw 位置
        let copy_rows = content_rows.min(h - current_throw);
        for dy in 0..copy_rows {
            let src_y = current_throw + dy;
            let dst_y = target_throw + dy;
            if dst_y >= h {
                break;
            }
            for x in 0..w {
                result.put_pixel(x, dst_y, *img.get_pixel(x, src_y));
            }
        }
    } else {
        // 减小投长度：顶部移除行，内容上移
        let content_rows = h.saturating_sub(target_throw);

        for dy in 0..content_rows {
            let src_y = current_throw + dy;
            let dst_y = target_throw + dy;
            if src_y >= h || dst_y >= h {
                break;
            }
            for x in 0..w {
                result.put_pixel(x, dst_y, *img.get_pixel(x, src_y));
            }
        }
    }

    // 底部可能因移位产生空隙，取原图底部行平铺补足
    // 找出结果中最后一个有内容的行
    let result_throw = find_throw_length(&result);
    if result_throw >= h {
        return result;
    }

    // 内容结束位置：从 result_throw 开始有不透明像素
    // 找到最后一个不透明行
    let mut last_content_row = result_throw;
    for y in (result_throw..h).rev() {
        let mut has_pixel = false;
        for x in 0..w {
            if result.get_pixel(x, y)[3] > 0 {
                has_pixel = true;
                break;
            }
        }
        if has_pixel {
            last_content_row = y + 1;
            break;
        }
    }

    let bottom_gap = h.saturating_sub(last_content_row);
    if bottom_gap == 0 {
        return result;
    }

    // 取原图底部 bottom_gap 行平铺到结果底部
    let tile_h = bottom_gap.min(h);
    let tile_start = h - tile_h;
    let mut out_y = last_content_row;
    while out_y < h {
        for ty in 0..tile_h {
            if out_y >= h {
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

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_find_throw_length() {
        let mut img = RgbaImage::new(10, 50);
        // 在第 20 行放一个不透明像素
        img.put_pixel(5, 20, Rgba([255, 0, 0, 255]));
        assert_eq!(find_throw_length(&img), 20);

        let transparent = RgbaImage::new(10, 50);
        assert_eq!(find_throw_length(&transparent), 50);
    }

    #[test]
    fn test_increase_throw_length() {
        // 第 10 行开始有内容，高度 30
        let mut img = RgbaImage::new(10, 30);
        for y in 10..30 {
            for x in 0..10 {
                img.put_pixel(x, y, Rgba([128, 128, 128, 255]));
            }
        }
        let result = modify_throw_length(&img, 15);
        // 内容应该从第 15 行开始
        assert_eq!(result.dimensions(), (10, 30));
        assert_eq!(find_throw_length(&result), 15);
    }

    #[test]
    fn test_decrease_throw_length() {
        let mut img = RgbaImage::new(10, 30);
        for y in 10..30 {
            for x in 0..10 {
                img.put_pixel(x, y, Rgba([128, 128, 128, 255]));
            }
        }
        let result = modify_throw_length(&img, 5);
        assert_eq!(result.dimensions(), (10, 30));
        assert_eq!(find_throw_length(&result), 5);
    }

    #[test]
    fn test_decrease_tiles_from_bottom_not_top() {
        // 构造 5×20 图：顶部 5 行透明，然后 5 行红色(值=100)，底部 10 行蓝色(值=200)
        let mut img = RgbaImage::new(5, 20);
        // 透明顶部 0-4
        // 红色 5-9
        for y in 5..10 {
            for x in 0..5 {
                img.put_pixel(x, y, Rgba([100, 0, 0, 255]));
            }
        }
        // 蓝色 10-19
        for y in 10..20 {
            for x in 0..5 {
                img.put_pixel(x, y, Rgba([200, 0, 0, 255]));
            }
        }
        // current_throw = 5
        assert_eq!(find_throw_length(&img), 5);

        // 减小投长度到 0（移除所有顶部透明行，内容上移）
        let result = modify_throw_length(&img, 0);

        // 投长度应为 0
        assert_eq!(find_throw_length(&result), 0);
        // 结果底部区域应该来自原图底部（蓝色=200），不是顶部（红色=100）
        let bottom_pixel = result.get_pixel(0, 19);
        assert_eq!(bottom_pixel[0], 200, "底部应来自原图底部（蓝色=200），而非顶部");
    }
}
