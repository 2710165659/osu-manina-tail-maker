use crate::config::{Preset, TailConfig, ValidationResult};
use crate::preset;
use crate::renderer;
use base64::Engine;
use image::ImageBuffer;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::path::PathBuf;
use tauri::Manager;

/// 渲染预览图（全宽，最多 1000 行），返回 base64 编码的 PNG
#[tauri::command]
pub fn render_preview(config: TailConfig) -> Result<String, String> {
    let validation = config.validate();
    if !validation.valid {
        return Err(validation.errors.join("; "));
    }

    let preview = renderer::render_preview(&config);

    // 编码为 PNG
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    image::DynamicImage::ImageRgba8(preview)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("PNG 编码失败: {}", e))?;

    Ok(base64::engine::general_purpose::STANDARD.encode(&png_bytes))
}

/// 导出完整分辨率图片到指定路径
#[tauri::command]
pub fn export_image(config: TailConfig, output_path: String) -> Result<(), String> {
    // 参数校验
    let validation = config.validate();
    if !validation.valid {
        return Err(validation.errors.join("; "));
    }

    // 渲染全分辨率
    let img = renderer::render(&config);

    // 写入文件
    img.save(&output_path)
        .map_err(|e| format!("图片保存失败: {}", e))?;

    Ok(())
}

/// 参数校验
#[tauri::command]
pub fn validate_config(config: TailConfig) -> Result<ValidationResult, String> {
    Ok(config.validate())
}

/// 用户预设文件路径：%LOCALAPPDATA%/osu-mania-tail-maker/presets.json
fn user_presets_path(_app: &tauri::AppHandle) -> PathBuf {
    if let Ok(dir) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(dir).join("osu-mania-tail-maker").join("presets.json")
    } else {
        // 非 Windows 回退到 app data 目录
        _app.path()
            .app_data_dir()
            .expect("无法获取 app data 目录")
            .join("presets.json")
    }
}

/// 从文件加载用户预设
fn load_user_presets(app: &tauri::AppHandle) -> Vec<Preset> {
    let path = user_presets_path(app);
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Vec<Preset>>(&s).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    }
}

/// 获取所有预设（内置 + 用户）
#[tauri::command]
pub fn get_presets(app: tauri::AppHandle) -> Vec<Preset> {
    let builtin = preset::builtin_presets();
    let user = load_user_presets(&app);
    // 内置排前面，用户排后面（去重：用户名与内置同名的用户预设被忽略）
    let builtin_names: std::collections::HashSet<&str> =
        builtin.iter().map(|p| p.name.as_str()).collect();
    let user_filtered: Vec<Preset> = user
        .into_iter()
        .filter(|p| !builtin_names.contains(p.name.as_str()))
        .collect();
    [builtin, user_filtered].concat()
}

/// 保存用户预设到 app data 目录
#[tauri::command]
pub fn save_user_presets(app: tauri::AppHandle, presets: Vec<Preset>) -> Result<(), String> {
    let path = user_presets_path(&app);
    // 确保目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    let json = serde_json::to_string_pretty(&presets).map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("写入失败: {}", e))?;
    Ok(())
}

/// 获取默认配置
#[tauri::command]
pub fn get_default_config() -> TailConfig {
    TailConfig::default_config()
}

/// 缓存目录：%LOCALAPPDATA%/osu-mania-tail-maker/cache
fn cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(dir).join("osu-mania-tail-maker").join("cache")
    } else {
        PathBuf::from("cache")
    }
}

/// 根据配置 JSON 计算 hash 作为缓存文件名
fn config_hash(config: &TailConfig) -> String {
    let json = serde_json::to_string(config).unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    json.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// 渲染预设缩略图（带磁盘缓存）
/// 以第一个非透明行为基准，上方留 50px，下方留 200px
#[tauri::command]
pub fn render_preset_thumbnail(config: TailConfig) -> Result<String, String> {
    let hash = config_hash(&config);
    let cache_path = cache_dir().join(format!("{}.png", hash));

    // 缓存命中
    if cache_path.exists() {
        if let Ok(bytes) = fs::read(&cache_path) {
            return Ok(base64::engine::general_purpose::STANDARD.encode(&bytes));
        }
    }

    // 缓存未命中，渲染预览
    let preview = renderer::render_preview(&config);
    let (w, h) = (preview.width(), preview.height());

    // 找到第一个非透明行
    let mut first_row = 0u32;
    'outer: for y in 0..h {
        for x in 0..w {
            if preview.get_pixel(x, y)[3] > 0 {
                first_row = y;
                break 'outer;
            }
        }
    }

    // 裁剪：上方留 50px，下方留 200px
    let pad_top: u32 = 50;
    let pad_bottom: u32 = 200;
    let crop_top = first_row.saturating_sub(pad_top);
    let crop_bottom = (first_row + pad_bottom).min(h);
    let crop_h = crop_bottom.saturating_sub(crop_top);

    let cropped: image::RgbaImage = if crop_h > 0 {
        ImageBuffer::from_fn(w, crop_h, |x, y| {
            *preview.get_pixel(x, crop_top + y)
        })
    } else {
        ImageBuffer::from_pixel(w, 1, image::Rgba([0, 0, 0, 0]))
    };

    // 编码 PNG
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    image::DynamicImage::ImageRgba8(cropped)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("PNG 编码失败: {}", e))?;

    // 写入缓存
    if let Some(parent) = cache_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&cache_path, &png_bytes);

    Ok(base64::engine::general_purpose::STANDARD.encode(&png_bytes))
}
