mod converter;
mod key_finder;
mod preset_loader;
mod skin_finder;

use std::collections::HashSet;
use std::path::PathBuf;
use tauri::Emitter;
use tauri_plugin_dialog::DialogExt;

/// 校验文件夹是否为有效皮肤目录（包含 skin.ini）
#[tauri::command]
fn check_skin_ini(folder_path: String) -> Result<bool, String> {
    let dir = PathBuf::from(&folder_path);
    if !dir.is_dir() {
        return Ok(false);
    }
    Ok(dir.join("skin.ini").is_file())
}

/// 用默认浏览器打开 URL（包装 shared 库）
#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    shared::open_url(&url)
}

/// 打开文件夹选择对话框，返回所选路径
#[tauri::command]
async fn browse_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let path = app
        .dialog()
        .file()
        .blocking_pick_folder();
    Ok(path.map(|p| p.to_string()))
}

/// 获取投长度信息 — thin wrapper
#[tauri::command]
async fn get_skin_throw_info(skin_root: String) -> Result<Vec<shared::throw_info::SkinThrowInfo>, String> {
    shared::throw_info::get_throw_info(std::path::Path::new(&skin_root))
}

/// 按需计算 lazer 投长度
#[tauri::command]
async fn compute_lazer_throws(skin_root: String, stems: Vec<String>) -> Result<Vec<(String, u32)>, String> {
    shared::throw_info::compute_lazer_throws(std::path::Path::new(&skin_root), &stems)
}

/// 计算所有面尾 stem 的 lazer 投长度（自发现 stems，通过 app:throw-result 事件推送结果）
#[tauri::command]
async fn compute_all_lazer_throws(app: tauri::AppHandle, folder_path: String) -> Result<(), String> {
    let skin_dir = PathBuf::from(&folder_path);
    if !skin_dir.is_dir() {
        return Err("指定的路径不是有效的文件夹".to_string());
    }
    let ini_path = skin_dir.join("skin.ini");
    if !ini_path.exists() {
        return Err("未找到 skin.ini 文件".to_string());
    }
    let skin_ini = shared::skin_ini::parse_skin_ini(&ini_path)?;
    let mut stems: Vec<String> = Vec::new();
    let mut seen = HashSet::new();
    for section in &skin_ini.mania_sections {
        for r in &section.note_image_ls {
            if seen.insert(r.name.clone()) {
                stems.push(r.name.clone());
            }
        }
    }
    if stems.is_empty() {
        let _ = app.emit("app:throw-result", serde_json::json!({ "items": [] }));
        let _ = app.emit("app:event", serde_json::json!({
            "level": "done",
            "target": "throw",
            "message": "无面尾图片",
            "data": { "done": true }
        }));
        return Ok(());
    }

    tauri::async_runtime::spawn_blocking(move || {
        let total = stems.len();
        let mut results: Vec<serde_json::Value> = Vec::with_capacity(total);
        for stem in &stems {
            let t = shared::throw_info::compute_lazer_throw_single(&skin_dir, stem, 0);
            results.push(serde_json::json!({ "stem": stem, "lazer_throw": t }));
        }
        let _ = app.emit("app:throw-result", serde_json::json!({ "items": results }));
        let _ = app.emit("app:event", serde_json::json!({
            "level": "done",
            "target": "throw",
            "message": format!("投长度计算完成（{} 个）", total),
            "data": { "done": true }
        }));
    }).await.map_err(|e| format!("任务执行失败: {}", e))?;

    Ok(())
}

/// 获取图片-键数-轨道关联信息
#[tauri::command]
async fn get_image_key_info(skin_root: String) -> Result<Vec<shared::throw_info::ImageKeyInfo>, String> {
    shared::throw_info::get_image_key_info(std::path::Path::new(&skin_root))
}

/// 获取 Key/KeyD stem 列表
#[tauri::command]
async fn get_keyd_list(skin_root: String) -> Result<Vec<shared::throw_info::KeydStemInfo>, String> {
    shared::throw_info::get_keyd_list(std::path::Path::new(&skin_root))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            check_skin_ini,
            converter::convert_tail_toolbox,
            key_finder::find_keys,
            preset_loader::load_presets,
            skin_finder::find_skin_root,
            get_skin_throw_info,
            compute_lazer_throws,
            compute_all_lazer_throws,
            get_image_key_info,
            get_keyd_list,
            open_url,
            browse_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
