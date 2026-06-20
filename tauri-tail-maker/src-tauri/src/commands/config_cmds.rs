use crate::config::{Preset, TailConfig};
use crate::preset;
use crate::renderer;
use crate::tools::image_parser;
use base64::Engine;
use image::ImageBuffer;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::path::PathBuf;
use tauri::Manager;

/// 用默认浏览器打开 URL（包装 shared 库）
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    shared::open_url(&url)
}

/// 渲染预览图（全宽，最多 1000 行），返回 base64 编码的 PNG
#[tauri::command]
pub async fn render_preview(config: TailConfig) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let validation = config.validate();
        if !validation.valid {
            return Err(validation.errors.join("; "));
        }

        let preview = renderer::render_preview(&config);

        let mut png_bytes = Vec::new();
        let mut cursor = Cursor::new(&mut png_bytes);
        image::DynamicImage::ImageRgba8(preview)
            .write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| format!("PNG 编码失败: {}", e))?;

        Ok(base64::engine::general_purpose::STANDARD.encode(&png_bytes))
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

/// 导出完整分辨率图片到指定路径
#[tauri::command]
pub async fn export_image(config: TailConfig, output_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let validation = config.validate();
        if !validation.valid {
            return Err(validation.errors.join("; "));
        }

        let img = renderer::render(&config);

        img.save(&output_path)
            .map_err(|e| format!("图片保存失败: {}", e))?;

        Ok(())
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

/// 用户预设文件路径：%LOCALAPPDATA%/osu-mania-tail-maker/presets.json
pub fn user_presets_path(_app: &tauri::AppHandle) -> PathBuf {
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
pub fn load_user_presets(app: &tauri::AppHandle) -> Vec<Preset> {
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
pub fn cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(dir).join("osu-mania-tail-maker").join("cache")
    } else {
        PathBuf::from("cache")
    }
}

/// 根据配置 JSON 计算 hash 作为缓存文件名
pub fn config_hash(config: &TailConfig) -> String {
    let json = serde_json::to_string(config).unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    json.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// 渲染预设缩略图（带磁盘缓存）
#[tauri::command]
pub async fn render_preset_thumbnail(config: TailConfig) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let hash = config_hash(&config);
        let cache_path = cache_dir().join(format!("{}.png", hash));

        if cache_path.exists() {
            if let Ok(bytes) = fs::read(&cache_path) {
                return Ok(base64::engine::general_purpose::STANDARD.encode(&bytes));
            }
        }

        let preview = renderer::render_preview(&config);
        let cropped = shared::image_utils::crop_preset_thumbnail(&preview);

        let mut png_bytes = Vec::new();
        let mut cursor = Cursor::new(&mut png_bytes);
        image::DynamicImage::ImageRgba8(cropped)
            .write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| format!("PNG 编码失败: {}", e))?;

        if let Some(parent) = cache_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&cache_path, &png_bytes);

        Ok(base64::engine::general_purpose::STANDARD.encode(&png_bytes))
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

/// 导出完整分辨率图片并返回 base64 编码的 PNG 字节
#[tauri::command]
pub async fn export_image_bytes(config: TailConfig) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let validation = config.validate();
        if !validation.valid {
            return Err(validation.errors.join("; "));
        }

        let img = renderer::render(&config);

        let mut png_bytes = Vec::new();
        let mut cursor = Cursor::new(&mut png_bytes);
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| format!("PNG 编码失败: {}", e))?;

        Ok(base64::engine::general_purpose::STANDARD.encode(&png_bytes))
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

/// 解析图片为预设配置
#[tauri::command]
pub async fn parse_image_to_preset(image_path: String) -> Result<(TailConfig, Vec<String>, Vec<String>), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = PathBuf::from(&image_path);
        if !path.exists() {
            return Err("图片文件不存在".to_string());
        }

        let result = image_parser::parse_image(&path)
            .map_err(|e| format!("解析失败: {}", e))?;

        Ok((result.config, result.warnings, result.debug_info))
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

/// 读取图片顶部 500px 并返回 base64 编码的 PNG
#[tauri::command]
pub async fn get_image_preview_top(image_path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = PathBuf::from(&image_path);
        if !path.exists() {
            return Err("图片文件不存在".to_string());
        }

        let img = image::open(&path)
            .map_err(|e| format!("读取图片失败: {}", e))?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();

        let crop_h = h.min(500);
        let cropped = ImageBuffer::from_fn(w, crop_h, |x, y| {
            *rgba.get_pixel(x, y)
        });

        let mut png_bytes = Vec::new();
        let mut cursor = Cursor::new(&mut png_bytes);
        image::DynamicImage::ImageRgba8(cropped)
            .write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| format!("PNG 编码失败: {}", e))?;

        Ok(base64::engine::general_purpose::STANDARD.encode(&png_bytes))
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

