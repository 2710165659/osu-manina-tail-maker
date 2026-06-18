use crate::config::{Preset, TailConfig};
use crate::preset;
use crate::renderer;
use super::config_cmds::{cache_dir, config_hash, load_user_presets};
use base64::Engine;
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::events;

// ---------------------------------------------------------------------------
// 工具箱命令 —— 包装 shared 库，负责所有日志和事件推送
// ---------------------------------------------------------------------------

// ---- Tail repair (面尾修复, NoteImage#L) ----

/// Lazer 面尾修复（皮肤文件夹模式，始终备份）
#[tauri::command]
pub async fn repair_lazer_tail_folder(folder_path: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dir = PathBuf::from(&folder_path);
        if !dir.is_dir() {
            return Err("指定的路径不是有效的文件夹".to_string());
        }
        shared::logger::log_info("toolbox", "开始面尾修复...");
        let backup_dir = dir.join("_backup");
        let result = shared::lazer_repair::execute_lazer_tail_repair(&dir, &backup_dir)?;
        shared::logger::log_info("toolbox", "面尾修复完成");
        Ok(result)
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

// ---- Key image repair (KeyImage# + KeyImage#D) ----

/// Key 图片修复（皮肤文件夹模式，始终备份）
#[tauri::command]
pub async fn repair_key_image_folder(folder_path: String, mode: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dir = PathBuf::from(&folder_path);
        if !dir.is_dir() {
            return Err("指定的路径不是有效的文件夹".to_string());
        }
        shared::logger::log_info("toolbox", &format!("开始 Key 图片修复 (mode={})...", mode));
        let backup_dir = dir.join("_backup");
        let result = shared::lazer_repair::execute_lazer_key_repair(&dir, &backup_dir, &mode)?;
        shared::logger::log_info("toolbox", "Key 图片修复完成");
        Ok(result)
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

// ---- Throw length ----

/// 获取皮肤的投长度信息（按文件夹读取 skin.ini）
#[tauri::command]
pub async fn get_skin_throw_info(folder_path: String) -> Result<Vec<shared::throw_info::SkinThrowInfo>, String> {
    shared::throw_info::get_throw_info(Path::new(&folder_path))
}

/// 获取图片-键数-轨道关联信息（供预设替换列表）
#[tauri::command]
pub async fn get_image_key_info(folder_path: String) -> Result<Vec<shared::throw_info::ImageKeyInfo>, String> {
    shared::throw_info::get_image_key_info(Path::new(&folder_path))
}

/// 获取 Key/KeyD stem 列表（供 Key/KeyD 修复列表）
#[tauri::command]
pub async fn get_keyd_list(folder_path: String) -> Result<Vec<shared::throw_info::KeydStemInfo>, String> {
    shared::throw_info::get_keyd_list(Path::new(&folder_path))
}

// ---------------------------------------------------------------------------
// 工具箱"一键修改面尾" — fire-and-forget + 事件流式推送
// ---------------------------------------------------------------------------

static TOOLBOX_CANCELLED: AtomicBool = AtomicBool::new(false);

/// 工具箱"一键修改面尾"命令（fire-and-forget，通过 app:event 推送进度）。
///
/// 前端调用后立刻返回 Ok(())，监听 target="toolbox" 的 `app:event` 获取进度。
/// 每完成一个子步骤即推送一条事件，不再等全部结束才一次性返回。
#[tauri::command]
pub async fn convert_tail_toolbox(
    app: tauri::AppHandle,
    folder_path: String,
    _skin_mode: String,
    work_mode: String,
    throws: Vec<(u32, u32)>,
    presets: Vec<(String, String)>,
    keyd_stems: Vec<String>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        TOOLBOX_CANCELLED.store(false, Ordering::SeqCst);

        let skin_dir = PathBuf::from(&folder_path);
        if !skin_dir.is_dir() {
            events::emit_log(&app, "error", "toolbox", "指定的路径不是有效的文件夹");
            events::emit_log(&app, "done", "toolbox", "任务异常终止");
            return;
        }
        let backup_dir = skin_dir.join("_backup");

        events::emit_log(&app, "info", "toolbox", "开始一键修改面尾...");
        shared::logger::log_info("toolbox", "开始一键修改面尾...");

        let throw_map: HashMap<u32, u32> = throws.iter().cloned().collect();
        let keyd_stems: Vec<String> = keyd_stems;

        // ---- Step 1: Key/KeyD 修复（仅 lazer） ----
        if work_mode == "lazer" && !keyd_stems.is_empty() {
            if TOOLBOX_CANCELLED.load(Ordering::SeqCst) {
                events::emit_log(&app, "done", "toolbox", "任务已取消");
                shared::logger::log_info("toolbox", "任务已取消");
                return;
            }
            events::emit_log(&app, "info", "toolbox", "--- Key/KeyD 修复 ---");
            shared::logger::log_info("toolbox", "--- Key/KeyD 修复 ---");

            let ts_dir = shared::backup::backup_timestamp();
            match shared::tail_toolbox::execute_key_repair_step(
                &skin_dir, &backup_dir, &keyd_stems, &ts_dir,
            ) {
                Ok(key_log) => {
                    for line in &key_log {
                        events::emit_log(&app, "info", "toolbox", line);
                    }
                    shared::logger::log_info("toolbox", "Key/KeyD 修复完成");
                    events::emit_log(&app, "success", "toolbox", "Key/KeyD 修复完成");
                }
                Err(e) => {
                    let msg = format!("Key 修复失败: {}", e);
                    events::emit_log(&app, "error", "toolbox", &msg);
                    shared::logger::log_error("toolbox", &msg);
                }
            }
        }

        // ---- Step 2: 预设替换 ----
        if !presets.is_empty() {
            if TOOLBOX_CANCELLED.load(Ordering::SeqCst) {
                events::emit_log(&app, "done", "toolbox", "任务已取消");
                shared::logger::log_info("toolbox", "任务已取消");
                return;
            }
            events::emit_log(&app, "info", "toolbox", "--- 用预设替换现有图片 ---");
            shared::logger::log_info("toolbox", "--- 用预设替换现有图片 ---");

            // 解析预设名 → 全尺寸图片源（base64 data URL 或文件路径）
            let app_presets = {
                let builtin = preset::builtin_presets();
                let user = load_user_presets(&app);
                let builtin_names: std::collections::HashSet<&str> =
                    builtin.iter().map(|p| p.name.as_str()).collect();
                let user_filtered: Vec<Preset> = user.into_iter()
                    .filter(|p| !builtin_names.contains(p.name.as_str()))
                    .collect();
                [builtin, user_filtered].concat()
            };

            let resolved: Vec<(String, String)> = presets.iter().map(|(stem, name)| {
                // 1) 皮肤目录下查找文件
                for ext in &["png", "jpg", "jpeg"] {
                    let p = skin_dir.join("presets").join(format!("{}.{}", name, ext));
                    if p.exists() { return (stem.clone(), p.to_string_lossy().to_string()); }
                    let p = skin_dir.join(format!("{}.{}", name, ext));
                    if p.exists() { return (stem.clone(), p.to_string_lossy().to_string()); }
                }
                // 2) 内置/用户预设 → 渲染全尺寸图 → base64
                if let Some(preset) = app_presets.iter().find(|p| p.name == *name) {
                    let img = renderer::render(&preset.config);
                    let mut png = Vec::new();
                    let mut cursor = std::io::Cursor::new(&mut png);
                    image::DynamicImage::ImageRgba8(img)
                        .write_to(&mut cursor, image::ImageFormat::Png).ok();
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
                    return (stem.clone(), format!("data:image/png;base64,{}", b64));
                }
                // 3) 兜底：传原名（shared 层会尝试在皮肤目录查找）
                (stem.clone(), name.clone())
            }).collect();

            match shared::tail_toolbox::execute_preset_step(
                &skin_dir, &resolved, &backup_dir,
                &shared::backup::backup_timestamp(),
            ) {
                Ok(preset_log) => {
                    for line in &preset_log {
                        events::emit_log(&app, "info", "toolbox", line);
                    }
                    events::emit_log(&app, "success", "toolbox", "预设替换完成");
                    shared::logger::log_info("toolbox", "预设替换完成");
                }
                Err(e) => {
                    let msg = format!("预设替换失败: {}", e);
                    events::emit_log(&app, "error", "toolbox", &msg);
                    shared::logger::log_error("toolbox", &msg);
                }
            }
        }

        // ---- Step 3: 修改投长度（逐 stem） ----
        if TOOLBOX_CANCELLED.load(Ordering::SeqCst) {
            events::emit_log(&app, "done", "toolbox", "任务已取消");
            shared::logger::log_info("toolbox", "任务已取消");
            return;
        }
        events::emit_log(&app, "info", "toolbox", "--- 修改投长度 ---");
        shared::logger::log_info("toolbox", "--- 修改投长度 ---");

        let ini_path = skin_dir.join("skin.ini");
        let skin_ini = match shared::skin_ini::parse_skin_ini(&ini_path) {
            Ok(s) => s,
            Err(e) => {
                events::emit_log(&app, "error", "toolbox", &format!("解析 skin.ini 失败: {}", e));
                shared::logger::log_error("toolbox", &format!("解析 skin.ini 失败: {}", e));
                events::emit_log(&app, "done", "toolbox", "任务异常终止");
                return;
            }
        };

        let throw_ts_dir = shared::backup::backup_timestamp();
        let mut first_seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut done_pairs: std::collections::HashSet<(String, u32)> = std::collections::HashSet::new();
        let mut processed = false;

        for section in &skin_ini.mania_sections {
            let target_throw = match throw_map.get(&section.keys) {
                Some(&t) => t,
                None => continue,
            };

            let cw = section.column_width;

            for r in &section.note_image_ls {
                let pair = (r.name.clone(), cw);
                if !done_pairs.insert(pair) { continue; }
                if TOOLBOX_CANCELLED.load(Ordering::SeqCst) {
                    events::emit_log(&app, "done", "toolbox", "任务已取消");
                    return;
                }

                if !first_seen.contains(&r.name) {
                    // 首个 cw：直接修改原图
                    first_seen.insert(r.name.clone());
                    match shared::throw_info::modify_one_throw_stem(
                        &skin_dir, &r.name, section.keys, target_throw,
                        &work_mode, cw, &backup_dir, &throw_ts_dir,
                    ) {
                        Ok(stem_log) => {
                            for line in &stem_log {
                                events::emit_log(&app, "info", "toolbox", line);
                            }
                            processed = true;
                        }
                        Err(e) => {
                            events::emit_log(&app, "error", "toolbox", &format!("{}: {}", r.name, e));
                        }
                    }
                } else {
                    // 不同 cw：生成副本，修改并更新 skin.ini
                    let copy_stem = format!("{}_cw{}", r.name, cw);
                    let image_path = match shared::skin_ini::find_image_file(&skin_dir, &r.name) {
                        Some(p) => p,
                        None => {
                            events::emit_log(&app, "warning", "toolbox", &format!("⚠ 找不到原图: {}", r.name));
                            continue;
                        }
                    };
                    let ext = image_path.extension().and_then(|e| e.to_str()).unwrap_or("png");
                    let copy_path = skin_dir.join(format!("{}.{}", copy_stem, ext));

                    // 读原图 → 修改 → 保存副本
                    match image::open(&image_path) {
                        Ok(img) => {
                            let rgba = img.to_rgba8();
                            let current_throw = shared::throw_length::find_throw_length(&rgba);
                            let modified = if work_mode == "lazer" {
                                let h = rgba.height();
                                let cur_lazer = if h > 0 { ((current_throw as u64 * 32800) / h as u64) as u32 } else { 0 };
                                events::emit_log(&app, "info", "toolbox", &format!(
                                    "{} {}k: 投长度 {}px → {}px (Lazer, cw={}, 副本: {})",
                                    image_path.display(), section.keys, cur_lazer, target_throw, cw, copy_stem
                                ));
                                shared::throw_length::modify_throw_length_lazer(&rgba, target_throw, cw)
                            } else {
                                events::emit_log(&app, "info", "toolbox", &format!(
                                    "{} {}k: 投长度 {}px → {}px (Stable, 副本: {})",
                                    image_path.display(), section.keys, current_throw, target_throw, copy_stem
                                ));
                                shared::throw_length::modify_throw_length(&rgba, target_throw)
                            };
                            if let Err(e) = modified.save(&copy_path) {
                                events::emit_log(&app, "error", "toolbox", &format!("保存副本失败 {}: {}", copy_stem, e));
                            }
                        }
                        Err(e) => {
                            events::emit_log(&app, "error", "toolbox", &format!("读取原图失败 {}: {}", r.name, e));
                            continue;
                        }
                    }

                    // 更新 skin.ini：该 section 下所有引用此 stem 的列
                    for sec_r in &section.note_image_ls {
                        if sec_r.name == r.name {
                            match shared::skin_ini::update_note_image_l_in_ini(
                                &ini_path, section.keys, sec_r.column, &copy_stem,
                            ) {
                                Ok(()) => {
                                    events::emit_log(&app, "info", "toolbox", &format!(
                                        "  {}k NoteImage{}L → {}", section.keys, sec_r.column, copy_stem
                                    ));
                                }
                                Err(e) => {
                                    events::emit_log(&app, "warning", "toolbox", &format!("⚠ {}", e));
                                }
                            }
                        }
                    }
                    processed = true;
                }
            }
        }

        if !processed {
            events::emit_log(&app, "info", "toolbox", "未找到匹配的键数小节");
        }
        events::emit_log(&app, "success", "toolbox", "投长度修改完成");
        shared::logger::log_info("toolbox", "投长度修改完成");

        events::emit_log(&app, "done", "toolbox", "全部完成！");
        shared::logger::log_info("toolbox", "一键修改面尾完成");
    }).await.map_err(|e| format!("任务执行失败: {}", e))?;

    Ok(())
}

/// 取消正在进行的工具箱任务
#[tauri::command]
#[allow(dead_code)]
pub fn cancel_toolbox() {
    TOOLBOX_CANCELLED.store(true, Ordering::SeqCst);
    shared::logger::log_info("toolbox", "收到取消请求");
}

// ---- Presets ----

/// 加载预设图片（工具箱用：皮肤目录 + 系统内置 + 用户预设）
#[tauri::command]
pub async fn load_presets(app: tauri::AppHandle, skin_root: String) -> Vec<shared::preset_loader::PresetInfo> {
    let mut all: Vec<shared::preset_loader::PresetInfo> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // 1. 皮肤目录下的 presets/ 文件夹
    let skin_root_path = PathBuf::from(&skin_root);
    for p in shared::preset_loader::load_presets_from_dir(&skin_root_path) {
        seen.insert(p.name.clone());
        all.push(p);
    }

    // 2. 系统内置 + 用户预设（渲染缩略图 → base64 data URL）
    let builtin = preset::builtin_presets();
    let user = load_user_presets(&app);
    let builtin_names: std::collections::HashSet<&str> =
        builtin.iter().map(|p| p.name.as_str()).collect();
    let user_filtered: Vec<Preset> = user
        .into_iter()
        .filter(|p| !builtin_names.contains(p.name.as_str()))
        .collect();
    let app_presets: Vec<Preset> = [builtin, user_filtered].concat();

    for p in &app_presets {
        if seen.contains(&p.name) { continue; }
        seen.insert(p.name.clone());
        let b64 = render_preset_thumbnail_sync(&p.config);
        all.push(shared::preset_loader::PresetInfo {
            name: p.name.clone(),
            image_path: format!("data:image/png;base64,{}", b64),
        });
    }

    all
}

/// 同步渲染预设缩略图（带缓存）
fn render_preset_thumbnail_sync(config: &TailConfig) -> String {
    let hash = config_hash(config);
    let cache_path = cache_dir().join(format!("{}.png", hash));

    if cache_path.exists() {
        if let Ok(bytes) = fs::read(&cache_path) {
            return base64::engine::general_purpose::STANDARD.encode(&bytes);
        }
    }

    let preview = renderer::render_preview(config);
    let cropped = shared::image_utils::crop_preset_thumbnail(&preview);

    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    image::DynamicImage::ImageRgba8(cropped)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .ok();

    if let Some(parent) = cache_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&cache_path, &png_bytes);

    base64::engine::general_purpose::STANDARD.encode(&png_bytes)
}

// ---- Validator ----

/// 皮肤文件校验
#[tauri::command]
pub async fn validate_skin_files_cmd(folder_path: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dir = PathBuf::from(&folder_path);
        if !dir.is_dir() {
            return Err("指定的路径不是有效的文件夹".to_string());
        }

        let ini_path = dir.join("skin.ini");
        if !ini_path.exists() {
            return Err("未找到 skin.ini 文件".to_string());
        }

        let skin_ini = shared::skin_ini::parse_skin_ini(&ini_path)?;
        let missing = shared::skin_validator::validate_skin_files(&skin_ini);

        if missing.is_empty() {
            let ok_msg = "所有图片文件均存在 ✓";
            shared::logger::log_info("validator", ok_msg);
            Ok(vec![ok_msg.to_string()])
        } else {
            let summary = format!("发现 {} 个缺失文件:", missing.len());
            shared::logger::log_warn("validator", &summary);
            let mut log: Vec<String> = vec![summary];
            for m in &missing {
                let keys_str: Vec<String> = m.keys.iter().map(|k| format!("{}k", k)).collect();
                let msg = format!("  ✗ [{}] {} (引用自: {})", m.image_type, m.stem, keys_str.join(", "));
                shared::logger::log_warn("validator", &msg);
                log.push(msg);
            }
            Ok(log)
        }
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

/// 校验文件夹是否为有效皮肤目录（包含 skin.ini）
#[tauri::command]
pub fn check_skin_ini(folder_path: String) -> Result<bool, String> {
    let dir = PathBuf::from(&folder_path);
    if !dir.is_dir() {
        return Ok(false);
    }
    Ok(dir.join("skin.ini").is_file())
}

// ── 皮肤适配修复（统一包装）──────────────────────────────────

static REPAIR_ADAPTER_CANCELLED: AtomicBool = AtomicBool::new(false);

/// 皮肤适配修复：逐 stem 执行并通过 app:event 实时推送进度。
/// 前端 fire-and-forget 调用，监听 target="repair" 的事件。
#[tauri::command]
pub async fn repair_skin_adapter(
    app: tauri::AppHandle,
    folder_path: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        REPAIR_ADAPTER_CANCELLED.store(false, Ordering::SeqCst);

        let dir = PathBuf::from(&folder_path);
        if !dir.is_dir() {
            events::emit_log(&app, "error", "repair", "指定的路径不是有效的文件夹");
            events::emit_log(&app, "done", "repair", "修复异常终止");
            return;
        }
        let backup_dir = dir.join("_backup");
        let ini_path = dir.join("skin.ini");
        if !ini_path.exists() {
            events::emit_log(&app, "error", "repair", "未找到 skin.ini 文件");
            events::emit_log(&app, "done", "repair", "修复异常终止");
            return;
        }

        // ---- Phase 1: 面尾修复（逐 stem） ----
        events::emit_log(&app, "info", "repair", "开始修复面尾拉伸...");
        shared::logger::log_info("repair", "开始修复面尾拉伸...");

        let skin_ini = match shared::skin_ini::parse_skin_ini(&ini_path) {
            Ok(s) => s,
            Err(e) => {
                events::emit_log(&app, "error", "repair", &format!("解析 skin.ini 失败: {}", e));
                events::emit_log(&app, "done", "repair", "修复异常终止");
                return;
            }
        };
        events::emit_log(&app, "info", "repair", &format!("解析到 {} 个 [Mania] 小节", skin_ini.mania_sections.len()));

        let tail_groups = shared::skin_ini::group_note_image_l_by_stem(&skin_ini.mania_sections);
        let total_tails = tail_groups.len();
        events::emit_log(&app, "info", "repair", &format!("共 {} 组不同的面尾图片", total_tails));

        let ts_dir = shared::backup::backup_timestamp();
        let mut tail_patches: Vec<(u32, u32, String)> = Vec::new();
        
        let mut tail_ok = 0usize;

        for (stem, refs) in &tail_groups {
            if REPAIR_ADAPTER_CANCELLED.load(Ordering::SeqCst) {
                events::emit_log(&app, "done", "repair", "修复已取消");
                return;
            }
            
            match shared::lazer_repair::repair_one_tail_stem(
                &dir, stem, refs, &skin_ini.mania_sections, &backup_dir, &ts_dir,
            ) {
                Ok((stem_log, patches)) => {
                    for line in &stem_log {
                        events::emit_log(&app, "info", "repair", line);
                    }
                    tail_patches.extend(patches);
                    tail_ok += 1;
                }
                Err(e) => {
                    events::emit_log(&app, "error", "repair", &format!("{}: {}", stem, e));
                }
            }
        }

        // 应用面尾 ini 补丁
        for line in shared::lazer_repair::apply_tail_ini_patches(&ini_path, &tail_patches) {
            events::emit_log(&app, "info", "repair", &line);
        }

        let tail_msg = format!("面尾修复完成，{}/{} 组", tail_ok, total_tails);
        events::emit_log(&app, "success", "repair", &tail_msg);
        shared::logger::log_info("repair", &tail_msg);

        // ---- Phase 2: Key/KeyD 修复（逐 stem） ----
        if REPAIR_ADAPTER_CANCELLED.load(Ordering::SeqCst) {
            events::emit_log(&app, "done", "repair", "修复已取消");
            return;
        }
        events::emit_log(&app, "info", "repair", "开始修复 Key/KeyD 拉伸...");
        shared::logger::log_info("repair", "开始修复 Key/KeyD 拉伸...");

        let key_groups = shared::skin_ini::group_key_images_by_stem(&skin_ini.mania_sections);
        let total_keys = key_groups.len();
        events::emit_log(&app, "info", "repair", &format!("共 {} 组不同的 Key 图片", total_keys));

        let mut key_patches: Vec<(u32, u32, bool, String)> = Vec::new();
        let mut key_ok = 0usize;

        for (stem, refs) in &key_groups {
            if REPAIR_ADAPTER_CANCELLED.load(Ordering::SeqCst) {
                events::emit_log(&app, "done", "repair", "修复已取消");
                return;
            }
            match shared::lazer_repair::repair_one_key_stem(
                &dir, stem, refs, &skin_ini.mania_sections, &backup_dir, &ts_dir,
            ) {
                Ok((stem_log, patches)) => {
                    for line in &stem_log {
                        events::emit_log(&app, "info", "repair", line);
                    }
                    key_patches.extend(patches);
                    key_ok += 1;
                }
                Err(e) => {
                    events::emit_log(&app, "error", "repair", &format!("{}: {}", stem, e));
                }
            }
        }

        // 应用 Key ini 补丁
        for line in shared::lazer_repair::apply_key_ini_patches(&ini_path, &key_patches) {
            events::emit_log(&app, "info", "repair", &line);
        }

        let key_msg = format!("Key/KeyD 修复完成，{}/{} 组", key_ok, total_keys);
        events::emit_log(&app, "success", "repair", &key_msg);
        shared::logger::log_info("repair", &key_msg);

        events::emit_log(&app, "done", "repair", "全部修复完成");
        shared::logger::log_info("repair", "全部修复完成");
    }).await.map_err(|e| format!("任务执行失败: {}", e))?;

    Ok(())
}

/// 取消正在进行的皮肤适配修复任务
#[tauri::command]
pub fn cancel_repair_skin_adapter() {
    REPAIR_ADAPTER_CANCELLED.store(true, Ordering::SeqCst);
}