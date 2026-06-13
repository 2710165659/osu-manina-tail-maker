/// 预设图片加载
///
/// 扫描皮肤目录下的 presets/ 子目录，加载预设图片列表。
use std::path::Path;

use serde::Serialize;

/// 预设信息
#[derive(Debug, Serialize, Clone)]
pub struct PresetInfo {
    /// 预设名称（不含扩展名）
    pub name: String,
    /// 图片路径（绝对路径）
    pub image_path: String,
}

/// 支持的图片扩展名
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "bmp", "webp", "gif"];

/// 扫描指定目录下的预设图片。
///
/// # 参数
/// - `skin_root`: 皮肤根目录路径
///
/// # 返回
/// 预设图片列表，如果 presets 文件夹不存在则返回空列表。
pub fn load_presets_from_dir(skin_root: &Path) -> Vec<PresetInfo> {
    let presets_dir = skin_root.join("presets");
    if !presets_dir.exists() || !presets_dir.is_dir() {
        return vec![];
    }
    scan_presets_dir(&presets_dir)
}

/// 扫描任意目录下的预设图片（直接扫描该目录，不附加 presets/ 子目录）。
pub fn load_presets_from_dir_direct(dir: &Path) -> Vec<PresetInfo> {
    if !dir.exists() || !dir.is_dir() {
        return vec![];
    }
    scan_presets_dir(dir)
}

/// 扫描预设目录
fn scan_presets_dir(dir: &Path) -> Vec<PresetInfo> {
    let mut presets = vec![];

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return presets,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // 只处理文件
        if !path.is_file() {
            continue;
        }

        // 检查扩展名
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !IMAGE_EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }

        // 获取文件名（不含扩展名）作为预设名
        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // 获取绝对路径
        let image_path = match path.canonicalize() {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => path.to_string_lossy().to_string(),
        };

        presets.push(PresetInfo { name, image_path });
    }

    // 按名称排序
    presets.sort_by(|a, b| a.name.cmp(&b.name));

    presets
}
