mod converter;
mod key_finder;
mod preset_loader;
mod skin_finder;

/// 用默认浏览器打开 URL（包装 shared 库）
#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    shared::open_url(&url)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            converter::convert_tail,
            key_finder::find_keys,
            preset_loader::load_presets,
            skin_finder::find_skin_root,
            open_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
