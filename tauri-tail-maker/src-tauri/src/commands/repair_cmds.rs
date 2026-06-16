use std::path::PathBuf;
use tauri::AppHandle;

use crate::events;

/// 扫描皮肤文件夹的待修复图片列表，通过 app:event 流式推送结果。
///
/// 前端 fire-and-forget 调用，监听 `app:event` target="repair":
/// - data `{ kind: "tails", items: [...] }` — 面尾列表
/// - data `{ kind: "keyds", items: [...] }` — Key/KeyD 列表
/// - level="done" 表示扫描完成
#[tauri::command]
pub async fn scan_repair_info(
    app: AppHandle,
    folder_path: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dir = PathBuf::from(&folder_path);

        // Phase 1: 面尾列表
        match shared::throw_info::get_image_key_info(&dir) {
            Ok(tails) => {
                let count = tails.len();
                events::emit_data(
                    &app,
                    "success",
                    "repair",
                    &format!("面尾 {} 项", count),
                    serde_json::json!({ "kind": "tails", "items": tails }),
                );
            }
            Err(e) => {
                events::emit_log(&app, "error", "repair", &format!("面尾列表加载失败: {}", e));
            }
        }

        // Phase 2: Key/KeyD 列表
        match shared::throw_info::get_keyd_list(&dir) {
            Ok(kds) => {
                let count = kds.len();
                events::emit_data(
                    &app,
                    "success",
                    "repair",
                    &format!("Key/KeyD {} 项", count),
                    serde_json::json!({ "kind": "keyds", "items": kds }),
                );
            }
            Err(e) => {
                events::emit_log(&app, "error", "repair", &format!("Key/KeyD 列表加载失败: {}", e));
            }
        }

        events::emit_log(&app, "done", "repair", "扫描完成");
    }).await.map_err(|e| format!("任务执行失败: {}", e))?;

    Ok(())
}
