//! 外发光效果 - 使用几何分析实现高性能

use crate::config::{CapShape, TailConfig};
use tiny_skia::Pixmap;

/// 边缘点信息（用于颜色和透明度采样）
#[derive(Clone, Copy)]
struct EdgePoint {
    x: usize,
    y: usize,
    alpha: u8,
    r: u8,
    g: u8,
    b: u8,
}

/// 空间网格索引，用于加速最近邻搜索
struct SpatialGrid {
    cells: Vec<Vec<usize>>,
    cell_size: usize,
    grid_width: usize,
    grid_height: usize,
}

/// 图像区域的几何信息（用于优化扫描范围）
struct GeometryInfo {
    // Body矩形区域
    body_left: usize,
    body_right: usize,
    body_bottom: usize,
    // Echo区域（如果启用）
    echo_enabled: bool,
    echo_cap_top: usize,
    // 原始Cap区域
    cap_top: usize,
}

pub fn draw_glow_layer(pixmap: &mut Pixmap, config: &TailConfig) {
    if !config.effect.glow_enabled || config.effect.glow_size == 0 {
        return;
    }

    let w = pixmap.width() as usize;
    let h = pixmap.height() as usize;
    let glow_radius = config.effect.glow_size as usize;
    let glow_spread = config.effect.glow_spread as usize;
    let total_radius = glow_radius + glow_spread;

    // 提取配置
    let glow_color = config.effect.glow_color;
    let glow_opacity = config.effect.glow_opacity;
    let match_body = config.effect.glow_match_body;
    let independent = config.effect.glow_opacity_independent;

    // 计算几何信息
    let geometry = compute_geometry(config, w, h);

    // 提取alpha通道和扫描边缘点
    let data = pixmap.data();
    let mut alpha_mask = vec![0u8; w * h];
    let mut edge_points = Vec::new();

    for i in 0..(w * h) {
        alpha_mask[i] = data[i * 4 + 3];
    }

    // 扫描边缘点（只扫描可能的边缘区域以提高性能）
    scan_edge_points(&alpha_mask, data, w, h, &geometry, total_radius, &mut edge_points);

    // 如果没有边缘点，直接返回
    if edge_points.is_empty() {
        return;
    }

    // 构建空间索引以加速最近邻搜索
    let grid = build_spatial_grid(&edge_points, w, h, total_radius);

    // 应用外发光效果
    let data = pixmap.data_mut();

    // 只处理边缘附近的区域（减少计算量）
    let y_start = if geometry.echo_enabled {
        geometry.echo_cap_top.saturating_sub(total_radius)
    } else {
        geometry.cap_top.saturating_sub(total_radius)
    };
    let y_end = (geometry.body_bottom + total_radius).min(h);
    let x_start = geometry.body_left.saturating_sub(total_radius);
    let x_end = (geometry.body_right + total_radius).min(w);

    for y in y_start..y_end {
        for x in x_start..x_end {
            let idx = y * w + x;

            // 如果原图该像素有内容，跳过（外发光只在外部）
            if alpha_mask[idx] > 0 {
                continue;
            }

            // 计算到最近边缘的距离
            let (distance, nearest_edge) = compute_distance_to_edge(
                x, y, &geometry, &edge_points, &grid, total_radius
            );

            // 如果超出发光范围，跳过
            if distance > total_radius as f32 || distance <= 0.0 {
                continue;
            }

            // 计算渐变透明度
            let fade_factor = 1.0 - (distance / total_radius as f32);
            let fade_factor = fade_factor.max(0.0).min(1.0);

            // 获取颜色
            let (final_r, final_g, final_b) = if match_body {
                (nearest_edge.r, nearest_edge.g, nearest_edge.b)
            } else {
                (glow_color.r, glow_color.g, glow_color.b)
            };

            // 计算透明度
            let final_alpha = if independent {
                (fade_factor * glow_opacity as f32) as u8
            } else {
                (fade_factor * nearest_edge.alpha as f32) as u8
            };

            if final_alpha == 0 {
                continue;
            }

            // 写入发光颜色（预乘 alpha）
            let pixel_idx = idx * 4;
            let a_norm = final_alpha as f32 / 255.0;
            data[pixel_idx] = (final_r as f32 * a_norm).round().min(255.0) as u8;
            data[pixel_idx + 1] = (final_g as f32 * a_norm).round().min(255.0) as u8;
            data[pixel_idx + 2] = (final_b as f32 * a_norm).round().min(255.0) as u8;
            data[pixel_idx + 3] = final_alpha;
        }
    }
}

/// 计算图像的几何信息
fn compute_geometry(config: &TailConfig, _w: usize, _h: usize) -> GeometryInfo {
    let left = config.margin as usize;
    let right = (config.image.width - config.margin) as usize;
    let cap_h = config.cap_height() as usize;

    let echo_enabled = config.effect.cap_echo_enabled && config.cap.shape != CapShape::Gradient;
    let echo_cap_h = if echo_enabled { cap_h } else { 0 };
    let echo_start = config.throw_length as usize;
    let echo_cap_end = echo_start + echo_cap_h;
    let echo_rect_end = echo_cap_end + if echo_enabled { config.effect.echo_length as usize } else { 0 };

    let cap_start = echo_rect_end;
    let cap_end = cap_start + cap_h;
    let body_start = cap_end;
    let body_h = config.body_height() as usize;

    GeometryInfo {
        body_left: left,
        body_right: right,
        body_bottom: body_start + body_h,
        echo_enabled,
        echo_cap_top: echo_start,
        cap_top: cap_start,
    }
}

/// 扫描边缘点
fn scan_edge_points(
    alpha: &[u8],
    data: &[u8],
    w: usize,
    h: usize,
    geometry: &GeometryInfo,
    buffer: usize,
    edge_points: &mut Vec<EdgePoint>,
) {
    // 只扫描可能有边缘的区域（减少扫描范围）
    let y_start = if geometry.echo_enabled {
        geometry.echo_cap_top.saturating_sub(buffer)
    } else {
        geometry.cap_top.saturating_sub(buffer)
    };
    let y_end = (geometry.body_bottom + buffer).min(h);
    let x_start = geometry.body_left.saturating_sub(buffer);
    let x_end = (geometry.body_right + buffer).min(w);

    for y in y_start..y_end {
        for x in x_start..x_end {
            let idx = y * w + x;

            // 只有有内容的像素才可能是边缘
            if alpha[idx] == 0 {
                continue;
            }

            // 检查是否是边缘：8邻域有空白
            let is_border = (x > 0 && alpha[idx - 1] == 0)
                || (x + 1 < w && alpha[idx + 1] == 0)
                || (y > 0 && alpha[idx - w] == 0)
                || (y + 1 < h && alpha[idx + w] == 0)
                || (x > 0 && y > 0 && alpha[idx - w - 1] == 0)
                || (x + 1 < w && y > 0 && alpha[idx - w + 1] == 0)
                || (x > 0 && y + 1 < h && alpha[idx + w - 1] == 0)
                || (x + 1 < w && y + 1 < h && alpha[idx + w + 1] == 0);

            if is_border {
                let pixel_idx = idx * 4;
                let a = data[pixel_idx + 3];
                // 反预乘得到原始颜色
                let (r, g, b) = if a > 0 {
                    let scale = 255.0 / a as f32;
                    (
                        (data[pixel_idx] as f32 * scale).min(255.0) as u8,
                        (data[pixel_idx + 1] as f32 * scale).min(255.0) as u8,
                        (data[pixel_idx + 2] as f32 * scale).min(255.0) as u8,
                    )
                } else {
                    (0, 0, 0)
                };

                edge_points.push(EdgePoint { x, y, alpha: a, r, g, b });
            }
        }
    }
}

/// 计算点到边缘的最短距离（简化版：完全基于实际边缘点）
fn compute_distance_to_edge(
    x: usize,
    y: usize,
    _geometry: &GeometryInfo,
    edge_points: &[EdgePoint],
    grid: &SpatialGrid,
    max_dist: usize,
) -> (f32, EdgePoint) {
    let px = x as f32;
    let py = y as f32;

    // 直接在边缘点列表中找最近的点（使用空间索引加速）
    let mut min_dist = f32::MAX;
    let mut nearest = EdgePoint { x: 0, y: 0, alpha: 0, r: 0, g: 0, b: 0 };

    // 使用网格索引查找附近的边缘点
    let cell_indices = grid.query_nearby_cells(x, y, max_dist);

    for &cell_idx in &cell_indices {
        if cell_idx >= grid.cells.len() {
            continue;
        }

        for &edge_idx in &grid.cells[cell_idx] {
            if edge_idx >= edge_points.len() {
                continue;
            }

            let edge = &edge_points[edge_idx];
            let dx = px - edge.x as f32;
            let dy = py - edge.y as f32;

            let dist = (dx * dx + dy * dy).sqrt();
            if dist < min_dist {
                min_dist = dist;
                nearest = *edge;
            }
        }
    }

    (min_dist, nearest)
}

/// 构建空间网格索引
fn build_spatial_grid(edge_points: &[EdgePoint], w: usize, h: usize, cell_size: usize) -> SpatialGrid {
    let grid_width = (w + cell_size - 1) / cell_size;
    let grid_height = (h + cell_size - 1) / cell_size;
    let mut cells = vec![Vec::new(); grid_width * grid_height];

    for (idx, edge) in edge_points.iter().enumerate() {
        let cell_x = edge.x / cell_size;
        let cell_y = edge.y / cell_size;
        let cell_idx = cell_y * grid_width + cell_x;

        if cell_idx < cells.len() {
            cells[cell_idx].push(idx);
        }
    }

    SpatialGrid {
        cells,
        cell_size,
        grid_width,
        grid_height,
    }
}

impl SpatialGrid {
    /// 查询点附近的网格单元
    fn query_nearby_cells(&self, x: usize, y: usize, radius: usize) -> Vec<usize> {
        let mut result = Vec::new();

        let cell_x = x / self.cell_size;
        let cell_y = y / self.cell_size;

        let radius_in_cells = (radius + self.cell_size - 1) / self.cell_size;

        let x_start = cell_x.saturating_sub(radius_in_cells);
        let x_end = (cell_x + radius_in_cells + 1).min(self.grid_width);
        let y_start = cell_y.saturating_sub(radius_in_cells);
        let y_end = (cell_y + radius_in_cells + 1).min(self.grid_height);

        for cy in y_start..y_end {
            for cx in x_start..x_end {
                let cell_idx = cy * self.grid_width + cx;
                result.push(cell_idx);
            }
        }

        result
    }
}
