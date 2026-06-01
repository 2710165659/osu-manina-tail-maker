mod commands;
mod config;
mod preset;
mod renderer;

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
            validate_config,
            get_presets,
            get_default_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
