mod commands;
mod config;
mod preset;
mod renderer;
mod tools;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
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
            render_preview,
            export_image,
            export_image_bytes,
            validate_config,
            get_presets,
            get_default_config,
            save_user_presets,
            render_preset_thumbnail,
            open_url,
            parse_image_to_preset,
            get_image_preview_top,
            get_external_tool_path,
            copy_external_tool_with_presets,
            add_external_tool_to_osk_with_presets,
            repair_lazer_tail_folder,
            repair_key_image_folder,
            repair_lazer_osk,
            get_skin_throw_info,
            compute_lazer_throws,
            compute_lazer_throw_single,
            get_image_key_info,
            get_keyd_list,
            modify_skin_throw_length,
            get_tail_preview,
            convert_tail_toolbox,
            load_presets,
            validate_skin_files_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
