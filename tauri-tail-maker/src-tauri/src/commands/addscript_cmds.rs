use crate::config::Preset;
use crate::preset;
use crate::renderer;
use super::config_cmds::{load_user_presets};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;

use crate::events;

// ---------------------------------------------------------------------------
// AddScript 命令 — fire-and-forget + 事件流式推送
// ---------------------------------------------------------------------------

static ADDSCRIPT_CANCELLED: AtomicBool = AtomicBool::new(false);

/// 为皮肤添加独立脚本工具（fire-and-forget，通过 app:event 推送进度）。
///
/// 前端调用后立刻返回 Ok(())，监听 target="addscript" 的 `app:event` 获取进度。
#[tauri::command]
pub async fn add_script_to_skin(
    app: tauri::AppHandle,
    folder_path: String,
    preset_names: Vec<String>,
) -> Result<(), String> {
    let presets: Vec<Preset> = {
        let builtin = preset::builtin_presets();
        let user = load_user_presets(&app);
        let builtin_names: std::collections::HashSet<&str> =
            builtin.iter().map(|p| p.name.as_str()).collect();
        let user_filtered: Vec<Preset> = user
            .into_iter()
            .filter(|p| !builtin_names.contains(p.name.as_str()))
            .collect();
        let all: Vec<Preset> = [builtin, user_filtered].concat();
        all.into_iter()
            .filter(|p| preset_names.contains(&p.name))
            .collect()
    };

    tauri::async_runtime::spawn_blocking(move || {
        ADDSCRIPT_CANCELLED.store(false, Ordering::SeqCst);

        let target_dir = PathBuf::from(&folder_path);
        events::emit_log(&app, "info", "addscript", &format!("目标文件夹: {}", folder_path));
        shared::logger::log_info("addscript", &format!("开始添加脚本: {}", folder_path));

        if presets.is_empty() {
            events::emit_log(&app, "info", "addscript", "未选择预设，仅复制工具程序");
        } else {
            let names: Vec<&str> = presets.iter().map(|p| p.name.as_str()).collect();
            events::emit_log(&app, "info", "addscript", &format!("选中预设: {}", names.join("、")));
        }

        // Phase 1: 导出预设图片
        let mut preset_images: Vec<(String, Vec<u8>)> = Vec::new();

        for preset in &presets {
            if ADDSCRIPT_CANCELLED.load(Ordering::SeqCst) {
                events::emit_log(&app, "done", "addscript", "任务已取消");
                shared::logger::log_info("addscript", "任务已取消");
                return;
            }

            shared::logger::log_info("addscript", &format!("导出预设: {}", preset.name));

            let _preview = renderer::render_preview(&preset.config);
            // 导出完整分辨率
            let full_img = renderer::render(&preset.config);

            let mut png_bytes = Vec::new();
            let mut cursor = std::io::Cursor::new(&mut png_bytes);
            match image::DynamicImage::ImageRgba8(full_img)
                .write_to(&mut cursor, image::ImageFormat::Png)
            {
                Ok(()) => {
                    preset_images.push((preset.name.clone(), png_bytes));
                    events::emit_log(&app, "success", "addscript", &format!("  ✓ 导出预设：{}", preset.name));
                }
                Err(e) => {
                    let msg = format!("  ✗ 导出预设 {} 失败: {}", preset.name, e);
                    events::emit_log(&app, "warning", "addscript", &msg);
                    shared::logger::log_error("addscript", &msg);
                }
            }
        }

        if ADDSCRIPT_CANCELLED.load(Ordering::SeqCst) {
            events::emit_log(&app, "done", "addscript", "任务已取消");
            shared::logger::log_info("addscript", "任务已取消");
            return;
        }

        // Phase 2: 复制外部工具
        shared::logger::log_info("addscript", "复制外部工具...");

        // 获取外部工具路径
        let tool_path = match get_external_tool_path_inner(&app) {
            Ok(p) => p,
            Err(e) => {
                events::emit_log(&app, "error", "addscript", &e);
                shared::logger::log_error("addscript", &e);
                events::emit_log(&app, "done", "addscript", "任务异常终止");
                return;
            }
        };

        let scripts_dir = target_dir.join("scripts");
        let presets_dir = scripts_dir.join("presets");
        let target_file = scripts_dir.join("tail-maker-external.exe");

        // 创建目录
        if let Err(e) = fs::create_dir_all(&scripts_dir) {
            let msg = format!("创建 scripts 目录失败: {}", e);
            events::emit_log(&app, "error", "addscript", &msg);
            shared::logger::log_error("addscript", &msg);
            events::emit_log(&app, "done", "addscript", "任务异常终止");
            return;
        }
        if !preset_images.is_empty() {
            let _ = fs::create_dir_all(&presets_dir);
        }

        // 复制 exe
        match fs::copy(&tool_path, &target_file) {
            Ok(_) => {
                events::emit_log(&app, "success", "addscript", &format!("✓ 外部工具已复制到: {}", target_file.display()));
                shared::logger::log_info("addscript", &format!("外部工具已复制到: {}", target_file.display()));
            }
            Err(e) => {
                let msg = format!("复制文件失败: {}", e);
                events::emit_log(&app, "error", "addscript", &msg);
                shared::logger::log_error("addscript", &msg);
                events::emit_log(&app, "done", "addscript", "任务异常终止");
                return;
            }
        }

        // 保存预设图片
        for (name, image_bytes) in &preset_images {
            let image_path = presets_dir.join(format!("{}.png", name));
            if let Err(e) = fs::write(&image_path, image_bytes) {
                let msg = format!("保存预设图片 {} 失败: {}", name, e);
                events::emit_log(&app, "warning", "addscript", &msg);
                shared::logger::log_error("addscript", &msg);
            }
        }

        if !preset_images.is_empty() {
            let msg = format!("✓ 已添加 {} 个预设图片", preset_images.len());
            events::emit_log(&app, "success", "addscript", &msg);
            shared::logger::log_info("addscript", &msg);
        }

        events::emit_log(&app, "done", "addscript", "✓ 脚本添加完成！");
        shared::logger::log_info("addscript", "脚本添加完成");
    }).await.map_err(|e| format!("任务执行失败: {}", e))?;

    Ok(())
}

/// 取消正在进行的 AddScript 任务
#[tauri::command]
pub fn cancel_add_script() {
    ADDSCRIPT_CANCELLED.store(true, Ordering::SeqCst);
    shared::logger::log_info("addscript", "收到取消请求");
}

// ---------------------------------------------------------------------------
// 内部辅助
// ---------------------------------------------------------------------------

/// 获取外部工具 exe 的路径（内部版本，不暴露为 Tauri command）
fn get_external_tool_path_inner(app: &tauri::AppHandle) -> Result<String, String> {
    if cfg!(debug_assertions) {
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("获取当前目录失败: {}", e))?;
        let workspace_root = current_dir
            .parent()
            .ok_or("无法获取项目根目录")?
            .parent()
            .ok_or("无法获取工作区根目录")?;

        for profile in &["release", "debug"] {
            let exe = workspace_root.join("target").join(profile).join("tail-maker-external.exe");
            if exe.exists() {
                return Ok(exe.to_string_lossy().to_string());
            }
        }
        return Err("开发模式下未找到小工具，请先编译 tauri-tail-maker-external".to_string());
    }

    let resource_path = app.path().resource_dir()
        .map_err(|e| format!("获取资源目录失败: {}", e))?
        .join("tail-maker-external.exe");

    if resource_path.exists() {
        return Ok(resource_path.to_string_lossy().to_string());
    }

    Err("未找到外部工具".to_string())
}
