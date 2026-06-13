/// 加载预设图片 — 从小工具 exe 同级的 presets/ 目录加载
use shared::preset_loader::PresetInfo;

#[tauri::command]
pub fn load_presets() -> Vec<PresetInfo> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_default();
    let presets_dir = exe_dir.join("presets");
    shared::preset_loader::load_presets_from_dir_direct(&presets_dir)
}
