use crate::config::TailConfig;
use crate::renderer;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::events;

// ── 批量导出 ─────────────────────────────────────────────────

static BATCH_CANCELLED: AtomicBool = AtomicBool::new(false);

/// 批量导出图片：收集所有配置后一次调用，后端异步逐个生成并通过事件推送结果。
///
/// 前端 fire-and-forget 调用，监听 `app:event` 事件，target="batch"。
/// 进度数据通过 `data: { index, total }` 字段携带。
#[tauri::command]
pub async fn batch_export_images(
    app: tauri::AppHandle,
    configs: Vec<TailConfig>,
    output_folder: String,
    filenames: Vec<String>,
    preset_names: Vec<String>,
) -> Result<(), String> {
    if configs.len() != filenames.len() {
        return Err("configs 与 filenames 长度不匹配".to_string());
    }
    let total = configs.len();
    let names_str = preset_names.join("、");

    tauri::async_runtime::spawn_blocking(move || {
        BATCH_CANCELLED.store(false, Ordering::SeqCst);

        // 开始
        let start_msg = format!("开始批量生成：{}，共 {} 张", names_str, total);
        shared::logger::log_info("batch", &start_msg);
        events::emit_data(&app, "info", "batch", &start_msg, serde_json::json!({ "index": 0, "total": total }));

        let mut ok = 0usize;
        let mut fail = 0usize;

        for i in 0..total {
            if BATCH_CANCELLED.load(Ordering::SeqCst) {
                let cancel_msg = format!("已取消（完成 {}/{}）", ok + fail, total);
                shared::logger::log_info("batch", &cancel_msg);
                events::emit_log(&app, "done", "batch", &cancel_msg);
                return;
            }

            let output_path = PathBuf::from(&output_folder).join(&filenames[i]);
            let img = renderer::render(&configs[i]);
            match img.save(&output_path) {
                Ok(()) => {
                    ok += 1;
                    let msg = format!("✓ {}", filenames[i]);
                    shared::logger::log_info("batch", &msg);
                    events::emit_data(&app, "success", "batch", &msg, serde_json::json!({ "index": i + 1, "total": total }));
                }
                Err(e) => {
                    fail += 1;
                    let msg = format!("✗ {}: {}", filenames[i], e);
                    shared::logger::log_error("batch", &msg);
                    events::emit_data(&app, "error", "batch", &msg, serde_json::json!({ "index": i + 1, "total": total }));
                }
            }
        }

        // 结束
        let end_msg = if fail > 0 {
            format!("批量生成完成：成功 {}，失败 {}", ok, fail)
        } else {
            format!("批量生成完成：全部 {} 张成功", ok)
        };
        shared::logger::log_info("batch", &end_msg);
        events::emit_log(&app, "done", "batch", &end_msg);
    }).await.map_err(|e| format!("任务执行失败: {}", e))?;

    Ok(())
}

/// 取消正在进行的批量导出任务
#[tauri::command]
pub fn cancel_batch_export() {
    BATCH_CANCELLED.store(true, Ordering::SeqCst);
    shared::logger::log_info("batch", "收到取消请求");
}
