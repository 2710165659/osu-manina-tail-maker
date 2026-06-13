mod converter;
mod key_finder;
mod preset_loader;
mod skin_finder;

/// 用默认浏览器打开 URL（包装 shared 库）
#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    shared::open_url(&url)
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

/// 获取尾部预览图 base64 — thin wrapper
#[tauri::command]
async fn get_tail_preview(skin_root: String, stem: String) -> Result<String, String> {
    shared::throw_info::get_tail_preview_base64(std::path::Path::new(&skin_root), &stem)
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
            get_skin_throw_info,
            compute_lazer_throws,
            get_image_key_info,
            get_keyd_list,
            get_tail_preview,
            open_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
