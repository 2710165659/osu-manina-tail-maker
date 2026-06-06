//! 边框效果
use crate::config::{RgbaColor, TailConfig};
use tiny_skia::Pixmap;

pub fn draw_border_layer(pixmap: &mut Pixmap, config: &TailConfig) {
    if !config.body.border_enabled || config.body.border_width == 0 { return; }

    let w = pixmap.width();
    let h = pixmap.height();
    let border_width = config.body.border_width;
    let (br, bg, bb) = border_rgb(config);

    let data = pixmap.data();
    let len = (w * h) as usize;
    let mask: Vec<u8> = (0..len).map(|i| if data[i * 4 + 3] > 0 { 255 } else { 0 }).collect();
    let orig_alpha: Vec<u8> = (0..len).map(|i| data[i * 4 + 3]).collect();
    let _ = data; // 释放不可变借用，后面需要 data_mut

    let mut eroded = mask.clone();
    erode_mask(&mut eroded, w, h, border_width);

    let border_opacity = config.body.border_opacity;
    let independent = config.body.border_opacity_independent;
    let data = pixmap.data_mut();
    for i in 0..len {
        if mask[i] == 0 || eroded[i] != 0 { continue; }
        let alpha = if independent { border_opacity } else { orig_alpha[i] };
        data[i * 4] = br;
        data[i * 4 + 1] = bg;
        data[i * 4 + 2] = bb;
        data[i * 4 + 3] = alpha;
    }
}

fn border_rgb(config: &TailConfig) -> (u8, u8, u8) {
    let bc: RgbaColor = if config.body.border_match_body {
        if config.body.independent_settings { config.body.color } else { config.global_color }
    } else {
        config.body.border_color
    };
    (bc.r, bc.g, bc.b)
}

/// 一维形态学腐蚀
fn erode_1d(buf: &mut [u8], length: usize, stride: usize, radius: usize) {
    let mut tmp = vec![0u8; length];
    for i in 0..length {
        let lo = i.saturating_sub(radius);
        let hi = (i + radius).min(length - 1);
        let mut min_val = 255u8;
        for j in lo..=hi {
            let v = buf[j * stride];
            if v < min_val { min_val = v; }
        }
        tmp[i] = min_val;
    }
    for i in 0..length {
        buf[i * stride] = tmp[i];
    }
}

/// 形态学腐蚀：分离式两遍（水平 + 垂直），前景向内收缩 radius 像素
fn erode_mask(mask: &mut [u8], w: u32, h: u32, radius: u32) {
    if radius == 0 { return; }
    let r = radius as usize;
    for y in 0..h as usize {
        let row_start = y * w as usize;
        erode_1d(&mut mask[row_start..row_start + w as usize], w as usize, 1, r);
    }
    for x in 0..w as usize {
        erode_1d(&mut mask[x..], h as usize, w as usize, r);
    }
}
