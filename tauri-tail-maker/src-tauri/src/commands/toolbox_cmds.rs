use crate::config::{Preset, TailConfig};
use crate::preset;
use crate::renderer;
use super::config_cmds::{cache_dir, config_hash, load_user_presets};
use base64::Engine;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::events;

// ---------------------------------------------------------------------------
// 工具箱命令 —— 包装 shared 库
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
        let backup_dir = dir.join("_backup");
        shared::lazer_repair::execute_lazer_tail_repair(&dir, &backup_dir)
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
        let backup_dir = dir.join("_backup");
        shared::lazer_repair::execute_lazer_key_repair(&dir, &backup_dir, &mode)
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

/// 工具箱"一键修改面尾"命令
#[derive(Debug, serde::Serialize)]
pub struct ToolboxConvertResult {
    pub success: bool,
    pub message: String,
    pub logs: Vec<String>,
}

#[tauri::command]
pub async fn convert_tail_toolbox(
    folder_path: String,
    _skin_mode: String,
    work_mode: String,
    throws: Vec<(u32, u32)>,
    presets: Vec<(String, String)>,
    keyd_stems: Vec<String>,
) -> Result<ToolboxConvertResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        shared::logger::log_info("toolbox", "开始一键修改面尾...");
        let skin_dir = PathBuf::from(&folder_path);

        // Folder mode
        let backup_dir = skin_dir.join("_backup");
        match shared::tail_toolbox::execute_toolbox(
            &skin_dir, &work_mode, &throws, &presets, &keyd_stems, &backup_dir,
        ) {
            Ok(log) => {
                shared::logger::log_info("toolbox", "一键修改面尾完成");
                Ok(ToolboxConvertResult { success: true, message: "修改完成".to_string(), logs: log })
            }
            Err(e) => Ok(ToolboxConvertResult { success: false, message: e, logs: vec![] }),
        }
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

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
        // Render thumbnail to get base64
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

    // 缓存命中
    if cache_path.exists() {
        if let Ok(bytes) = fs::read(&cache_path) {
            return base64::engine::general_purpose::STANDARD.encode(&bytes);
        }
    }

    // 渲染全尺寸预览 → 裁剪缩略图
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

/// 皮肤适配修复：串行执行面尾修复 + Key/KeyD 修复，通过 app:event 流式推送进度。
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

        // Phase 1: 面尾修复
        events::emit_log(&app, "info", "repair", "开始修复面尾拉伸...");
        if REPAIR_ADAPTER_CANCELLED.load(Ordering::SeqCst) {
            events::emit_log(&app, "done", "repair", "修复已取消");
            return;
        }
        match shared::lazer_repair::execute_lazer_tail_repair(&dir, &backup_dir) {
            Ok(logs) => {
                events::emit_log(&app, "success", "repair", &format!("面尾修复完成，{} 项", logs.len()));
            }
            Err(e) => {
                events::emit_log(&app, "error", "repair", &format!("面尾修复失败: {}", e));
                events::emit_log(&app, "done", "repair", "修复异常终止");
                return;
            }
        }

        // Phase 2: Key/KeyD 修复
        if REPAIR_ADAPTER_CANCELLED.load(Ordering::SeqCst) {
            events::emit_log(&app, "done", "repair", "修复已取消");
            return;
        }
        events::emit_log(&app, "info", "repair", "开始修复 Key/KeyD 拉伸...");
        match shared::lazer_repair::execute_lazer_key_repair(&dir, &backup_dir, "all") {
            Ok(logs) => {
                events::emit_log(&app, "success", "repair", &format!("Key/KeyD 修复完成，{} 项", logs.len()));
            }
            Err(e) => {
                events::emit_log(&app, "error", "repair", &format!("Key/KeyD 修复失败: {}", e));
                events::emit_log(&app, "done", "repair", "修复异常终止");
                return;
            }
        }

        events::emit_log(&app, "done", "repair", "全部修复完成");
    }).await.map_err(|e| format!("任务执行失败: {}", e))?;

    Ok(())
}

/// 取消正在进行的皮肤适配修复任务
#[tauri::command]
pub fn cancel_repair_skin_adapter() {
    REPAIR_ADAPTER_CANCELLED.store(true, Ordering::SeqCst);
}

