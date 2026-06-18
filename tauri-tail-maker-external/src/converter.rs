/// 工具箱"一键修改面尾"命令
///
/// 统一编排一键修改面尾的三个步骤：
/// 1. Key/KeyD 修复（仅 lazer 模式）
/// 2. 预设替换（用预设图片覆盖面尾图片，替换前备份原图）
/// 3. 修改投长度（stable 或 lazer 含拉伸，支持多 ColumnWidth 副本生成）
///
/// 通过 app:event 流式推送进度，同时收集所有日志在返回值中。
/// 前端 await 返回值即可获取全部日志（事件不可用时也能正常工作）。
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use serde::Serialize;
use tauri::Emitter;

/// 日志条目，返回给前端
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
}

/// 工具箱"一键修改面尾"命令（fire-and-forget 风格，但返回全部日志供前端 fallback）。
///
/// 通过 app:event 流式推送进度（有 listen 时实时展示），同时收集所有日志在返回值中。
/// 前端应 await 返回值，若事件不可用则直接用返回值渲染日志。
#[tauri::command]
pub async fn convert_tail_toolbox(
    app: tauri::AppHandle,
    folder_path: String,
    work_mode: String,
    throws: Vec<(u32, u32)>,
    presets: Vec<(String, String)>,
    keyd_stems: Vec<String>,
) -> Result<Vec<LogEntry>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let skin_dir = PathBuf::from(&folder_path);
        if !skin_dir.is_dir() {
            let msg = "指定的路径不是有效的文件夹".to_string();
            emit_log(&app, "error", "toolbox", &msg);
            return Err(msg);
        }
        let backup_dir = skin_dir.join("_backup");
        let mut all_logs: Vec<LogEntry> = Vec::new();

        let mut log = |level: &str, msg: &str| {
            emit_log(&app, level, "toolbox", msg);
            all_logs.push(LogEntry { level: level.into(), message: msg.into() });
        };

        log("info","开始一键修改面尾...");
        shared::logger::log_info("toolbox", "开始一键修改面尾...");

        let throw_map: HashMap<u32, u32> = throws.iter().cloned().collect();

        // ---- Step 1: Key/KeyD 修复（仅 lazer） ----
        if work_mode == "lazer" && !keyd_stems.is_empty() {
            log("info","--- Key/KeyD 修复 ---");
            shared::logger::log_info("toolbox", "--- Key/KeyD 修复 ---");

            let ts_dir = shared::backup::backup_timestamp();
            match shared::tail_toolbox::execute_key_repair_step(
                &skin_dir, &backup_dir, &keyd_stems, &ts_dir,
            ) {
                Ok(key_log) => {
                    for line in &key_log {
                        log("info",line);
                    }
                    let msg = "Key/KeyD 修复完成";
                    log("success",msg);
                    shared::logger::log_info("toolbox", msg);
                }
                Err(e) => {
                    let msg = format!("Key 修复失败: {}", e);
                    log("error",&msg);
                    shared::logger::log_error("toolbox", &msg);
                }
            }
        }

        // ---- Step 2: 预设替换 ----
        if !presets.is_empty() {
            log("info","--- 用预设替换现有图片 ---");
            shared::logger::log_info("toolbox", "--- 用预设替换现有图片 ---");

            let resolved: Vec<(String, String)> = presets.iter().map(|(stem, name)| {
                for ext in &["png", "jpg", "jpeg"] {
                    let p = skin_dir.join("presets").join(format!("{}.{}", name, ext));
                    if p.exists() {
                        return (stem.clone(), p.to_string_lossy().to_string());
                    }
                    let p = skin_dir.join(format!("{}.{}", name, ext));
                    if p.exists() {
                        return (stem.clone(), p.to_string_lossy().to_string());
                    }
                }
                (stem.clone(), name.clone())
            }).collect();

            let preset_ts = shared::backup::backup_timestamp();
            match shared::tail_toolbox::execute_preset_step(
                &skin_dir, &resolved, &backup_dir, &preset_ts,
            ) {
                Ok(preset_log) => {
                    for line in &preset_log {
                        log("info",line);
                    }
                    let msg = "预设替换完成";
                    log("success",msg);
                    shared::logger::log_info("toolbox", msg);
                }
                Err(e) => {
                    let msg = format!("预设替换失败: {}", e);
                    log("error",&msg);
                    shared::logger::log_error("toolbox", &msg);
                }
            }
        }

        // ---- Step 3: 修改投长度（逐 stem，支持多 ColumnWidth） ----
        log("info","--- 修改投长度 ---");
        shared::logger::log_info("toolbox", "--- 修改投长度 ---");

        let ini_path = skin_dir.join("skin.ini");
        let skin_ini = match shared::skin_ini::parse_skin_ini(&ini_path) {
            Ok(s) => s,
            Err(e) => {
                let msg = format!("解析 skin.ini 失败: {}", e);
                log("error",&msg);
                shared::logger::log_error("toolbox", &msg);
                return Ok(all_logs);
            }
        };

        let throw_ts_dir = shared::backup::backup_timestamp();
        let mut first_seen: HashSet<String> = HashSet::new();
        let mut done_pairs: HashSet<(String, u32)> = HashSet::new();
        let mut processed = false;

        for section in &skin_ini.mania_sections {
            let target_throw = match throw_map.get(&section.keys) {
                Some(&t) => t,
                None => continue,
            };

            let cw = section.column_width;

            for r in &section.note_image_ls {
                let pair = (r.name.clone(), cw);
                if !done_pairs.insert(pair) {
                    continue;
                }

                if !first_seen.contains(&r.name) {
                    first_seen.insert(r.name.clone());
                    match shared::throw_info::modify_one_throw_stem(
                        &skin_dir, &r.name, section.keys, target_throw,
                        &work_mode, cw, &backup_dir, &throw_ts_dir,
                    ) {
                        Ok(stem_log) => {
                            for line in &stem_log {
                                log("info",line);
                            }
                            processed = true;
                        }
                        Err(e) => {
                            log("error",&format!("{}: {}", r.name, e));
                        }
                    }
                } else {
                    let copy_stem = format!("{}_cw{}", r.name, cw);
                    let image_path = match shared::skin_ini::find_image_file(&skin_dir, &r.name) {
                        Some(p) => p,
                        None => {
                            log("warning",&format!("⚠ 找不到原图: {}", r.name));
                            continue;
                        }
                    };
                    let ext = image_path.extension().and_then(|e| e.to_str()).unwrap_or("png");
                    let copy_path = skin_dir.join(format!("{}.{}", copy_stem, ext));

                    match image::open(&image_path) {
                        Ok(img) => {
                            let rgba = img.to_rgba8();
                            let current_throw = shared::throw_length::find_throw_length(&rgba);
                            let modified = if work_mode == "lazer" {
                                let h = rgba.height();
                                let cur_lazer = if h > 0 {
                                    ((current_throw as u64 * 32800) / h as u64) as u32
                                } else {
                                    0
                                };
                                log("info",&format!(
                                    "{} {}k: 投长度 {}px → {}px (Lazer, cw={}, 副本: {})",
                                    image_path.display(), section.keys, cur_lazer, target_throw, cw, copy_stem
                                ));
                                shared::throw_length::modify_throw_length_lazer(&rgba, target_throw, cw)
                            } else {
                                log("info",&format!(
                                    "{} {}k: 投长度 {}px → {}px (Stable, 副本: {})",
                                    image_path.display(), section.keys, current_throw, target_throw, copy_stem
                                ));
                                shared::throw_length::modify_throw_length(&rgba, target_throw)
                            };
                            if let Err(e) = modified.save(&copy_path) {
                                log("error",&format!("保存副本失败 {}: {}", copy_stem, e));
                            }
                        }
                        Err(e) => {
                            log("error",&format!("读取原图失败 {}: {}", r.name, e));
                            continue;
                        }
                    }

                    for sec_r in &section.note_image_ls {
                        if sec_r.name == r.name {
                            match shared::skin_ini::update_note_image_l_in_ini(
                                &ini_path, section.keys, sec_r.column, &copy_stem,
                            ) {
                                Ok(()) => {
                                    log("info",&format!(
                                        "  {}k NoteImage{}L → {}", section.keys, sec_r.column, copy_stem
                                    ));
                                }
                                Err(e) => {
                                    log("warning",&format!("⚠ {}", e));
                                }
                            }
                        }
                    }
                    processed = true;
                }
            }
        }

        if !processed {
            log("info","未找到匹配的键数小节");
        }
        log("success","投长度修改完成");
        shared::logger::log_info("toolbox", "投长度修改完成");

        let done_msg = "全部完成！";
        emit_log(&app, "done", "toolbox", done_msg);
        all_logs.push(LogEntry { level: "done".into(), message: done_msg.into() });
        shared::logger::log_info("toolbox", "一键修改面尾完成");

        Ok(all_logs)
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

// ---------------------------------------------------------------------------
// 事件推送辅助
// ---------------------------------------------------------------------------

/// 纯文本日志事件（target="toolbox"，前端通过 app:event 监听）
fn emit_log(app: &tauri::AppHandle, level: &str, target: &str, message: &str) {
    let _ = app.emit("app:event", serde_json::json!({
        "level": level,
        "target": target,
        "message": message,
    }));
}
