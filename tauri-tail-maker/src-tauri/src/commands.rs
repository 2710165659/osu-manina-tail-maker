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
use std::path::{Path, PathBuf};
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

/// 获取外部工具 exe 的路径
/// 开发模式下返回 tauri-tail-maker-external/target/release/tail-maker-external.exe
/// （release 不存在时 fallback 到 debug）
/// 打包模式下返回资源目录中的 tail-maker-external.exe
#[tauri::command]
pub fn get_external_tool_path(app: tauri::AppHandle) -> Result<String, String> {
    // 开发模式：从项目目录获取
    if cfg!(debug_assertions) {
        // current_dir 在 tauri dev 下通常是 src-tauri/，
        // 往上 2 层到 workspace root
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("获取当前目录失败: {}", e))?;
        let workspace_root = current_dir
            .parent()
            .ok_or("无法获取项目根目录")?
            .parent()
            .ok_or("无法获取工作区根目录")?;

        // Cargo workspace 构建产物统一在 workspace_root/target/ 下
        for profile in &["release", "debug"] {
            let exe = workspace_root.join("target").join(profile).join("tail-maker-external.exe");
            if exe.exists() {
                return Ok(exe.to_string_lossy().to_string());
            }
        }
        return Err("开发模式下未找到小工具，请先编译 tauri-tail-maker-external".to_string());
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

// ---------------------------------------------------------------------------
// 工具箱命令 —— 包装 shared 库
// ---------------------------------------------------------------------------

// ---- Tail repair (面尾修复, NoteImage#L) ----

/// Lazer 面尾修复（皮肤文件夹模式，始终备份）
#[tauri::command]
pub async fn repair_lazer_tail_folder(folder_path: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dir = PathBuf::from(&folder_path);
        if !dir.is_dir() {
            return Err("指定的路径不是有效的文件夹".to_string());
        }
        let backup_dir = dir.join("_backup");
        shared::lazer_repair::execute_lazer_tail_repair(&dir, &backup_dir)
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

/// Lazer 皮肤修复（osk 文件模式，始终备份）
#[tauri::command]
pub async fn repair_lazer_osk(osk_path: String, modes: Vec<String>) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = PathBuf::from(&osk_path);
        if !path.exists() || !path.is_file() {
            return Err("osk 文件不存在".to_string());
        }

        let mode_set: std::collections::HashSet<&str> = modes.iter().map(|s| s.as_str()).collect();
        let mut log: Vec<String> = Vec::new();
        log.push("开始 osk 文件修复...".to_string());

        let backup_dir = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("_backup");

        shared::osk_ops::repair_osk(&path, &mode_set, &backup_dir, &mut log)?;
        log.push("osk 修复完成！".to_string());

        Ok(log)
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

// ---- Key image repair (KeyImage# + KeyImage#D) ----

/// Key 图片修复（皮肤文件夹模式，始终备份）
#[tauri::command]
pub async fn repair_key_image_folder(folder_path: String, mode: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dir = PathBuf::from(&folder_path);
        if !dir.is_dir() {
            return Err("指定的路径不是有效的文件夹".to_string());
        }
        let backup_dir = dir.join("_backup");
        shared::lazer_repair::execute_lazer_key_repair(&dir, &backup_dir, &mode)
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

// ---- Throw length ----

/// 获取皮肤的投长度信息（按文件夹读取 skin.ini）
#[tauri::command]
pub async fn get_skin_throw_info(folder_path: String) -> Result<Vec<shared::throw_info::SkinThrowInfo>, String> {
    shared::throw_info::get_throw_info(Path::new(&folder_path))
}

/// 按需计算 lazer 投长度（仅对指定 stems 调用 repair_tail_image）
#[tauri::command]
pub async fn compute_lazer_throws(folder_path: String, stems: Vec<String>) -> Result<Vec<(String, u32)>, String> {
    shared::throw_info::compute_lazer_throws(Path::new(&folder_path), &stems)
}

/// 计算单个 key 的 lazer 投长度（逐 key 异步调用，传入各自的 column_width）
#[tauri::command]
pub async fn compute_lazer_throw_single(folder_path: String, stem: String, column_width: u32) -> u32 {
    shared::throw_info::compute_lazer_throw_single(Path::new(&folder_path), &stem, column_width)
}

/// 获取图片-键数-轨道关联信息（供预设替换列表）
#[tauri::command]
pub async fn get_image_key_info(folder_path: String) -> Result<Vec<shared::throw_info::ImageKeyInfo>, String> {
    shared::throw_info::get_image_key_info(Path::new(&folder_path))
}

/// 获取 Key/KeyD stem 列表（供 Key/KeyD 修复列表）
#[tauri::command]
pub async fn get_keyd_list(folder_path: String) -> Result<Vec<shared::throw_info::KeydStemInfo>, String> {
    shared::throw_info::get_keyd_list(Path::new(&folder_path))
}

/// 修改投长度（皮肤文件夹模式，始终备份）
#[tauri::command]
pub async fn modify_skin_throw_length(
    folder_path: String,
    keys: Vec<u32>,
    throws: Vec<u32>,
    mode: String,
) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if keys.len() != throws.len() || keys.is_empty() {
            return Err("keys 与 throws 长度不匹配或为空".to_string());
        }

        let dir = PathBuf::from(&folder_path);
        let backup_dir = dir.join("_backup");

        let throw_map: std::collections::HashMap<u32, u32> = keys
            .iter()
            .zip(throws.iter())
            .map(|(&k, &t)| (k, t))
            .collect();

        // Collect column widths from skin.ini
        let ini_path = dir.join("skin.ini");
        let skin_ini = shared::skin_ini::parse_skin_ini(&ini_path)?;
        let mut column_widths: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
        for section in &skin_ini.mania_sections {
            if throw_map.contains_key(&section.keys) {
                column_widths.entry(section.keys).or_insert(section.column_width);
            }
        }

        shared::throw_info::execute_throw_modification(&dir, &throw_map, &backup_dir, &mode, &column_widths)
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

/// 获取尾部预览图 base64
#[tauri::command]
pub async fn get_tail_preview(folder_path: String, stem: String) -> Result<String, String> {
    shared::throw_info::get_tail_preview_base64(Path::new(&folder_path), &stem)
}

/// 工具箱"一键修改面尾"命令
#[derive(Debug, serde::Serialize)]
pub struct ToolboxConvertResult {
    pub success: bool,
    pub message: String,
    pub logs: Vec<String>,
}

#[tauri::command]
pub async fn convert_tail_toolbox(
    folder_path: String,
    skin_mode: String,
    work_mode: String,
    throws: Vec<(u32, u32)>,
    presets: Vec<(String, String)>,
    keyd_stems: Vec<String>,
) -> Result<ToolboxConvertResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let skin_dir = PathBuf::from(&folder_path);

        if skin_mode == "osk" {
            // osk mode: extract, operate, repack
            let path = PathBuf::from(&folder_path);
            if !path.exists() || !path.is_file() {
                return Ok(ToolboxConvertResult { success: false, message: "osk 文件不存在".to_string(), logs: vec![] });
            }

            let parent = path.parent().unwrap_or_else(|| Path::new("."));
            let work_dir = parent.join(format!(
                "_osk_work_{}",
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()
            ));

            // Extract
            let osk_file = fs::File::open(&path).map_err(|e| format!("打开 osk 文件失败: {}", e))?;
            let mut archive = zip::ZipArchive::new(osk_file).map_err(|e| format!("读取 zip 失败: {}", e))?;
            fs::create_dir_all(&work_dir).map_err(|e| format!("创建临时目录失败: {}", e))?;
            for i in 0..archive.len() {
                let mut file = archive.by_index(i).map_err(|e| format!("读取 zip 条目失败: {}", e))?;
                let name = file.name().to_string();
                if name.ends_with('/') || name.ends_with('\\') { fs::create_dir_all(work_dir.join(&name)).ok(); continue; }
                if let Some(p) = work_dir.join(&name).parent() { fs::create_dir_all(p).ok(); }
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).map_err(|e| format!("读取条目内容失败: {}", e))?;
                fs::write(work_dir.join(&name), &buf).map_err(|e| format!("写入临时文件失败: {}", e))?;
            }

            // Detect nested directory
            let extract_root = work_dir.clone();
            let mut actual_skin_dir = extract_root.clone();
            if !actual_skin_dir.join("skin.ini").exists() {
                let subdirs: Vec<PathBuf> = fs::read_dir(&actual_skin_dir)
                    .map_err(|e| format!("读取临时目录失败: {}", e))?
                    .filter_map(|e| e.ok()).filter(|e| e.path().is_dir() && !e.file_name().to_string_lossy().starts_with('_'))
                    .map(|e| e.path()).collect();
                if subdirs.len() == 1 && subdirs[0].join("skin.ini").exists() {
                    actual_skin_dir = subdirs[0].clone();
                }
            }

            let backup_dir = actual_skin_dir.join("_backup");
            match shared::tail_toolbox::execute_toolbox(
                &actual_skin_dir, &work_mode, &throws, &presets, &keyd_stems, &backup_dir,
            ) {
                Ok(mut log) => {
                    // Repack
                    let temp_osk = path.with_extension("osk.tmp");
                    let temp_file = fs::File::create(&temp_osk).map_err(|e| format!("创建临时 osk 失败: {}", e))?;
                    let mut writer = zip::ZipWriter::new(temp_file);
                    shared::osk_ops::add_files_to_zip(&work_dir, &work_dir, &mut writer)?;
                    writer.finish().map_err(|e| format!("完成 zip 写入失败: {}", e))?;
                    let _ = fs::remove_dir_all(&extract_root);
                    fs::rename(&temp_osk, &path).map_err(|e| format!("替换原 osk 文件失败: {}", e))?;
                    log.push("已重新打包 osk 文件".to_string());
                    Ok(ToolboxConvertResult { success: true, message: "修改完成".to_string(), logs: log })
                }
                Err(e) => {
                    let _ = fs::remove_dir_all(&extract_root);
                    Ok(ToolboxConvertResult { success: false, message: e, logs: vec![] })
                }
            }
        } else {
            // Folder mode
            let backup_dir = skin_dir.join("_backup");
            match shared::tail_toolbox::execute_toolbox(
                &skin_dir, &work_mode, &throws, &presets, &keyd_stems, &backup_dir,
            ) {
                Ok(log) => Ok(ToolboxConvertResult { success: true, message: "修改完成".to_string(), logs: log }),
                Err(e) => Ok(ToolboxConvertResult { success: false, message: e, logs: vec![] }),
            }
        }
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

/// 加载预设图片（工具箱用：皮肤目录 + 系统内置 + 用户预设）
#[tauri::command]
pub async fn load_presets(app: tauri::AppHandle, skin_root: String) -> Vec<shared::preset_loader::PresetInfo> {
    let mut all: Vec<shared::preset_loader::PresetInfo> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // 1. 皮肤目录下的 presets/ 文件夹
    let skin_root_path = PathBuf::from(&skin_root);
    for p in shared::preset_loader::load_presets_from_dir(&skin_root_path) {
        seen.insert(p.name.clone());
        all.push(p);
    }

    // 2. 系统内置 + 用户预设（渲染缩略图 → base64 data URL）
    let builtin = preset::builtin_presets();
    let user = load_user_presets(&app);
    let builtin_names: std::collections::HashSet<&str> =
        builtin.iter().map(|p| p.name.as_str()).collect();
    let user_filtered: Vec<Preset> = user
        .into_iter()
        .filter(|p| !builtin_names.contains(p.name.as_str()))
        .collect();
    let app_presets: Vec<Preset> = [builtin, user_filtered].concat();

    for p in &app_presets {
        if seen.contains(&p.name) { continue; }
        seen.insert(p.name.clone());
        // Render thumbnail to get base64
        let b64 = render_preset_thumbnail_sync(&p.config);
        all.push(shared::preset_loader::PresetInfo {
            name: p.name.clone(),
            image_path: format!("data:image/png;base64,{}", b64),
        });
    }

    all
}

/// 同步渲染预设缩略图（带缓存）
fn render_preset_thumbnail_sync(config: &TailConfig) -> String {
    let hash = config_hash(config);
    let cache_path = cache_dir().join(format!("{}.png", hash));

    // 缓存命中
    if cache_path.exists() {
        if let Ok(bytes) = fs::read(&cache_path) {
            return base64::engine::general_purpose::STANDARD.encode(&bytes);
        }
    }

    // 渲染全尺寸预览 → 裁剪缩略图
    let preview = renderer::render_preview(config);
    let cropped = shared::image_utils::crop_preset_thumbnail(&preview);

    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    image::DynamicImage::ImageRgba8(cropped)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .ok();

    if let Some(parent) = cache_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&cache_path, &png_bytes);

    base64::engine::general_purpose::STANDARD.encode(&png_bytes)
}

// ---- Validator ----

/// 皮肤文件校验
#[tauri::command]
pub async fn validate_skin_files_cmd(folder_path: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dir = PathBuf::from(&folder_path);
        if !dir.is_dir() {
            return Err("指定的路径不是有效的文件夹".to_string());
        }

        let ini_path = dir.join("skin.ini");
        if !ini_path.exists() {
            return Err("未找到 skin.ini 文件".to_string());
        }

        let skin_ini = shared::skin_ini::parse_skin_ini(&ini_path)?;
        let missing = shared::skin_validator::validate_skin_files(&skin_ini);

        if missing.is_empty() {
            Ok(vec!["所有图片文件均存在 ✓".to_string()])
        } else {
            let mut log: Vec<String> = vec![format!("发现 {} 个缺失文件:", missing.len())];
            for m in &missing {
                let keys_str: Vec<String> = m.keys.iter().map(|k| format!("{}k", k)).collect();
                log.push(format!("  ✗ [{}] {} (引用自: {})", m.image_type, m.stem, keys_str.join(", ")));
            }
            Ok(log)
        }
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

/// 校验文件夹是否为有效皮肤目录（包含 skin.ini）
#[tauri::command]
pub fn check_skin_ini(folder_path: String) -> Result<bool, String> {
    let dir = PathBuf::from(&folder_path);
    if !dir.is_dir() {
        return Ok(false);
    }
    Ok(dir.join("skin.ini").is_file())
}
