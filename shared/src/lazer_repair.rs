/// Lazer 修复执行
///
/// 基于 `tail_repair`、`keyd_repair`、`skin_ini`、`image_utils` 模块，
/// 提供面尾和 Key 图片的批量 Lazer 适配修复。
use std::collections::HashSet;
use std::path::Path;

use crate::{backup, image_utils, keyd_repair, skin_ini, tail_repair};

/// @2x 后缀标记
fn is2x_tag(path: &Path) -> &'static str {
    if image_utils::is_2x(path) {
        " (@2x)"
    } else {
        ""
    }
}

/// 执行 Lazer 面尾修复（皮肤文件夹模式）。
///
/// 对所有 NoteImage#L 面尾图片执行修复算法。
///
/// # 返回
/// 日志行列表。
pub fn execute_lazer_tail_repair(
    skin_dir: &Path,
    backup_dir: &Path,
) -> Result<Vec<String>, String> {
    let ini_path = skin_dir.join("skin.ini");
    if !ini_path.exists() {
        return Err("未找到 skin.ini 文件".to_string());
    }

    let mut log: Vec<String> = Vec::new();
    let add_log = |log: &mut Vec<String>, msg: &str| log.push(msg.to_string());
    let ts_dir = backup::backup_timestamp();

    let skin_ini = skin_ini::parse_skin_ini(&ini_path)?;
    add_log(
        &mut log,
        &format!("解析到 {} 个 [Mania] 小节", skin_ini.mania_sections.len()),
    );

    let groups = skin_ini::group_note_image_l_by_stem(&skin_ini.mania_sections);
    add_log(&mut log, &format!("共 {} 组不同的面尾图片", groups.len()));

    let mut ini_patches: Vec<(u32, u32, String)> = Vec::new(); // (keys, col, new_stem)

    for (stem, refs) in &groups {
        let image_path = skin_ini::find_image_file(skin_dir, stem)
            .ok_or_else(|| format!("找不到面尾图片: {}", stem))?;

        let img = image::open(&image_path)
            .map_err(|e| format!("读取图片失败 {}: {}", stem, e))?
            .to_rgba8();

        let mut unique_cw: Vec<u32> = refs.iter().map(|(_, cw)| *cw).collect();
        unique_cw.sort();
        unique_cw.dedup();

        add_log(
            &mut log,
            &format!("处理: {} ({} 个键数/列宽组合)", stem, refs.len()),
        );

        if unique_cw.len() == 1 {
            let cw = unique_cw[0];
            let repaired = tail_repair::repair_tail_image(&img, cw);
            backup::backup_file(skin_dir, &image_path, backup_dir, &ts_dir)?;
            repaired
                .save(&image_path)
                .map_err(|e| format!("保存失败 {}: {}", stem, e))?;
            add_log(
                &mut log,
                &format!(
                    "  ✓ {}: cw={} → {}×{}{}",
                    stem,
                    cw,
                    repaired.width(),
                    repaired.height(),
                    is2x_tag(&image_path)
                ),
            );
        } else {
            let ext = image_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png");
            let first_cw = unique_cw[0];
            let repaired = tail_repair::repair_tail_image(&img, first_cw);
            backup::backup_file(skin_dir, &image_path, backup_dir, &ts_dir)?;
            repaired
                .save(&image_path)
                .map_err(|e| format!("保存失败 {}: {}", stem, e))?;
            add_log(
                &mut log,
                &format!(
                    "  ✓ {}: cw={} → {}×{}{}",
                    stem,
                    first_cw,
                    repaired.width(),
                    repaired.height(),
                    is2x_tag(&image_path)
                ),
            );

            for &cw in &unique_cw[1..] {
                let repaired_cw = tail_repair::repair_tail_image(&img, cw);
                let copy_stem = format!("{}_cw{}", stem, cw);
                let copy_path = skin_dir.join(format!("{}.{}", copy_stem, ext));
                repaired_cw
                    .save(&copy_path)
                    .map_err(|e| format!("保存副本失败 {}: {}", copy_stem, e))?;

                for (keys_k, _) in refs.iter().filter(|(_, c)| *c == cw) {
                    for sec in &skin_ini.mania_sections {
                        if sec.keys == *keys_k && sec.column_width == cw {
                            for r in &sec.note_image_ls {
                                if r.name == *stem {
                                    ini_patches.push((*keys_k, r.column, copy_stem.clone()));
                                }
                            }
                        }
                    }
                }
                add_log(
                    &mut log,
                    &format!(
                        "  ✓ {}: cw={} → {}×{} (副本: {}){}",
                        stem,
                        cw,
                        repaired_cw.width(),
                        repaired_cw.height(),
                        copy_stem,
                        is2x_tag(&copy_path)
                    ),
                );
            }
        }
    }

    // 统一应用 ini 修改
    for (keys, col, new_stem) in &ini_patches {
        skin_ini::update_note_image_l_in_ini(&ini_path, *keys, *col, new_stem)?;
        add_log(
            &mut log,
            &format!("  已更新 NoteImage{}L → {}", col, new_stem),
        );
    }

    add_log(&mut log, "面尾修复完成！");
    Ok(log)
}

/// 执行 Lazer Key 图片修复（皮肤文件夹模式）。
///
/// `mode`: "keyd" 只修 KeyImage#D，"key" 只修 KeyImage#，"all" 两者都修。
pub fn execute_lazer_key_repair(
    skin_dir: &Path,
    backup_dir: &Path,
    mode: &str,
) -> Result<Vec<String>, String> {
    let ini_path = skin_dir.join("skin.ini");
    if !ini_path.exists() {
        return Err("未找到 skin.ini 文件".to_string());
    }

    let mut log: Vec<String> = Vec::new();
    let add_log = |log: &mut Vec<String>, msg: &str| log.push(msg.to_string());
    let ts_dir = backup::backup_timestamp();

    let skin_ini = skin_ini::parse_skin_ini(&ini_path)?;

    let include_d = mode == "keyd" || mode == "all";
    let include_key = mode == "key" || mode == "all";

    let mut groups = skin_ini::group_key_images_by_stem(&skin_ini.mania_sections);
    if !include_d || !include_key {
        groups.retain(|stem, _| {
            let has_d = skin_ini
                .mania_sections
                .iter()
                .any(|sec| sec.key_image_ds.iter().any(|r| &r.name == stem));
            let has_key = skin_ini
                .mania_sections
                .iter()
                .any(|sec| sec.key_images.iter().any(|r| &r.name == stem));
            (include_d && has_d) || (include_key && has_key)
        });
    }

    add_log(&mut log, &format!("共 {} 组不同的 Key 图片", groups.len()));

    let mut ini_patches: Vec<(u32, u32, bool, String)> = Vec::new();

    for (stem, refs) in &groups {
        let image_path = match skin_ini::find_image_file(skin_dir, stem) {
            Some(p) => p,
            None => {
                add_log(&mut log, &format!("⚠ 找不到 Key 图片: {}", stem));
                continue;
            }
        };

        let img = image::open(&image_path)
            .map_err(|e| format!("读取图片失败 {}: {}", stem, e))?
            .to_rgba8();

        let mut unique_cw: Vec<u32> = refs.iter().map(|(_, cw)| *cw).collect();
        unique_cw.sort();
        unique_cw.dedup();

        let is_2x = image_utils::is_2x(&image_path);

        add_log(
            &mut log,
            &format!("处理: {} ({} 个键数/列宽组合)", stem, refs.len()),
        );

        if unique_cw.len() == 1 {
            let cw = unique_cw[0];
            let repaired = keyd_repair::repair_key_image(&img, cw, is_2x);
            backup::backup_file(skin_dir, &image_path, backup_dir, &ts_dir)?;
            repaired
                .save(&image_path)
                .map_err(|e| format!("保存失败 {}: {}", stem, e))?;
            add_log(
                &mut log,
                &format!(
                    "  ✓ {}: cw={} → {}×{}{}",
                    stem,
                    cw,
                    repaired.width(),
                    repaired.height(),
                    is2x_tag(&image_path)
                ),
            );
        } else {
            let ext = image_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png");
            let first_cw = unique_cw[0];
            let repaired = keyd_repair::repair_key_image(&img, first_cw, is_2x);
            backup::backup_file(skin_dir, &image_path, backup_dir, &ts_dir)?;
            repaired
                .save(&image_path)
                .map_err(|e| format!("保存失败 {}: {}", stem, e))?;
            add_log(
                &mut log,
                &format!(
                    "  ✓ {}: cw={} → {}×{}{}",
                    stem,
                    first_cw,
                    repaired.width(),
                    repaired.height(),
                    is2x_tag(&image_path)
                ),
            );

            for &cw in &unique_cw[1..] {
                let repaired_cw = keyd_repair::repair_key_image(&img, cw, is_2x);
                let copy_stem = format!("{}_cw{}", stem, cw);
                let copy_path = skin_dir.join(format!("{}.{}", copy_stem, ext));
                repaired_cw
                    .save(&copy_path)
                    .map_err(|e| format!("保存副本失败 {}: {}", copy_stem, e))?;

                for (keys_k, _) in refs.iter().filter(|(_, c)| *c == cw) {
                    for sec in &skin_ini.mania_sections {
                        if sec.keys == *keys_k && sec.column_width == cw {
                            for r in &sec.key_image_ds {
                                if r.name == *stem {
                                    ini_patches.push((
                                        *keys_k,
                                        r.column,
                                        true,
                                        copy_stem.clone(),
                                    ));
                                }
                            }
                            for r in &sec.key_images {
                                if r.name == *stem {
                                    ini_patches.push((
                                        *keys_k,
                                        r.column,
                                        false,
                                        copy_stem.clone(),
                                    ));
                                }
                            }
                        }
                    }
                }
                add_log(
                    &mut log,
                    &format!(
                        "  ✓ {}: cw={} → {}×{} (副本: {}){}",
                        stem,
                        cw,
                        repaired_cw.width(),
                        repaired_cw.height(),
                        copy_stem,
                        is2x_tag(&copy_path)
                    ),
                );
            }
        }
    }

    // 统一应用 ini 修改
    for (keys, col, is_d, new_stem) in &ini_patches {
        match skin_ini::update_key_image_in_ini(&ini_path, *keys, *col, *is_d, new_stem) {
            Ok(()) => {
                let label = if *is_d { "KeyImage#D" } else { "KeyImage#" };
                add_log(
                    &mut log,
                    &format!("  已更新 {}{} → {}", label, col, new_stem),
                );
            }
            Err(e) => add_log(&mut log, &format!("⚠ {}", e)),
        }
    }

    add_log(&mut log, "Key 图片修复完成！");
    Ok(log)
}

/// 执行 Lazer Key 图片修复（按 stem 过滤版本）。
///
/// 只修复 `stems` 中包含的 stem，其他与 `execute_lazer_key_repair` 一致。
/// 外部需传入已生成的时间戳目录名以避免重复创建。
pub fn execute_lazer_key_repair_filtered(
    skin_dir: &Path,
    backup_dir: &Path,
    stems: &HashSet<&str>,
    ts_dir: &str,
) -> Result<Vec<String>, String> {
    let ini_path = skin_dir.join("skin.ini");
    if !ini_path.exists() {
        return Err("未找到 skin.ini 文件".to_string());
    }

    let mut log: Vec<String> = Vec::new();
    let add_log = |log: &mut Vec<String>, msg: &str| log.push(msg.to_string());

    let skin_ini = skin_ini::parse_skin_ini(&ini_path)?;

    let mut groups = skin_ini::group_key_images_by_stem(&skin_ini.mania_sections);
    // 过滤：只保留用户勾选的 stem
    groups.retain(|stem, _| stems.contains(stem.as_str()));

    if groups.is_empty() {
        add_log(&mut log, "没有需要修复的 Key 图片");
        return Ok(log);
    }

    add_log(&mut log, &format!("共 {} 组 Key 图片待修复", groups.len()));

    let mut ini_patches: Vec<(u32, u32, bool, String)> = Vec::new();

    for (stem, refs) in &groups {
        let image_path = match skin_ini::find_image_file(skin_dir, stem) {
            Some(p) => p,
            None => {
                add_log(&mut log, &format!("⚠ 找不到 Key 图片: {}", stem));
                continue;
            }
        };

        let img = image::open(&image_path)
            .map_err(|e| format!("读取图片失败 {}: {}", stem, e))?
            .to_rgba8();

        let mut unique_cw: Vec<u32> = refs.iter().map(|(_, cw)| *cw).collect();
        unique_cw.sort();
        unique_cw.dedup();

        let is_2x = image_utils::is_2x(&image_path);

        add_log(
            &mut log,
            &format!("处理: {} ({} 个键数/列宽组合)", stem, refs.len()),
        );

        if unique_cw.len() == 1 {
            let cw = unique_cw[0];
            let repaired = keyd_repair::repair_key_image(&img, cw, is_2x);
            backup::backup_file(skin_dir, &image_path, backup_dir, ts_dir)?;
            repaired
                .save(&image_path)
                .map_err(|e| format!("保存失败 {}: {}", stem, e))?;
            add_log(
                &mut log,
                &format!(
                    "  ✓ {}: cw={} → {}×{}{}",
                    stem, cw, repaired.width(), repaired.height(), is2x_tag(&image_path)
                ),
            );
        } else {
            let ext = image_path.extension().and_then(|e| e.to_str()).unwrap_or("png");
            let first_cw = unique_cw[0];
            let repaired = keyd_repair::repair_key_image(&img, first_cw, is_2x);
            backup::backup_file(skin_dir, &image_path, backup_dir, ts_dir)?;
            repaired
                .save(&image_path)
                .map_err(|e| format!("保存失败 {}: {}", stem, e))?;
            add_log(
                &mut log,
                &format!(
                    "  ✓ {}: cw={} → {}×{}{}",
                    stem, first_cw, repaired.width(), repaired.height(), is2x_tag(&image_path)
                ),
            );

            for &cw in &unique_cw[1..] {
                let repaired_cw = keyd_repair::repair_key_image(&img, cw, is_2x);
                let copy_stem = format!("{}_cw{}", stem, cw);
                let copy_path = skin_dir.join(format!("{}.{}", copy_stem, ext));
                repaired_cw
                    .save(&copy_path)
                    .map_err(|e| format!("保存副本失败 {}: {}", copy_stem, e))?;

                for (keys_k, _) in refs.iter().filter(|(_, c)| *c == cw) {
                    for sec in &skin_ini.mania_sections {
                        if sec.keys == *keys_k && sec.column_width == cw {
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
                }
                add_log(
                    &mut log,
                    &format!(
                        "  ✓ {}: cw={} → {}×{} (副本: {}){}",
                        stem, cw, repaired_cw.width(), repaired_cw.height(), copy_stem, is2x_tag(&copy_path)
                    ),
                );
            }
        }
    }

    for (keys, col, is_d, new_stem) in &ini_patches {
        match skin_ini::update_key_image_in_ini(&ini_path, *keys, *col, *is_d, new_stem) {
            Ok(()) => {
                let label = if *is_d { "KeyImage#D" } else { "KeyImage#" };
                add_log(&mut log, &format!("  已更新 {}{} → {}", label, col, new_stem));
            }
            Err(e) => add_log(&mut log, &format!("⚠ {}", e)),
        }
    }

    add_log(&mut log, "Key 图片修复完成！");
    Ok(log)
}
