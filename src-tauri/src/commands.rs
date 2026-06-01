use crate::config::{Preset, TailConfig, ValidationResult};
use crate::preset;
use crate::renderer;
use base64::Engine;
use std::io::Cursor;

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

/// 获取所有预设（内置 + 用户）
#[tauri::command]
pub fn get_presets() -> Vec<Preset> {
    preset::builtin_presets()
}

/// 获取默认配置
#[tauri::command]
pub fn get_default_config() -> TailConfig {
    TailConfig::default_config()
}
