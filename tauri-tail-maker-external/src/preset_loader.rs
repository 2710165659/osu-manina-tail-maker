/// 加载预设图片 — 从小工具 exe 同级的 presets/ 目录加载，裁剪缩略图后转为 base64
use shared::preset_loader::PresetInfo;

#[tauri::command]
pub fn load_presets() -> Vec<PresetInfo> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_default();
    let presets_dir = exe_dir.join("presets");
    let mut presets = shared::preset_loader::load_presets_from_dir_direct(&presets_dir);

    for p in &mut presets {
        if p.image_path.starts_with("data:") {
            continue;
        }
        // 读取 PNG → 裁剪缩略图 → 编码 base64
        let b64 = match image::open(&p.image_path) {
            Ok(img) => {
                let cropped = shared::image_utils::crop_preset_thumbnail(&img.to_rgba8());
                let mut buf = Vec::new();
                if image::DynamicImage::ImageRgba8(cropped)
                    .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
                    .is_ok()
                {
                    use base64::Engine;
                    let b = base64::engine::general_purpose::STANDARD.encode(&buf);
                    format!("data:image/png;base64,{}", b)
                } else {
                    continue;
                }
            }
            Err(_) => continue,
        };
        p.image_path = b64;
    }

    presets
}
