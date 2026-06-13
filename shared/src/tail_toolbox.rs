/// 工具箱编排入口
///
/// 统一编排一键修改面尾的三个步骤：
/// 1. Key/KeyD 修复（仅 lazer 模式）
/// 2. 预设替换（用预设图片覆盖面尾图片）
/// 3. 修改投长度（stable 或 lazer 含拉伸）
///
/// 供 Tauri commands.rs 和外部工具 converter.rs 共享使用。

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::{backup, lazer_repair, skin_ini, throw_info};

/// 执行工具箱全流程。
///
/// # 参数
/// - `skin_dir`: 皮肤根目录
/// - `work_mode`: "lazer" 或 "stable"
/// - `throws`: 键数 → 目标投长度
/// - `presets`: (stem, preset_name) 的列表，每个 stem 用哪个预设替换
/// - `keyd_stems`: 需要修复的 Key/KeyD stem 列表（仅 lazer 模式生效）
/// - `backup_dir`: 备份根目录
pub fn execute_toolbox(
    skin_dir: &Path,
    work_mode: &str,
    throws: &[(u32, u32)],
    presets: &[(String, String)],
    keyd_stems: &[String],
    backup_dir: &Path,
) -> Result<Vec<String>, String> {
    let mut log: Vec<String> = Vec::new();
    let ts_dir = backup::backup_timestamp();

    let throw_map: HashMap<u32, u32> = throws.iter().cloned().collect();
    let keyd_set: HashSet<&str> = keyd_stems.iter().map(|s| s.as_str()).collect();

    // ---------- Step 1: Key/KeyD 修复（仅 lazer） ----------
    if work_mode == "lazer" && !keyd_stems.is_empty() {
        log.push("--- Key/KeyD 修复 ---".to_string());
        match lazer_repair::execute_lazer_key_repair_filtered(
            skin_dir,
            backup_dir,
            &keyd_set,
            &ts_dir,
        ) {
            Ok(key_log) => log.extend(key_log),
            Err(e) => log.push(format!("Key 修复失败: {}", e)),
        }
    }

    // ---------- Step 2: 预设替换 ----------
    if !presets.is_empty() {
        log.push("--- 用预设替换现有图片 ---".to_string());
        let preset_map: HashMap<&str, &str> = presets
            .iter()
            .map(|(s, p)| (s.as_str(), p.as_str()))
            .collect();
        apply_presets_by_stem(skin_dir, &preset_map, &mut log)?;
    }

    // ---------- Step 3: 修改投长度 ----------
    log.push("--- 修改投长度 ---".to_string());

    // 收集 ColumnWidth 映射
    let ini_path = skin_dir.join("skin.ini");
    let skin_ini = skin_ini::parse_skin_ini(&ini_path)?;
    let mut column_widths: HashMap<u32, u32> = HashMap::new();
    for section in &skin_ini.mania_sections {
        if throw_map.contains_key(&section.keys) {
            column_widths.entry(section.keys).or_insert(section.column_width);
        }
    }

    match throw_info::execute_throw_modification(
        skin_dir,
        &throw_map,
        backup_dir,
        work_mode,
        &column_widths,
    ) {
        Ok(throw_log) => log.extend(throw_log),
        Err(e) => return Err(format!("投长度修改失败: {}", e)),
    }

    log.push("全部完成！".to_string());
    Ok(log)
}

/// 按 stem 应用预设：用预设图片覆盖对应面尾图片文件。
fn apply_presets_by_stem(
    skin_dir: &Path,
    presets: &HashMap<&str, &str>,
    log: &mut Vec<String>,
) -> Result<(), String> {
    let ini_path = skin_dir.join("skin.ini");
    let skin_ini = skin_ini::parse_skin_ini(&ini_path)?;
    let mut seen = HashSet::new();

    for section in &skin_ini.mania_sections {
        for r in &section.note_image_ls {
            let preset_name = match presets.get(r.name.as_str()) {
                Some(n) => n,
                None => continue,
            };

            if !seen.insert(r.name.clone()) {
                continue;
            }

            // 查找预设图片
            let preset_path = find_preset_image_file(skin_dir, preset_name)
                .ok_or_else(|| format!("找不到预设图片: {}", preset_name))?;

            let preset_img = image::open(&preset_path)
                .map_err(|e| format!("读取预设图片失败: {}", e))?
                .to_rgba8();

            // 找到原面尾图片
            let image_path = match skin_ini::find_image_file(skin_dir, &r.name) {
                Some(p) => p,
                None => {
                    log.push(format!("⚠ 找不到面尾图片: {}", r.name));
                    continue;
                }
            };

            preset_img
                .save(&image_path)
                .map_err(|e| format!("保存预设图片失败 {}: {}", r.name, e))?;

            log.push(format!("  ✓ {} ← {}", r.name, preset_name));
        }
    }

    Ok(())
}

/// 在皮肤目录下查找预设图片文件。
fn find_preset_image_file(skin_dir: &Path, name: &str) -> Option<std::path::PathBuf> {
    let exts = ["png", "jpg", "jpeg", "bmp", "webp", "gif"];
    let presets_dir = skin_dir.join("presets");

    for ext in &exts {
        let p = presets_dir.join(format!("{}.{}", name, ext));
        if p.exists() {
            return Some(p);
        }
    }

    for ext in &exts {
        let p = skin_dir.join(format!("{}.{}", name, ext));
        if p.exists() {
            return Some(p);
        }
    }

    None
}
