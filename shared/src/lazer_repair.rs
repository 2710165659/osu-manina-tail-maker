/// Lazer 修复执行
use std::collections::HashSet;
use std::path::Path;

use crate::{backup, image_utils, keyd_repair, skin_ini, tail_repair};

fn is2x_tag(path: &Path) -> &'static str {
    if image_utils::is_2x(path) { " (@2x)" } else { "" }
}

// ---------------------------------------------------------------------------
// 单一职责辅助函数
// ---------------------------------------------------------------------------

/// 对单个 stem 执行面尾修复。返回 `(log_lines, ini_patches)`。
/// `sections` 用于查找真正的 NoteImage 列号（非列宽）。
pub fn repair_one_tail_stem(
    skin_dir: &Path,
    stem: &str,
    refs: &[(u32, u32)],
    sections: &[skin_ini::ManiaSection],
    backup_dir: &Path,
    ts_dir: &str,
) -> Result<(Vec<String>, Vec<(u32, u32, String)>), String> {
    let mut log: Vec<String> = Vec::new();

    let image_path = skin_ini::find_image_file(skin_dir, stem)
        .ok_or_else(|| format!("找不到面尾图片: {}", stem))?;

    let img = image::open(&image_path)
        .map_err(|e| format!("读取图片失败 {}: {}", stem, e))?
        .to_rgba8();

    let unique_cw = unique_column_widths(refs);

    log.push(format!("处理: {} ({} 个键数/列宽组合)", stem, refs.len()));

    if unique_cw.len() == 1 {
        let cw = unique_cw[0];
        let repaired = tail_repair::repair_tail_image(&img, cw);
        backup::backup_file(skin_dir, &image_path, backup_dir, ts_dir)?;
        repaired.save(&image_path).map_err(|e| format!("保存失败 {}: {}", stem, e))?;
        log.push(format!("  ✓ {}: cw={} → {}×{}{}", stem, cw, repaired.width(), repaired.height(), is2x_tag(&image_path)));
        return Ok((log, Vec::new()));
    }

    let ext = image_path.extension().and_then(|e| e.to_str()).unwrap_or("png");
    let first_cw = unique_cw[0];
    let repaired = tail_repair::repair_tail_image(&img, first_cw);
    backup::backup_file(skin_dir, &image_path, backup_dir, ts_dir)?;
    repaired.save(&image_path).map_err(|e| format!("保存失败 {}: {}", stem, e))?;
    log.push(format!("  ✓ {}: cw={} → {}×{}{}", stem, first_cw, repaired.width(), repaired.height(), is2x_tag(&image_path)));

    let mut ini_patches: Vec<(u32, u32, String)> = Vec::new();

    for &cw in &unique_cw[1..] {
        let repaired_cw = tail_repair::repair_tail_image(&img, cw);
        let copy_stem = format!("{}_cw{}", stem, cw);
        let copy_path = skin_dir.join(format!("{}.{}", copy_stem, ext));
        repaired_cw.save(&copy_path).map_err(|e| format!("保存副本失败 {}: {}", copy_stem, e))?;

        for (keys_k, _) in refs.iter().filter(|(_, c)| *c == cw) {
            for sec in sections.iter().filter(|s| s.keys == *keys_k && s.column_width == cw) {
                for r in &sec.note_image_ls {
                    if r.name == *stem {
                        ini_patches.push((*keys_k, r.column, copy_stem.clone()));
                    }
                }
            }
        }

        log.push(format!("  ✓ {}: cw={} → {}×{} (副本: {}){}", stem, cw, repaired_cw.width(), repaired_cw.height(), copy_stem, is2x_tag(&copy_path)));
    }

    Ok((log, ini_patches))
}

/// 对单个 stem 执行 Key 图片修复。返回 `(log_lines, ini_patches)`。
/// `sections` 用于查找真正的 KeyImage 列号。
pub fn repair_one_key_stem(
    skin_dir: &Path,
    stem: &str,
    refs: &[(u32, u32)],
    sections: &[skin_ini::ManiaSection],
    backup_dir: &Path,
    ts_dir: &str,
) -> Result<(Vec<String>, Vec<(u32, u32, bool, String)>), String> {
    let mut log: Vec<String> = Vec::new();

    let image_path = match skin_ini::find_image_file(skin_dir, stem) {
        Some(p) => p,
        None => { log.push(format!("⚠ 找不到 Key 图片: {}", stem)); return Ok((log, Vec::new())); }
    };

    let img = image::open(&image_path)
        .map_err(|e| format!("读取图片失败 {}: {}", stem, e))?
        .to_rgba8();

    if image_utils::find_bounding_box(&img).is_none() {
        log.push(format!("  - {}: 全透明，跳过", stem));
        return Ok((log, Vec::new()));
    }

    let unique_cw = unique_column_widths(refs);
    let is_2x = image_utils::is_2x(&image_path);

    log.push(format!("处理: {} ({} 个键数/列宽组合)", stem, refs.len()));

    if unique_cw.len() == 1 {
        let cw = unique_cw[0];
        let repaired = keyd_repair::repair_key_image(&img, cw, is_2x);
        backup::backup_file(skin_dir, &image_path, backup_dir, ts_dir)?;
        repaired.save(&image_path).map_err(|e| format!("保存失败 {}: {}", stem, e))?;
        log.push(format!("  ✓ {}: cw={} → {}×{}{}", stem, cw, repaired.width(), repaired.height(), is2x_tag(&image_path)));
        return Ok((log, Vec::new()));
    }

    let ext = image_path.extension().and_then(|e| e.to_str()).unwrap_or("png");
    let first_cw = unique_cw[0];
    let repaired = keyd_repair::repair_key_image(&img, first_cw, is_2x);
    backup::backup_file(skin_dir, &image_path, backup_dir, ts_dir)?;
    repaired.save(&image_path).map_err(|e| format!("保存失败 {}: {}", stem, e))?;
    log.push(format!("  ✓ {}: cw={} → {}×{}{}", stem, first_cw, repaired.width(), repaired.height(), is2x_tag(&image_path)));

    let mut ini_patches: Vec<(u32, u32, bool, String)> = Vec::new();

    for &cw in &unique_cw[1..] {
        let repaired_cw = keyd_repair::repair_key_image(&img, cw, is_2x);
        let copy_stem = format!("{}_cw{}", stem, cw);
        let copy_path = skin_dir.join(format!("{}.{}", copy_stem, ext));
        repaired_cw.save(&copy_path).map_err(|e| format!("保存副本失败 {}: {}", copy_stem, e))?;

        for (keys_k, _) in refs.iter().filter(|(_, c)| *c == cw) {
            for sec in sections.iter().filter(|s| s.keys == *keys_k && s.column_width == cw) {
                for r in &sec.key_image_ds {
                    if r.name == *stem {
                        ini_patches.push((*keys_k, r.column, true, copy_stem.clone()));
                    }
                }
                for r in &sec.key_images {
                    if r.name == *stem {
                        ini_patches.push((*keys_k, r.column, false, copy_stem.clone()));
                    }
                }
            }
        }

        log.push(format!("  ✓ {}: cw={} → {}×{} (副本: {}){}", stem, cw, repaired_cw.width(), repaired_cw.height(), copy_stem, is2x_tag(&copy_path)));
    }

    Ok((log, ini_patches))
}

/// 应用面尾 skin.ini 补丁（合并输出，每 stem 一行）。
pub fn apply_tail_ini_patches(ini_path: &Path, patches: &[(u32, u32, String)]) -> Vec<String> {
    let mut log: Vec<String> = Vec::new();
    // Apply all patches first
    let mut by_stem: std::collections::HashMap<&str, Vec<(u32, u32)>> = std::collections::HashMap::new();
    for (keys, col, new_stem) in patches {
        match skin_ini::update_note_image_l_in_ini(ini_path, *keys, *col, new_stem) {
            Ok(()) => by_stem.entry(new_stem).or_default().push((*keys, *col)),
            Err(e) => log.push(format!("⚠ {}", e)),
        }
    }
    // Grouped output
    for (stem, refs) in by_stem {
        let mut keys: Vec<u32> = refs.iter().map(|(k, _)| *k).collect();
        keys.sort(); keys.dedup();
        let cols: Vec<String> = refs.iter().map(|(_, c)| c.to_string()).collect();
        log.push(format!("  已更新 {}k [{}] → {}", 
            keys.iter().map(|k| k.to_string()).collect::<Vec<_>>().join(","), 
            cols.join(","), stem));
    }
    log
}

/// 应用 Key skin.ini 补丁（合并输出，每 stem 一行）。
pub fn apply_key_ini_patches(ini_path: &Path, patches: &[(u32, u32, bool, String)]) -> Vec<String> {
    let mut log: Vec<String> = Vec::new();
    let mut by_stem: std::collections::HashMap<&str, Vec<(u32, u32, bool)>> = std::collections::HashMap::new();
    for (keys, col, is_d, new_stem) in patches {
        match skin_ini::update_key_image_in_ini(ini_path, *keys, *col, *is_d, new_stem) {
            Ok(()) => by_stem.entry(new_stem).or_default().push((*keys, *col, *is_d)),
            Err(e) => log.push(format!("⚠ {}", e)),
        }
    }
    for (stem, refs) in by_stem {
        let mut keys: Vec<u32> = refs.iter().map(|(k, _, _)| *k).collect();
        keys.sort(); keys.dedup();
        let key_cols: Vec<String> = refs.iter().filter(|(_, _, d)| !d).map(|(_, c, _)| c.to_string()).collect();
        let keyd_cols: Vec<String> = refs.iter().filter(|(_, _, d)| *d).map(|(_, c, _)| c.to_string()).collect();
        let mut parts = vec![];
        if !key_cols.is_empty() { parts.push(format!("Key[{}]", key_cols.join(","))); }
        if !keyd_cols.is_empty() { parts.push(format!("KeyD[{}]", keyd_cols.join(","))); }
        log.push(format!("  已更新 {}k {} → {}", 
            keys.iter().map(|k| k.to_string()).collect::<Vec<_>>().join(","),
            parts.join(" "), stem));
    }
    log
}

// ---------------------------------------------------------------------------
// 编排函数
// ---------------------------------------------------------------------------

pub fn execute_lazer_tail_repair(skin_dir: &Path, backup_dir: &Path) -> Result<Vec<String>, String> {
    let ini_path = skin_dir.join("skin.ini");
    if !ini_path.exists() { return Err("未找到 skin.ini 文件".to_string()); }

    let mut log: Vec<String> = Vec::new();
    let ts_dir = backup::backup_timestamp();

    let skin_ini = skin_ini::parse_skin_ini(&ini_path)?;
    log.push(format!("解析到 {} 个 [Mania] 小节", skin_ini.mania_sections.len()));

    let groups = skin_ini::group_note_image_l_by_stem(&skin_ini.mania_sections);
    log.push(format!("共 {} 组不同的面尾图片", groups.len()));

    let mut ini_patches: Vec<(u32, u32, String)> = Vec::new();

    for (stem, refs) in &groups {
        let (stem_log, patches) = repair_one_tail_stem(
            skin_dir, stem, refs, &skin_ini.mania_sections, backup_dir, &ts_dir,
        )?;
        log.extend(stem_log);
        ini_patches.extend(patches);
    }

    log.extend(apply_tail_ini_patches(&ini_path, &ini_patches));
    log.push("面尾修复完成！".to_string());
    Ok(log)
}

pub fn execute_lazer_key_repair(skin_dir: &Path, backup_dir: &Path, mode: &str) -> Result<Vec<String>, String> {
    let ini_path = skin_dir.join("skin.ini");
    if !ini_path.exists() { return Err("未找到 skin.ini 文件".to_string()); }

    let mut log: Vec<String> = Vec::new();
    let ts_dir = backup::backup_timestamp();

    let skin_ini = skin_ini::parse_skin_ini(&ini_path)?;

    let include_d = mode == "keyd" || mode == "all";
    let include_key = mode == "key" || mode == "all";

    let mut groups = skin_ini::group_key_images_by_stem(&skin_ini.mania_sections);
    if !include_d || !include_key {
        groups.retain(|stem, _| {
            let has_d = skin_ini.mania_sections.iter().any(|sec| sec.key_image_ds.iter().any(|r| &r.name == stem));
            let has_key = skin_ini.mania_sections.iter().any(|sec| sec.key_images.iter().any(|r| &r.name == stem));
            (include_d && has_d) || (include_key && has_key)
        });
    }

    log.push(format!("共 {} 组不同的 Key 图片", groups.len()));

    let mut ini_patches: Vec<(u32, u32, bool, String)> = Vec::new();

    for (stem, refs) in &groups {
        let (stem_log, patches) = repair_one_key_stem(
            skin_dir, stem, refs, &skin_ini.mania_sections, backup_dir, &ts_dir,
        )?;
        log.extend(stem_log);
        ini_patches.extend(patches);
    }

    log.extend(apply_key_ini_patches(&ini_path, &ini_patches));
    log.push("Key 图片修复完成！".to_string());
    Ok(log)
}

pub fn execute_lazer_key_repair_filtered(
    skin_dir: &Path,
    backup_dir: &Path,
    stems: &HashSet<&str>,
    ts_dir: &str,
) -> Result<Vec<String>, String> {
    let ini_path = skin_dir.join("skin.ini");
    if !ini_path.exists() { return Err("未找到 skin.ini 文件".to_string()); }

    let mut log: Vec<String> = Vec::new();
    let skin_ini = skin_ini::parse_skin_ini(&ini_path)?;

    let mut groups = skin_ini::group_key_images_by_stem(&skin_ini.mania_sections);
    groups.retain(|stem, _| stems.contains(stem.as_str()));

    if groups.is_empty() {
        log.push("没有需要修复的 Key 图片".to_string());
        return Ok(log);
    }

    log.push(format!("共 {} 组 Key 图片待修复", groups.len()));

    let mut ini_patches: Vec<(u32, u32, bool, String)> = Vec::new();

    for (stem, refs) in &groups {
        let (stem_log, patches) = repair_one_key_stem(
            skin_dir, stem, refs, &skin_ini.mania_sections, backup_dir, ts_dir,
        )?;
        log.extend(stem_log);
        ini_patches.extend(patches);
    }

    log.extend(apply_key_ini_patches(&ini_path, &ini_patches));
    log.push("Key 图片修复完成！".to_string());
    Ok(log)
}

// ---------------------------------------------------------------------------
fn unique_column_widths(refs: &[(u32, u32)]) -> Vec<u32> {
    let mut cws: Vec<u32> = refs.iter().map(|(_, cw)| *cw).collect();
    cws.sort();
    cws.dedup();
    cws
}