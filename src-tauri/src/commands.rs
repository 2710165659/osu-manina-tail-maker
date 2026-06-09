use crate::config::{Preset, TailConfig, ValidationResult};
use crate::preset;
use crate::renderer;
use crate::tools::image_parser;
use base64::Engine;
use image::ImageBuffer;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{Cursor, Read, Write};
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

/// 用默认浏览器打开 URL
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", &url])
            .spawn()
            .map_err(|e| format!("打开链接失败: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开链接失败: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开链接失败: {}", e))?;
    }
    Ok(())
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

/// 导出完整分辨率图片并返回 base64 编码的 PNG 字节
/// 用于需要获取图片字节数据的场景（如嵌入脚本）
#[tauri::command]
pub fn export_image_bytes(config: TailConfig) -> Result<String, String> {
    // 参数校验
    let validation = config.validate();
    if !validation.valid {
        return Err(validation.errors.join("; "));
    }

    // 渲染全分辨率
    let img = renderer::render(&config);

    // 编码为 PNG
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("PNG 编码失败: {}", e))?;

    Ok(base64::engine::general_purpose::STANDARD.encode(&png_bytes))
}

/// 解析图片为预设配置
/// 返回 (配置, 警告列表, 调试信息)
#[tauri::command]
pub fn parse_image_to_preset(image_path: String) -> Result<(TailConfig, Vec<String>, Vec<String>), String> {
    let path = PathBuf::from(&image_path);
    if !path.exists() {
        return Err("图片文件不存在".to_string());
    }

    let result = image_parser::parse_image(&path)
        .map_err(|e| format!("解析失败: {}", e))?;

    Ok((result.config, result.warnings, result.debug_info))
}

/// 读取图片顶部 500px 并返回 base64 编码的 PNG
#[tauri::command]
pub fn get_image_preview_top(image_path: String) -> Result<String, String> {
    let path = PathBuf::from(&image_path);
    if !path.exists() {
        return Err("图片文件不存在".to_string());
    }

    let img = image::open(&path)
        .map_err(|e| format!("读取图片失败: {}", e))?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    // 截取顶部 500px
    let crop_h = h.min(500);
    let cropped = image::ImageBuffer::from_fn(w, crop_h, |x, y| {
        *rgba.get_pixel(x, y)
    });

    // 编码为 PNG
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    image::DynamicImage::ImageRgba8(cropped)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("PNG 编码失败: {}", e))?;

    Ok(base64::engine::general_purpose::STANDARD.encode(&png_bytes))
}

/// 获取外部工具 exe 的路径
/// 开发模式下返回 src-tauri-tools/target/release/tail-maker-external.exe
/// 打包模式下返回资源目录中的 tail-maker-external.exe
#[tauri::command]
pub fn get_external_tool_path(app: tauri::AppHandle) -> Result<String, String> {
    // 开发模式：从项目目录获取
    if cfg!(debug_assertions) {
        let project_dir = std::env::current_dir()
            .map_err(|e| format!("获取当前目录失败: {}", e))?
            .parent()
            .ok_or("无法获取项目根目录")?
            .to_path_buf();
        let tool_path = project_dir
            .join("src-tauri-tools")
            .join("target")
            .join("release")
            .join("tail-maker-external.exe");

        if tool_path.exists() {
            return Ok(tool_path.to_string_lossy().to_string());
        }
        return Err("开发模式下未找到小工具，请先运行 npm run build:tools".to_string());
    }

    // 打包模式：从资源目录获取
    let resource_path = app.path().resource_dir()
        .map_err(|e| format!("获取资源目录失败: {}", e))?
        .join("tail-maker-external.exe");

    if resource_path.exists() {
        return Ok(resource_path.to_string_lossy().to_string());
    }

    Err("未找到外部工具".to_string())
}

/// 复制外部工具和预设图片到目标位置
/// target_path: 目标文件夹路径（.osk 解压后的目录或皮肤文件夹）
/// preset_images: 预设图片列表，每项包含 (name, image_bytes)
#[tauri::command]
pub fn copy_external_tool_with_presets(
    app: tauri::AppHandle,
    target_path: String,
    preset_images: Vec<(String, Vec<u8>)>,
) -> Result<String, String> {
    // 获取外部工具路径
    let tool_path = get_external_tool_path(app)?;

    // 构建目标路径
    let target_dir = PathBuf::from(&target_path);
    let scripts_dir = target_dir.join("scripts");
    let presets_dir = scripts_dir.join("presets");
    let target_file = scripts_dir.join("tail-maker-external.exe");

    // 创建目录
    fs::create_dir_all(&scripts_dir)
        .map_err(|e| format!("创建 scripts 目录失败: {}", e))?;
    if !preset_images.is_empty() {
        fs::create_dir_all(&presets_dir)
            .map_err(|e| format!("创建 presets 目录失败: {}", e))?;
    }

    // 复制 exe
    fs::copy(&tool_path, &target_file)
        .map_err(|e| format!("复制文件失败: {}", e))?;

    // 保存预设图片
    for (name, image_bytes) in preset_images {
        let image_path = presets_dir.join(format!("{}.png", name));
        fs::write(&image_path, &image_bytes)
            .map_err(|e| format!("保存预设图片 {} 失败: {}", name, e))?;
    }

    Ok(target_file.to_string_lossy().to_string())
}

/// 将外部工具和预设图片添加到 osk 文件中
/// osk_path: osk 文件路径
/// preset_images: 预设图片列表，每项包含 (name, image_bytes)
#[tauri::command]
pub fn add_external_tool_to_osk_with_presets(
    app: tauri::AppHandle,
    osk_path: String,
    preset_images: Vec<(String, Vec<u8>)>,
) -> Result<String, String> {
    // 获取外部工具路径
    let tool_path = get_external_tool_path(app)?;

    // 读取外部工具文件
    let tool_bytes = fs::read(&tool_path)
        .map_err(|e| format!("读取外部工具失败: {}", e))?;

    let osk_path = PathBuf::from(&osk_path);
    if !osk_path.exists() {
        return Err("osk 文件不存在".to_string());
    }

    // 读取原始 osk 文件
    let osk_file = fs::File::open(&osk_path)
        .map_err(|e| format!("打开 osk 文件失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(osk_file)
        .map_err(|e| format!("读取 zip 失败: {}", e))?;

    // 创建新的 osk 文件（临时文件）
    let temp_path = osk_path.with_extension("osk.tmp");
    let temp_file = fs::File::create(&temp_path)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;
    let mut new_archive = zip::ZipWriter::new(temp_file);

    // 复制原有文件（跳过已存在的 scripts 目录内容）
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?;
        let name = file.name().to_string();

        // 跳过 scripts 目录下的文件
        if name.starts_with("scripts/") || name.starts_with("scripts\\") {
            continue;
        }

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(file.compression());
        new_archive.start_file(&name, options)
            .map_err(|e| format!("创建 zip 条目失败: {}", e))?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| format!("读取文件内容失败: {}", e))?;
        new_archive.write_all(&buffer)
            .map_err(|e| format!("写入文件内容失败: {}", e))?;
    }

    // 添加外部工具
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);
    new_archive.start_file("scripts/tail-maker-external.exe", options)
        .map_err(|e| format!("创建 exe 条目失败: {}", e))?;
    new_archive.write_all(&tool_bytes)
        .map_err(|e| format!("写入 exe 失败: {}", e))?;

    // 添加预设图片
    for (name, image_bytes) in preset_images {
        let image_name = format!("scripts/presets/{}.png", name);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        new_archive.start_file(&image_name, options)
            .map_err(|e| format!("创建预设图片条目 {} 失败: {}", name, e))?;
        new_archive.write_all(&image_bytes)
            .map_err(|e| format!("写入预设图片 {} 失败: {}", name, e))?;
    }

    // 完成写入
    new_archive.finish()
        .map_err(|e| format!("完成 zip 写入失败: {}", e))?;

    // 替换原文件
    fs::rename(&temp_path, &osk_path)
        .map_err(|e| format!("替换原文件失败: {}", e))?;

    Ok("scripts/tail-maker-external.exe".to_string())
}
