/// 工具箱编排入口
///
/// 统一编排一键修改面尾的三个步骤：
/// 1. Key/KeyD 修复（仅 lazer 模式）
/// 2. 预设替换（用预设图片覆盖面尾图片）
/// 3. 修改投长度（stable 或 lazer 含拉伸）
///
/// 每个步骤均独立暴露为公共函数，供 commands 层按步骤调用并流式推送日志。
/// 本模块所有函数均为纯逻辑，不包含日志记录。

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::{backup, lazer_repair, skin_ini, throw_info};

// ---------------------------------------------------------------------------
// 独立步骤函数（供 commands 层逐个调用并推送事件）
// ---------------------------------------------------------------------------

/// 步骤 1：Key/KeyD 修复（仅 lazer 模式，需预生成 ts_dir）。
///
pub fn execute_key_repair_step(
    skin_dir: &Path,
    backup_dir: &Path,
    keyd_stems: &[String],
    ts_dir: &str,
) -> Result<Vec<String>, String> {
    if keyd_stems.is_empty() {
        return Ok(vec!["没有需要修复的 Key/KeyD 图片".to_string()]);
    }

    let keyd_set: HashSet<&str> = keyd_stems.iter().map(|s| s.as_str()).collect();
    lazer_repair::execute_lazer_key_repair_filtered(
        skin_dir,
        backup_dir,
        &keyd_set,
        ts_dir,
    )
}

/// 步骤 2：预设替换。
///
/// `presets`: `[(stem, preset_name)]` 的列表。
pub fn execute_preset_step(
    skin_dir: &Path,
    presets: &[(String, String)],
) -> Result<Vec<String>, String> {
    if presets.is_empty() {
        return Ok(vec!["没有选择预设替换".to_string()]);
    }

    let preset_map: HashMap<&str, &str> = presets
        .iter()
        .map(|(s, p)| (s.as_str(), p.as_str()))
        .collect();

    apply_presets_by_stem(skin_dir, &preset_map)
}

/// 步骤 3：修改投长度。
///
/// `throw_map`: 键数 → 目标投长度。
/// `column_widths`: 键数 → ColumnWidth（仅 lazer 需要）。
pub fn execute_throw_step(
    skin_dir: &Path,
    throw_map: &HashMap<u32, u32>,
    backup_dir: &Path,
    work_mode: &str,
    column_widths: &HashMap<u32, u32>,
) -> Result<Vec<String>, String> {
    if throw_map.is_empty() {
        return Ok(vec!["没有选择投长度修改".to_string()]);
    }

    throw_info::execute_throw_modification(
        skin_dir,
        throw_map,
        backup_dir,
        work_mode,
        column_widths,
    )
}

// ---------------------------------------------------------------------------
// 全流程编排（保留向后兼容）
// ---------------------------------------------------------------------------

/// 执行工具箱全流程（供外部工具 converter.rs 等使用）。
///
/// 推荐新代码使用独立步骤函数以获得更细粒度的控制。
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

    // ---------- Step 1: Key/KeyD 修复（仅 lazer） ----------
    if work_mode == "lazer" && !keyd_stems.is_empty() {
        log.push("--- Key/KeyD 修复 ---".to_string());
        match execute_key_repair_step(skin_dir, backup_dir, keyd_stems, &ts_dir) {
            Ok(key_log) => log.extend(key_log),
            Err(e) => log.push(format!("Key 修复失败: {}", e)),
        }
    }

    // ---------- Step 2: 预设替换 ----------
    if !presets.is_empty() {
        log.push("--- 用预设替换现有图片 ---".to_string());
        match execute_preset_step(skin_dir, presets) {
            Ok(preset_log) => log.extend(preset_log),
            Err(e) => log.push(format!("预设替换失败: {}", e)),
        }
    }

    // ---------- Step 3: 修改投长度 ----------
    log.push("--- 修改投长度 ---".to_string());

    let ini_path = skin_dir.join("skin.ini");
    let skin_ini = skin_ini::parse_skin_ini(&ini_path)?;
    let mut column_widths: HashMap<u32, u32> = HashMap::new();
    for section in &skin_ini.mania_sections {
        if throw_map.contains_key(&section.keys) {
            column_widths.entry(section.keys).or_insert(section.column_width);
        }
    }

    match execute_throw_step(skin_dir, &throw_map, backup_dir, work_mode, &column_widths) {
        Ok(throw_log) => log.extend(throw_log),
        Err(e) => return Err(format!("投长度修改失败: {}", e)),
    }

    log.push("全部完成！".to_string());
    Ok(log)
}

/// 解析 skin.ini 并收集 ColumnWidth 映射（供 commands 层在步骤 3 前调用）。
pub fn collect_column_widths(
    skin_dir: &Path,
    throw_map: &HashMap<u32, u32>,
) -> Result<HashMap<u32, u32>, String> {
    let ini_path = skin_dir.join("skin.ini");
    let skin_ini = skin_ini::parse_skin_ini(&ini_path)?;
    let mut column_widths: HashMap<u32, u32> = HashMap::new();
    for section in &skin_ini.mania_sections {
        if throw_map.contains_key(&section.keys) {
            column_widths.entry(section.keys).or_insert(section.column_width);
        }
    }
    Ok(column_widths)
}

// ---------------------------------------------------------------------------
// 预设替换内部实现
// ---------------------------------------------------------------------------

/// 按 stem 应用预设：用预设图片覆盖对应面尾图片文件。
fn apply_presets_by_stem(
    skin_dir: &Path,
    presets: &HashMap<&str, &str>,
) -> Result<Vec<String>, String> {
    let ini_path = skin_dir.join("skin.ini");
    let skin_ini = skin_ini::parse_skin_ini(&ini_path)?;
    let mut log: Vec<String> = Vec::new();
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

            match apply_one_preset(skin_dir, &r.name, preset_name) {
                Ok(msg) => log.push(msg),
                Err(msg) => log.push(msg),
            }
        }
    }

    Ok(log)
}

/// 对单个 stem 应用一个预设替换。
/// `preset_src` 可以是 base64 data URL（`data:image/png;base64,...`）或文件名/路径。
fn apply_one_preset(
    skin_dir: &Path,
    stem: &str,
    preset_src: &str,
) -> Result<String, String> {
    let preset_img = if preset_src.starts_with("data:") {
        // base64 data URL — 内置/用户预设
        decode_data_url(preset_src)
            .map_err(|e| format!("⚠ 解码预设图片失败: {}", e))?
    } else {
        // 文件路径或文件名 — 皮肤本地预设
        let preset_path = find_preset_image_file(skin_dir, preset_src)
            .ok_or_else(|| format!("⚠ 找不到预设图片: {}", preset_src))?;
        image::open(&preset_path)
            .map_err(|e| format!("⚠ 读取预设图片失败: {}", e))?
            .to_rgba8()
    };

    let image_path = match skin_ini::find_image_file(skin_dir, stem) {
        Some(p) => p,
        None => return Ok(format!("⚠ 找不到面尾图片: {}", stem)),
    };

    preset_img
        .save(&image_path)
        .map_err(|e| format!("⚠ 保存预设图片失败 {}: {}", stem, e))?;

    Ok(format!("  ✓ {}", stem))
}

/// 解码 base64 data URL 为 RGBA 图片。
fn decode_data_url(url: &str) -> Result<image::RgbaImage, String> {
    let b64 = url
        .split(',')
        .nth(1)
        .ok_or("无效的 data URL")?;
    let bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        b64,
    ).map_err(|e| format!("base64 解码失败: {}", e))?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("图片解码失败: {}", e))?;
    Ok(img.to_rgba8())
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