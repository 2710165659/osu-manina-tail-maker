/// 从 skin.ini 查找键数 — thin wrapper
use shared::skin_ini;

#[tauri::command]
pub fn find_keys(skin_root: String) -> skin_ini::KeyFinderResult {
    skin_ini::find_keys(std::path::Path::new(&skin_root))
}
