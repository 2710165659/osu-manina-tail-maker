mod commands;
mod config;
mod events;
mod logger;
mod preset;
mod renderer;
mod tools;

use commands::*;
use logger::{init_logger, emit_logs};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // ── 初始化日志分发器（全局单例 + 后台异步消费 loop）──
            let (dispatcher, rx) = shared::logger::init_global_dispatcher();
            tauri::async_runtime::spawn(shared::logger::run_dispatcher(dispatcher, rx));

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            init_logger,
            emit_logs,
            compute_all_lazer_throws,
            render_preview,
            export_image,
            export_image_bytes,
            get_presets,
            get_default_config,
            save_user_presets,
            render_preset_thumbnail,
            open_url,
            parse_image_to_preset,
            get_image_preview_top,
            copy_external_tool_with_presets,
            repair_lazer_tail_folder,
            repair_key_image_folder,
            repair_skin_adapter,
            cancel_repair_skin_adapter,
            scan_repair_info,
            get_skin_throw_info,
            get_image_key_info,
            get_keyd_list,
            convert_tail_toolbox,
            add_script_to_skin,
            cancel_add_script,
            batch_export_images,
            cancel_batch_export,
            load_presets,
            validate_skin_files_cmd,
            check_skin_ini,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
