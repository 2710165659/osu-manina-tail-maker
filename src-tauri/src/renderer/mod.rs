mod cap;
mod body;

use cap::draw_cap;
use body::draw_body;

use crate::config::{CapShape, TailConfig};
use image::{ImageBuffer, Rgba, RgbaImage};

pub const PREVIEW_MAX_ROWS: u32 = 500;

/// 渲染参数：所有绘制所需信息预先计算，解耦各层
struct RenderLayout {
    w: u32,
    h: u32,
    left: u32,
    right: u32,
    cap_h: u32,
    echo_enabled: bool,
    echo_start: u32,
    echo_cap_end: u32,

    cap_start: u32,
    cap_end: u32,
    body_start: u32,
    body_h: u32,
}

impl RenderLayout {
    fn new(config: &TailConfig) -> Self {
        let w = config.image.width;
        let h = config.image.height;
        let left = config.margin;
        let right = w.saturating_sub(config.margin);
        let cap_h = config.cap_height();
        let echo_enabled = config.effect.cap_echo_enabled && config.cap.shape != CapShape::Gradient;
        let echo_cap_h = if echo_enabled { cap_h } else { 0 };
        let echo_start = config.throw_length;
        let echo_cap_end = echo_start + echo_cap_h;
        let echo_rect_end = echo_cap_end + if echo_enabled { config.effect.echo_length } else { 0 };
        let cap_start = echo_rect_end;
        let cap_end = cap_start + cap_h;
        let body_start = cap_end;
        let body_h = config.body_height();
        Self { w, h, left, right, cap_h, echo_enabled, echo_start, echo_cap_end, cap_start, cap_end, body_start, body_h }
    }
}

pub fn render(config: &TailConfig) -> RgbaImage {
    let layout = RenderLayout::new(config);
    let mut img: RgbaImage = ImageBuffer::from_pixel(layout.w, layout.h, Rgba([0, 0, 0, 0]));

    draw_echo_layer(&mut img, config, &layout);
    draw_cap_layer(&mut img, config, &layout);
    draw_body_layer(&mut img, config, &layout);
    draw_border_layer(&mut img, config);

    img
}

fn draw_echo_layer(img: &mut RgbaImage, config: &TailConfig, l: &RenderLayout) {
    if !l.echo_enabled || l.cap_h == 0 { return }
    let echo_cap_end = l.echo_cap_end.min(l.h);
    let right = l.right.min(l.w);
    if l.echo_start >= echo_cap_end || l.left >= right { return }

    let mut echo_layer: RgbaImage = ImageBuffer::from_pixel(l.w, l.h, Rgba([0, 0, 0, 0]));
    let echo_config = create_echo_config(config);
    draw_cap(&mut echo_layer, &echo_config, l.left, right, l.echo_start, echo_cap_end);

    let echo_color = config.effect.echo_color;
    let a = (echo_color.a as u32 * config.effect.echo_opacity as u32 * config.global_opacity as u32 / 65025) as u8;
    let px = Rgba([echo_color.r, echo_color.g, echo_color.b, a]);
    let fill_end = l.cap_end.min(l.h);
    for y in echo_cap_end..fill_end {
        for x in l.left..right {
            echo_layer.put_pixel(x, y, px);
        }
    }
    for y in l.echo_start..fill_end {
        for x in l.left..right {
            let ep = echo_layer.get_pixel(x, y);
            if ep[3] > 0 && img.get_pixel(x, y)[3] == 0 {
                img.put_pixel(x, y, *ep);
            }
        }
    }
}

fn draw_cap_layer(img: &mut RgbaImage, config: &TailConfig, l: &RenderLayout) {
    if l.cap_h > 0 {
        draw_cap(img, config, l.left, l.right, l.cap_start, l.cap_end);
    }
}

fn draw_body_layer(img: &mut RgbaImage, config: &TailConfig, l: &RenderLayout) {
    if l.body_h > 0 {
        draw_body(img, config, l.left, l.right, l.body_start, l.body_h);
    }
}



/// 一维形态学腐蚀：对行/列上每个像素取其半径 radius 邻域内的最小值
/// 边界处窗口截断（不镜像/不填充）
fn erode_1d(buf: &mut [u8], length: usize, stride: usize, radius: u32) {
    let r = radius as usize;
    let mut tmp = vec![0u8; length];
    for i in 0..length {
        let lo = i.saturating_sub(r);
        let hi = (i + r).min(length - 1);
        let mut min_val = 255u8;
        for j in lo..=hi {
            let v = buf[j * stride];
            if v < min_val {
                min_val = v;
            }
        }
        tmp[i] = min_val;
    }
    for i in 0..length {
        buf[i * stride] = tmp[i];
    }
}

/// 形态学腐蚀：分离式两遍（水平 + 垂直），时间复杂度 O(W×H)
/// mask 中非零像素视为前景，腐蚀后前景收缩 radius 像素
fn erode_mask(mask: &mut [u8], w: u32, h: u32, radius: u32) {
    if radius == 0 { return; }
    // 水平：每行独立腐蚀
    for y in 0..h as usize {
        let row_start = y * w as usize;
        erode_1d(&mut mask[row_start..row_start + w as usize], w as usize, 1, radius);
    }
    // 垂直：每列独立腐蚀
    for x in 0..w as usize {
        erode_1d(&mut mask[x..], h as usize, w as usize, radius);
    }
}

/// 边框渲染层：对合成图像中所有非透明像素的边缘做形态学腐蚀，
/// 腐蚀差集即为边框区域，用边框颜色替换
fn draw_border_layer(img: &mut RgbaImage, config: &TailConfig) {
    if !config.body.border_enabled || config.body.border_width == 0 { return; }

    let w = img.width();
    let h = img.height();
    let border_width = config.body.border_width;
    let (br, bg, bb) = if config.body.border_match_body {
        let bc = if config.body.independent_settings {
            config.body.color
        } else {
            config.global_color
        };
        (bc.r, bc.g, bc.b)
    } else {
        let bc = config.body.border_color;
        (bc.r, bc.g, bc.b)
    };

    // 1. 提取 alpha 通道作为二值 mask（>0 即前景）
    let len = (w * h) as usize;
    let mut mask: Vec<u8> = Vec::with_capacity(len);
    for pixel in img.pixels() {
        mask.push(if pixel[3] > 0 { 255 } else { 0 });
    }

    // 2. 形态学腐蚀：前景向内收缩 border_width 像素
    let mut eroded = mask.clone();
    erode_mask(&mut eroded, w, h, border_width);

    // 3. 差集 = 边框区域：原 mask 非零 且 腐蚀后为零
    //    同时根据透明度规则计算边框颜色
    let border_opacity = config.body.border_opacity;
    let independent = config.body.border_opacity_independent;

    for i in 0..len {
        if mask[i] == 0 || eroded[i] != 0 {
            continue;
        }
        let alpha = if independent {
            border_opacity
        } else {
            // 非独立：透明度 = 被挤占像素的透明度
            img.as_flat_samples().samples[i * 4 + 3]
        };
        let px = img.get_pixel_mut(i as u32 % w, i as u32 / w);
        *px = Rgba([br, bg, bb, alpha]);
    }
}

fn create_echo_config(config: &TailConfig) -> TailConfig {
    let mut echo_config = config.clone();
    echo_config.cap.color = config.effect.echo_color;
    echo_config.cap.independent_settings = true;
    echo_config.cap.opacity = (config.effect.echo_opacity as u16 * config.global_opacity as u16 / 255) as u8;
    let cap_h = config.cap_height();
    if cap_h > 0 {
        let scale_factor = config.effect.echo_length as f64 / cap_h as f64;
        echo_config.cap.scale = ((config.cap.scale as f64 * scale_factor) as u32).max(1);
    }
    echo_config
}

pub fn render_preview(config: &TailConfig) -> RgbaImage {
    let full = render(config);
    let h = full.height().min(PREVIEW_MAX_ROWS);
    let w = full.width();
    ImageBuffer::from_fn(w, h, |x, y| *full.get_pixel(x, y))
}
