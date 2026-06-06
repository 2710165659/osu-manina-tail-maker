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
pub fn erode_mask(mask: &mut [u8], w: u32, h: u32, radius: u32) {
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
