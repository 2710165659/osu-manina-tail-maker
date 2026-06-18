/// 加载预设图片 — 从皮肤目录 presets/ + 小工具 exe 同级 presets/ 目录加载，裁剪缩略图后转为 base64
use shared::preset_loader::PresetInfo;
use std::collections::HashSet;

#[tauri::command]
pub fn load_presets(skin_root: String) -> Vec<PresetInfo> {
    let skin_root_path = std::path::PathBuf::from(&skin_root);
    let mut all: Vec<PresetInfo> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    // 1. 皮肤目录下的 presets/ 文件夹
    for p in shared::preset_loader::load_presets_from_dir(&skin_root_path) {
        seen.insert(p.name.clone());
        all.push(p);
    }

    // 2. 小工具 exe 同级的 presets/ 目录（base64 编码）
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_default();
    let presets_dir = exe_dir.join("presets");
    let exe_presets = shared::preset_loader::load_presets_from_dir_direct(&presets_dir);

    for mut p in exe_presets {
        if seen.contains(&p.name) {
            continue;
        }
        if p.image_path.starts_with("data:") {
            seen.insert(p.name.clone());
            all.push(p);
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
        seen.insert(p.name.clone());
        p.image_path = b64;
        all.push(p);
    }

    all
}
