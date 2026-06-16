use std::collections::HashSet;
use std::path::PathBuf;
use tauri::AppHandle;

use crate::events;

/// 批量计算所有 stem 的 lazer 投长度，通过 app:event 逐个推送结果。
///
/// 前端 fire-and-forget，监听 `app:event` target="throw":
/// - data 中携带 `{ stem, lazer_throw }` 逐条结果
/// - level="done" 表示全部完成
#[tauri::command]
pub async fn compute_all_lazer_throws(
    app: AppHandle,
    folder_path: String,
) -> Result<(), String> {
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
        events::emit_log(&app, "done", "throw", "无面尾图片");
        return Ok(());
    }

    tauri::async_runtime::spawn_blocking(move || {
        let total = stems.len();
        for stem in &stems {
            let t = shared::throw_info::compute_lazer_throw_single(&skin_dir, stem, 0);
            events::emit_data(&app, "info", "throw", &format!("{} 投长度: {}", stem, t),
                serde_json::json!({ "stem": stem, "lazer_throw": t }));
        }
        events::emit_log(&app, "done", "throw", &format!("投长度计算完成（{} 个）", total));
    }).await.map_err(|e| format!("任务执行失败: {}", e))?;

    Ok(())
}
