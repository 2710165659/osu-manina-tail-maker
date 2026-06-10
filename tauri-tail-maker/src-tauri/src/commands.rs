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
/// @2x 后缀标记
fn is2x_tag(path: &std::path::Path) -> &'static str {
    if shared::image_utils::is_2x(path) { " (@2x)" } else { "" }
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    shared::open_url(&url)
}

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
/// 开发模式下返回 tauri-tail-maker-external/target/release/tail-maker-external.exe
/// 打包模式下返回资源目录中的 tail-maker-external.exe
#[tauri::command]
pub fn get_external_tool_path(app: tauri::AppHandle) -> Result<String, String> {
    // 开发模式：从项目目录获取
    // 当前目录为 src-tauri/，上两级到项目根目录，再进入 tauri-tail-maker-external/
    if cfg!(debug_assertions) {
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("获取当前目录失败: {}", e))?;
        let project_root = current_dir
            .parent()
            .ok_or("无法获取项目根目录")?
            .parent()
            .ok_or("无法获取项目根目录")?;
        let tool_path = project_root
            .join("tauri-tail-maker-external")
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

// ---------------------------------------------------------------------------
// 工具箱命令 —— 包装 shared 库
// ---------------------------------------------------------------------------

/// 修改 skin.ini 中某个 [Mania] 节下某个 NoteImage<col>L 的值。
/// 重新读取文件，按 Keys 定位小节，按列号匹配 NoteImage<col>L 行。
fn _update_note_image_l_in_ini(
    ini_path: &PathBuf,
    keys: u32,
    col: u32,
    new_value: &str,
) -> Result<(), String> {
    let content = fs::read_to_string(ini_path)
        .map_err(|e| format!("读取 skin.ini 失败: {}", e))?;

    let target_key = format!("NoteImage{}L", col).to_lowercase();
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines: Vec<String> = Vec::with_capacity(lines.len());
    let mut in_target_mania = false;
    let mut section_keys: Option<u32> = None;
    let mut found = false;

    for line in &lines {
        let trimmed = line.trim();

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_target_mania = trimmed.eq_ignore_ascii_case("[Mania]");
            section_keys = None;
        }

        if in_target_mania {
            // Keys:
            if let Some(val) = shared::skin_ini::try_parse_key(trimmed, "Keys:") {
                section_keys = val.parse::<u32>().ok();
            }

            // 确认是目标小节 + 匹配 NoteImage<col>L
            if section_keys == Some(keys) && trimmed.to_lowercase().starts_with(&target_key) {
                if let Some(colon_pos) = line.find(':') {
                    let before = &line[..colon_pos + 1];
                    new_lines.push(format!("{} {}", before, new_value));
                    found = true;
                    continue;
                }
            }
        }

        new_lines.push(line.to_string());
    }

    if !found {
        return Err(format!(
            "未在 {}k 小节中找到 NoteImage{}L 行",
            keys, col
        ));
    }

    fs::write(ini_path, new_lines.join("\n"))
        .map_err(|e| format!("写入 skin.ini 失败: {}", e))?;
    Ok(())
}

/// 修改 skin.ini 中某个 [Mania] 节下 KeyImage# 或 KeyImage#D 的值。
/// `is_d`: true 匹配 KeyImage#D，false 匹配 KeyImage#。
fn _update_key_image_in_ini(
    ini_path: &PathBuf,
    keys: u32,
    col: u32,
    is_d: bool,
    new_value: &str,
) -> Result<(), String> {
    let content = fs::read_to_string(ini_path)
        .map_err(|e| format!("读取 skin.ini 失败: {}", e))?;

    // KeyImage#D 必须在 KeyImage# 之前检测（D 后缀更长）
    let target_prefix = if is_d {
        format!("KeyImage{}D", col).to_lowercase()
    } else {
        format!("KeyImage{}", col).to_lowercase()
    };
    let suffix_char = if is_d { "D" } else { "" };
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines: Vec<String> = Vec::with_capacity(lines.len());
    let mut in_target_mania = false;
    let mut section_keys: Option<u32> = None;
    let mut found = false;

    for line in &lines {
        let trimmed = line.trim();

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_target_mania = trimmed.eq_ignore_ascii_case("[Mania]");
            section_keys = None;
        }

        if in_target_mania {
            if let Some(val) = shared::skin_ini::try_parse_key(trimmed, "Keys:") {
                section_keys = val.parse::<u32>().ok();
            }

            if section_keys == Some(keys) {
                let lower = trimmed.to_lowercase();
                // 精确匹配：KeyImage{col}D: 或 KeyImage{col}:（排除 KeyImage{col}D 当 is_d=false）
                let exact_prefix = format!("KeyImage{}:", col).to_lowercase();
                let exact_prefix_d = format!("KeyImage{}D:", col).to_lowercase();
                let matched = if is_d {
                    lower.starts_with(&exact_prefix_d)
                } else {
                    lower.starts_with(&exact_prefix) && !lower.starts_with(&exact_prefix_d)
                };
                if matched {
                    if let Some(colon_pos) = line.find(':') {
                        let before = &line[..colon_pos + 1];
                        new_lines.push(format!("{} {}", before, new_value));
                        found = true;
                        continue;
                    }
                }
            }
        }

        new_lines.push(line.to_string());
    }

    if !found {
        let label = if is_d { "KeyImage#D" } else { "KeyImage#" };
        return Err(format!(
            "未在 {}k 小节中找到 {}{} 行",
            keys, label, col
        ));
    }

    fs::write(ini_path, new_lines.join("\n"))
        .map_err(|e| format!("写入 skin.ini 失败: {}", e))?;
    Ok(())
}

// ---- Tail repair (面尾修复, NoteImage#L) ----

/// Lazer 面尾修复（皮肤文件夹模式）
///
/// 对所有 NoteImage#L 面尾图片执行：宽→ColumnWidth×1.6，高等比缩放。
/// >32800 裁切底部，<32800 底部平铺补足。
/// 不同 ColumnWidth 但同一图片 → 复制图片，修改 skin.ini。
#[tauri::command]
pub fn repair_lazer_tail_folder(folder_path: String, backup: bool) -> Result<Vec<String>, String> {
    let dir = PathBuf::from(&folder_path);
    if !dir.is_dir() {
        return Err("指定的路径不是有效的文件夹".to_string());
    }

    let ini_path = dir.join("skin.ini");
    if !ini_path.exists() {
        return Err("未找到 skin.ini 文件".to_string());
    }

    let mut log: Vec<String> = Vec::new();
    let add_log = |log: &mut Vec<String>, msg: &str| log.push(msg.to_string());
    let ts_dir = backup_timestamp();

    let skin_ini = shared::skin_ini::parse_skin_ini(&ini_path)?;
    add_log(
        &mut log,
        &format!("解析到 {} 个 [Mania] 小节", skin_ini.mania_sections.len()),
    );

    let groups = shared::skin_ini::group_note_image_l_by_stem(&skin_ini.mania_sections);
    add_log(&mut log, &format!("共 {} 组不同的面尾图片", groups.len()));

    // 收集需要 ini 修改的项（最后统一处理以保持行号稳定）
    let mut ini_patches: Vec<(u32, u32, String)> = Vec::new(); // (keys, col, new_stem)

    for (stem, refs) in &groups {
        let image_path = shared::skin_ini::find_image_file(&dir, stem)
            .ok_or_else(|| format!("找不到面尾图片: {}", stem))?;

        let img = image::open(&image_path)
            .map_err(|e| format!("读取图片失败 {}: {}", stem, e))?
            .to_rgba8();

        // 不同 (keys, cw) 组合
        let mut unique_cw: Vec<u32> = refs.iter().map(|(_, cw)| *cw).collect();
        unique_cw.sort();
        unique_cw.dedup();

        add_log(&mut log, &format!("处理: {} ({} 个键数/列宽组合)", stem, refs.len()));

        if unique_cw.len() == 1 {
            let cw = unique_cw[0];
            let repaired = shared::tail_repair::repair_tail_image(&img, cw);
            backup_file(&dir, &image_path, backup, &ts_dir)?;
            repaired
                .save(&image_path)
                .map_err(|e| format!("保存失败 {}: {}", stem, e))?;
            add_log(&mut log, &format!("  ✓ {}: cw={} → {}×{}{}", stem, cw, repaired.width(), repaired.height(), is2x_tag(&image_path)));
        } else {
            // 多个 ColumnWidth — 第一个覆盖原文件，其余复制
            let ext = image_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png");
            let first_cw = unique_cw[0];
            let repaired = shared::tail_repair::repair_tail_image(&img, first_cw);
            backup_file(&dir, &image_path, backup, &ts_dir)?;
            repaired
                .save(&image_path)
                .map_err(|e| format!("保存失败 {}: {}", stem, e))?;
            add_log(&mut log, &format!("  ✓ {}: cw={} → {}×{}{}", stem, first_cw, repaired.width(), repaired.height(), is2x_tag(&image_path)));

            for &cw in &unique_cw[1..] {
                let repaired_cw = shared::tail_repair::repair_tail_image(&img, cw);
                let copy_stem = format!("{}_cw{}", stem, cw);
                let copy_path = dir.join(format!("{}.{}", copy_stem, ext));
                repaired_cw
                    .save(&copy_path)
                    .map_err(|e| format!("保存副本失败 {}: {}", copy_stem, e))?;

                // 找到对应 cw 的列并记录 ini 修改（每个 NoteImage#L 行都需要更新）
                for (keys_k, _) in refs.iter().filter(|(_, c)| *c == cw) {
                    for sec in &skin_ini.mania_sections {
                        if sec.keys == *keys_k && sec.column_width == cw {
                            for r in &sec.note_image_ls {
                                if r.name == *stem {
                                    ini_patches.push((*keys_k, r.column, copy_stem.clone()));
                                }
                            }
                        }
                    }
                }
                add_log(&mut log, &format!("  ✓ {}: cw={} → {}×{} (副本: {}){}", stem, cw, repaired_cw.width(), repaired_cw.height(), copy_stem, is2x_tag(&copy_path)));
            }
        }
    }

    // 统一应用 ini 修改
    for (keys, col, new_stem) in &ini_patches {
        _update_note_image_l_in_ini(&ini_path, *keys, *col, new_stem)?;
        add_log(&mut log, &format!("  已更新 NoteImage{}L → {}", col, new_stem));
    }

    add_log(&mut log, "修复完成！");
    Ok(log)
}

/// osk 文件解压、修复后再打包
fn repair_in_work_dir(
    osk_path: &Path,
    backup: bool,
    repair_modes: &std::collections::HashSet<&str>,
    log: &mut Vec<String>,
    add_log: &dyn Fn(&mut Vec<String>, &str),
) -> Result<(), String> {
    let parent = osk_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let work_dir = parent.join(format!(
        "_osk_repair_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));

    // 1. 解压 osk → work_dir
    let osk_file = fs::File::open(osk_path)
        .map_err(|e| format!("打开 osk 文件失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(osk_file)
        .map_err(|e| format!("读取 zip 失败: {}", e))?;

    fs::create_dir_all(&work_dir)
        .map_err(|e| format!("创建临时目录失败: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?;
        let name = file.name().to_string();
        if name.ends_with('/') || name.ends_with('\\') {
            fs::create_dir_all(work_dir.join(&name)).ok();
            continue;
        }
        if let Some(p) = work_dir.join(&name).parent() {
            fs::create_dir_all(p).ok();
        }
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .map_err(|e| format!("读取条目内容失败: {}", e))?;
        fs::write(work_dir.join(&name), &buf)
            .map_err(|e| format!("写入临时文件失败: {}", e))?;
    }
    add_log(log, &format!("已解压到临时目录"));

    // 检测 osk 解压后是否有一层嵌套目录（如 LazerSkin/skin.ini 而非直接 skin.ini）
    let extract_root = work_dir.clone();  // 保留提取根目录用于清理
    let mut work_dir = extract_root.clone();
    let ini_path = work_dir.join("skin.ini");
    if !ini_path.exists() {
        // 查找正好一个子目录且里面有 skin.ini
        let subdirs: Vec<PathBuf> = fs::read_dir(&work_dir)
            .map_err(|e| format!("读取临时目录失败: {}", e))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir() && !e.file_name().to_string_lossy().starts_with('_'))
            .map(|e| e.path())
            .collect();
        if subdirs.len() == 1 && subdirs[0].join("skin.ini").exists() {
            work_dir = subdirs[0].clone();
            add_log(log, &format!("检测到嵌套目录: {}", work_dir.file_name().unwrap_or_default().to_string_lossy()));
        }
    }

    // 在 work_dir 上修复
    let ini_path = work_dir.join("skin.ini");
    if ini_path.exists() {
        let skin_ini = shared::skin_ini::parse_skin_ini(&ini_path)?;

        // 面尾修复
        if repair_modes.contains("tail") {
            let ts_dir = backup_timestamp();
            add_log(log, "--- 面尾修复 ---");
            let groups = shared::skin_ini::group_note_image_l_by_stem(&skin_ini.mania_sections);
            add_log(log, &format!("共 {} 组不同的面尾图片", groups.len()));
            let mut ini_patches: Vec<(u32, u32, String)> = Vec::new();

            for (stem, refs) in &groups {
                let image_path = match shared::skin_ini::find_image_file(&work_dir, stem) {
                    Some(p) => p,
                    None => {
                        add_log(log, &format!("⚠ 找不到面尾图片: {}", stem));
                        continue;
                    }
                };
                let img = image::open(&image_path)
                    .map_err(|e| format!("读取图片失败 {}: {}", stem, e))?
                    .to_rgba8();

                let mut unique_cw: Vec<u32> = refs.iter().map(|(_, cw)| *cw).collect();
                unique_cw.sort();
                unique_cw.dedup();

                if unique_cw.len() == 1 {
                    let cw = unique_cw[0];
                    let repaired = shared::tail_repair::repair_tail_image(&img, cw);
                    backup_file(&work_dir, &image_path, backup, &ts_dir)?;
                    repaired.save(&image_path)
                        .map_err(|e| format!("保存失败 {}: {}", stem, e))?;
                    add_log(log, &format!("  ✓ {}: cw={} → {}×{}{}", stem, cw, repaired.width(), repaired.height(), is2x_tag(&image_path)));
                } else {
                    let ext = image_path.extension().and_then(|e| e.to_str()).unwrap_or("png");
                    let first_cw = unique_cw[0];
                    let repaired = shared::tail_repair::repair_tail_image(&img, first_cw);
                    backup_file(&work_dir, &image_path, backup, &ts_dir)?;
                    repaired.save(&image_path)
                        .map_err(|e| format!("保存失败 {}: {}", stem, e))?;
                    add_log(log, &format!("  ✓ {}: cw={} → {}×{}{}", stem, first_cw, repaired.width(), repaired.height(), is2x_tag(&image_path)));
                    for &cw in &unique_cw[1..] {
                        let repaired_cw = shared::tail_repair::repair_tail_image(&img, cw);
                        let copy_stem = format!("{}_cw{}", stem, cw);
                        let copy_path = work_dir.join(format!("{}.{}", copy_stem, ext));
                        repaired_cw.save(&copy_path)
                            .map_err(|e| format!("保存副本失败 {}: {}", copy_stem, e))?;

                        // 收集 ini 更新：找到对应 cw 的 keys/col
                        for (keys_k, _cw_k) in refs.iter().filter(|(_, c)| *c == cw) {
                            for sec in &skin_ini.mania_sections {
                                if sec.keys == *keys_k && sec.column_width == cw {
                                    for r in &sec.note_image_ls {
                                        if r.name == *stem {
                                            ini_patches.push((*keys_k, r.column, copy_stem.clone()));
                                        }
                                    }
                                }
                            }
                        }
                        add_log(log, &format!("  ✓ {}: cw={} → {}×{} (副本: {}){}", stem, cw, repaired_cw.width(), repaired_cw.height(), copy_stem, is2x_tag(&copy_path)));
                    }
                }
            }

            // 应用 ini 修改
            for (keys, col, new_stem) in &ini_patches {
                if let Err(e) = _update_note_image_l_in_ini(&ini_path, *keys, *col, new_stem) {
                    add_log(log, &format!("⚠ 更新 skin.ini 失败 ({}k col={}): {}", keys, col, e));
                } else {
                    add_log(log, &format!("  已更新 NoteImage{}L → {}", col, new_stem));
                }
            }
        }

        // Key 修复
        if repair_modes.contains("keyd") {
            let ts_dir = backup_timestamp();
            add_log(log, "--- Key + KeyD 修复 ---");
            let groups = shared::skin_ini::group_key_images_by_stem(&skin_ini.mania_sections);
            add_log(log, &format!("共 {} 组不同的 Key 图片", groups.len()));
            let mut ini_patches: Vec<(u32, u32, bool, String)> = Vec::new();

            for (stem, refs) in &groups {
                let image_path = match shared::skin_ini::find_image_file(&work_dir, stem) {
                    Some(p) => p,
                    None => {
                        add_log(log, &format!("⚠ 找不到 Key 图片: {}", stem));
                        continue;
                    }
                };
                let img = image::open(&image_path)
                    .map_err(|e| format!("读取图片失败 {}: {}", stem, e))?
                    .to_rgba8();

                let mut unique_cw: Vec<u32> = refs.iter().map(|(_, cw)| *cw).collect();
                unique_cw.sort();
                unique_cw.dedup();

                let is_2x = shared::image_utils::is_2x(&image_path);

                if unique_cw.len() == 1 {
                    let cw = unique_cw[0];
                    let repaired = shared::keyd_repair::repair_key_image(&img, cw, is_2x);
                    backup_file(&work_dir, &image_path, backup, &ts_dir)?;
                    repaired.save(&image_path)
                        .map_err(|e| format!("保存失败 {}: {}", stem, e))?;
                    add_log(log, &format!("  ✓ {}: cw={} → {}×{}{}", stem, cw, repaired.width(), repaired.height(), is2x_tag(&image_path)));
                } else {
                    let ext = image_path.extension().and_then(|e| e.to_str()).unwrap_or("png");
                    let first_cw = unique_cw[0];
                    let repaired = shared::keyd_repair::repair_key_image(&img, first_cw, is_2x);
                    backup_file(&work_dir, &image_path, backup, &ts_dir)?;
                    repaired.save(&image_path)
                        .map_err(|e| format!("保存失败 {}: {}", stem, e))?;
                    add_log(log, &format!("  ✓ {}: cw={} → {}×{}{}", stem, first_cw, repaired.width(), repaired.height(), is2x_tag(&image_path)));

                    for &cw in &unique_cw[1..] {
                        let repaired_cw = shared::keyd_repair::repair_key_image(&img, cw, is_2x);
                        let copy_stem = format!("{}_cw{}", stem, cw);
                        let copy_path = work_dir.join(format!("{}.{}", copy_stem, ext));
                        repaired_cw.save(&copy_path)
                            .map_err(|e| format!("保存副本失败 {}: {}", copy_stem, e))?;

                        for (keys_k, _) in refs.iter().filter(|(_, c)| *c == cw) {
                            for sec in &skin_ini.mania_sections {
                                if sec.keys == *keys_k && sec.column_width == cw {
                                    for r in &sec.key_image_ds {
                                        if r.name == *stem {
                                            ini_patches.push((*keys_k, r.column, true, copy_stem.clone()));
                                        }
                                    }
                                    for r in &sec.key_images {
                                        if r.name == *stem {
                                            ini_patches.push((*keys_k, r.column, false, copy_stem.clone()));
                                        }
                                    }
                                }
                            }
                        }
                        add_log(log, &format!("  ✓ {}: cw={} → {}×{} (副本: {}){}", stem, cw, repaired_cw.width(), repaired_cw.height(), copy_stem, is2x_tag(&copy_path)));
                    }
                }
            }

            // 应用 ini 修改
            let ini_path = work_dir.join("skin.ini");
            for (keys, col, is_d, new_stem) in &ini_patches {
                match _update_key_image_in_ini(&ini_path, *keys, *col, *is_d, new_stem) {
                    Ok(()) => {
                        let label = if *is_d { "KeyImage#D" } else { "KeyImage#" };
                        add_log(log, &format!("  已更新 {}{} → {}", label, col, new_stem));
                    }
                    Err(e) => add_log(log, &format!("⚠ {}", e)),
                }
            }
        }
    }

    // 3. 重新打包 → 覆盖原 osk
    let temp_osk = osk_path.with_extension("osk.tmp");
    let temp_file = fs::File::create(&temp_osk)
        .map_err(|e| format!("创建临时 osk 失败: {}", e))?;
    let mut writer = zip::ZipWriter::new(temp_file);

    add_files_to_zip(&work_dir, &work_dir, &mut writer)?;
    writer.finish().map_err(|e| format!("完成 zip 写入失败: {}", e))?;

    // 清理临时目录
    let _ = fs::remove_dir_all(&extract_root);
    // 替换
    fs::rename(&temp_osk, osk_path)
        .map_err(|e| format!("替换原 osk 文件失败: {}", e))?;
    add_log(log, "已重新打包 osk 文件");

    Ok(())
}

/// Lazer 皮肤修复（osk 文件模式）
///
/// 解压 → 修复 → 重新打包。`modes`: ["tail"] / ["keyd"] / ["tail","keyd"]
#[tauri::command]
pub fn repair_lazer_osk(
    osk_path: String,
    backup: bool,
    modes: Vec<String>,
) -> Result<Vec<String>, String> {
    let path = PathBuf::from(&osk_path);
    if !path.exists() || !path.is_file() {
        return Err("osk 文件不存在".to_string());
    }

    let mode_set: std::collections::HashSet<&str> = modes.iter().map(|s| s.as_str()).collect();
    let mut log: Vec<String> = Vec::new();
    let add_log_fn: &dyn Fn(&mut Vec<String>, &str) = &|l, msg| l.push(msg.to_string());

    add_log_fn(&mut log, "开始 osk 文件修复...");
    repair_in_work_dir(&path, backup, &mode_set, &mut log, add_log_fn)?;
    add_log_fn(&mut log, "osk 修复完成！");

    Ok(log)
}

/// 递归将目录下文件添加到 zip（跳过 _backup）
fn add_files_to_zip(
    base: &Path,
    dir: &Path,
    writer: &mut zip::ZipWriter<fs::File>,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        // 跳过 _backup
        if relative.starts_with("_backup") {
            continue;
        }

        if path.is_dir() {
            add_files_to_zip(base, &path, writer)?;
        } else {
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            writer
                .start_file(&*relative, options)
                .map_err(|e| format!("创建 zip 条目失败: {}", e))?;
            let buf = fs::read(&path)
                .map_err(|e| format!("读取文件失败: {}", e))?;
            writer
                .write_all(&buf)
                .map_err(|e| format!("写入 zip 条目失败: {}", e))?;
        }
    }
    Ok(())
}

// ---- Key image repair (KeyImage# + KeyImage#D) ----

/// Key 图片修复（皮肤文件夹模式）
///
/// 对 KeyImage# 和 KeyImage#D 图片执行等比缩放修复算法。
/// `mode`: "keyd" 只修 KeyImage#D，"key" 只修 KeyImage#，"all" 两者都修。
#[tauri::command]
pub fn repair_key_image_folder(
    folder_path: String,
    backup: bool,
    mode: String,
) -> Result<Vec<String>, String> {
    let dir = PathBuf::from(&folder_path);
    if !dir.is_dir() {
        return Err("指定的路径不是有效的文件夹".to_string());
    }

    let ini_path = dir.join("skin.ini");
    if !ini_path.exists() {
        return Err("未找到 skin.ini 文件".to_string());
    }

    let mut log: Vec<String> = Vec::new();
    let add_log = |log: &mut Vec<String>, msg: &str| log.push(msg.to_string());
    let ts_dir = backup_timestamp();

    let skin_ini = shared::skin_ini::parse_skin_ini(&ini_path)?;

    let include_d = mode == "keyd" || mode == "all";
    let include_key = mode == "key" || mode == "all";

    // 按图片茎分组，收集所有 (keys, cw) 引用
    let mut groups = shared::skin_ini::group_key_images_by_stem(&skin_ini.mania_sections);
    // 根据 mode 过滤：只保留包含目标类型引用的组
    if !include_d || !include_key {
        groups.retain(|stem, _| {
            let has_d = skin_ini.mania_sections.iter().any(|sec| sec.key_image_ds.iter().any(|r| &r.name == stem));
            let has_key = skin_ini.mania_sections.iter().any(|sec| sec.key_images.iter().any(|r| &r.name == stem));
            (include_d && has_d) || (include_key && has_key)
        });
    }

    add_log(&mut log, &format!("共 {} 组不同的 Key 图片", groups.len()));

    // 收集需要 ini 修改的项
    let mut ini_patches: Vec<(u32, u32, bool, String)> = Vec::new(); // (keys, col, is_d, new_stem)

    for (stem, refs) in &groups {
        let image_path = match shared::skin_ini::find_image_file(&dir, stem) {
            Some(p) => p,
            None => {
                add_log(&mut log, &format!("⚠ 找不到 Key 图片: {}", stem));
                continue;
            }
        };

        let img = image::open(&image_path)
            .map_err(|e| format!("读取图片失败 {}: {}", stem, e))?
            .to_rgba8();

        let mut unique_cw: Vec<u32> = refs.iter().map(|(_, cw)| *cw).collect();
        unique_cw.sort();
        unique_cw.dedup();

        let is_2x = shared::image_utils::is_2x(&image_path);

        add_log(&mut log, &format!("处理: {} ({} 个键数/列宽组合)", stem, refs.len()));

        if unique_cw.len() == 1 {
            let cw = unique_cw[0];
            let repaired = shared::keyd_repair::repair_key_image(&img, cw, is_2x);
            backup_file(&dir, &image_path, backup, &ts_dir)?;
            repaired.save(&image_path)
                .map_err(|e| format!("保存失败 {}: {}", stem, e))?;
            add_log(&mut log, &format!("  ✓ {}: cw={} → {}×{}{}", stem, cw, repaired.width(), repaired.height(), is2x_tag(&image_path)));
        } else {
            let ext = image_path.extension().and_then(|e| e.to_str()).unwrap_or("png");
            let first_cw = unique_cw[0];
            let repaired = shared::keyd_repair::repair_key_image(&img, first_cw, is_2x);
            backup_file(&dir, &image_path, backup, &ts_dir)?;
            repaired.save(&image_path)
                .map_err(|e| format!("保存失败 {}: {}", stem, e))?;
            add_log(&mut log, &format!("  ✓ {}: cw={} → {}×{}{}", stem, first_cw, repaired.width(), repaired.height(), is2x_tag(&image_path)));

            for &cw in &unique_cw[1..] {
                let repaired_cw = shared::keyd_repair::repair_key_image(&img, cw, is_2x);
                let copy_stem = format!("{}_cw{}", stem, cw);
                let copy_path = dir.join(format!("{}.{}", copy_stem, ext));
                repaired_cw.save(&copy_path)
                    .map_err(|e| format!("保存副本失败 {}: {}", copy_stem, e))?;

                // 收集 ini 更新
                for (keys_k, _) in refs.iter().filter(|(_, c)| *c == cw) {
                    for sec in &skin_ini.mania_sections {
                        if sec.keys == *keys_k && sec.column_width == cw {
                            for r in &sec.key_image_ds {
                                if r.name == *stem {
                                    ini_patches.push((*keys_k, r.column, true, copy_stem.clone()));
                                }
                            }
                            for r in &sec.key_images {
                                if r.name == *stem {
                                    ini_patches.push((*keys_k, r.column, false, copy_stem.clone()));
                                }
                            }
                        }
                    }
                }
                add_log(&mut log, &format!("  ✓ {}: cw={} → {}×{} (副本: {}){}", stem, cw, repaired_cw.width(), repaired_cw.height(), copy_stem, is2x_tag(&copy_path)));
            }
        }
    }

    // 统一应用 ini 修改
    for (keys, col, is_d, new_stem) in &ini_patches {
        match _update_key_image_in_ini(&ini_path, *keys, *col, *is_d, new_stem) {
            Ok(()) => {
                let label = if *is_d { "KeyImage#D" } else { "KeyImage#" };
                add_log(&mut log, &format!("  已更新 {}{} → {}", label, col, new_stem));
            }
            Err(e) => add_log(&mut log, &format!("⚠ {}", e)),
        }
    }

    add_log(&mut log, "Key 图片修复完成！");
    Ok(log)
}

// ---- Throw length ----

/// 获取皮肤的投长度信息（按文件夹读取 skin.ini）
///
/// 返回每个有 NoteImage#L 的键数的：键数、图片茎、当前投长度、图片高度、是否合规（高>5000）。
#[derive(Debug, serde::Serialize)]
pub struct SkinThrowInfo {
    pub keys: u32,
    pub stem: String,
    pub current_throw: u32,
    pub height: u32,
    pub valid: bool,
    pub is_2x: bool,
}

#[tauri::command]
pub fn get_skin_throw_info(folder_path: String) -> Result<Vec<SkinThrowInfo>, String> {
    let dir = PathBuf::from(&folder_path);
    if !dir.is_dir() {
        return Err("指定的路径不是有效的文件夹".to_string());
    }

    let ini_path = dir.join("skin.ini");
    if !ini_path.exists() {
        return Err("未找到 skin.ini 文件".to_string());
    }

    let skin_ini = shared::skin_ini::parse_skin_ini(&ini_path)?;
    let mut results: Vec<SkinThrowInfo> = Vec::new();

    for section in &skin_ini.mania_sections {
        if section.note_image_ls.is_empty() {
            continue;
        }

        // 收集该小节所有唯一的图片茎
        let mut stems: Vec<String> = section
            .note_image_ls
            .iter()
            .map(|r| r.name.clone())
            .collect();
        stems.sort();
        stems.dedup();

        for stem in &stems {
            let (current_throw, height, valid, is_2x) = match shared::skin_ini::find_image_file(&dir, stem) {
                Some(p) => {
                    let is_2x = shared::image_utils::is_2x(&p);
                    match image::open(&p) {
                        Ok(img) => {
                            let rgba = img.to_rgba8();
                            let (ok, h) = shared::throw_length::validate_tail_image(&rgba);
                            (shared::throw_length::find_throw_length(&rgba), h, ok, is_2x)
                        }
                        Err(_) => (0, 0, false, is_2x),
                    }
                }
                None => (0, 0, false, false),
            };

            results.push(SkinThrowInfo {
                keys: section.keys,
                stem: stem.clone(),
                current_throw,
                height,
                valid,
                is_2x,
            });
        }
    }

    Ok(results)
}

/// 修改投长度（皮肤文件夹模式）
///
/// `keys` 与 `throws` 一一对应，每个键数可设不同的目标投长度。
#[tauri::command]
pub fn modify_skin_throw_length(
    folder_path: String,
    keys: Vec<u32>,
    throws: Vec<u32>,
    backup: bool,
) -> Result<Vec<String>, String> {
    if keys.len() != throws.len() || keys.is_empty() {
        return Err("keys 与 throws 长度不匹配或为空".to_string());
    }

    let dir = PathBuf::from(&folder_path);
    if !dir.is_dir() {
        return Err("指定的路径不是有效的文件夹".to_string());
    }

    let ini_path = dir.join("skin.ini");
    if !ini_path.exists() {
        return Err("未找到 skin.ini 文件".to_string());
    }

    // 构建 keys → target_throw 映射
    let throw_map: std::collections::HashMap<u32, u32> = keys
        .iter()
        .zip(throws.iter())
        .map(|(&k, &t)| (k, t))
        .collect();

    let skin_ini = shared::skin_ini::parse_skin_ini(&ini_path)?;
    let mut log: Vec<String> = Vec::new();
    let add_log = |log: &mut Vec<String>, msg: &str| log.push(msg.to_string());

    let mut seen = std::collections::HashSet::new();
    let mut processed = false;
    let ts_dir = backup_timestamp();

    for section in &skin_ini.mania_sections {
        let target_throw = match throw_map.get(&section.keys) {
            Some(&t) => t,
            None => continue,
        };

        for r in &section.note_image_ls {
            if !seen.insert(r.name.clone()) {
                continue;
            }

            let image_path = match shared::skin_ini::find_image_file(&dir, &r.name) {
                Some(p) => p,
                None => {
                    add_log(&mut log, &format!("⚠ 找不到面尾图片: {} ({}k)", r.name, section.keys));
                    continue;
                }
            };

            let img = image::open(&image_path)
                .map_err(|e| format!("读取图片失败 {}: {}", r.name, e))?
                .to_rgba8();

            let current_throw = shared::throw_length::find_throw_length(&img);
            add_log(
                &mut log,
                &format!("{}k: 投长度 {}px → {}px", section.keys, current_throw, target_throw),
            );

            let modified = shared::throw_length::modify_throw_length(&img, target_throw);
            backup_file(&dir, &image_path, backup, &ts_dir)?;
            modified
                .save(&image_path)
                .map_err(|e| format!("保存失败 {}: {}", r.name, e))?;
            add_log(&mut log, &format!("  ✓ {} 已保存", image_path.display()));
            processed = true;
        }
    }

    if !processed {
        add_log(&mut log, "未找到匹配的键数小节");
    } else {
        add_log(&mut log, "修改完成！");
    }

    Ok(log)
}

// ---- Validator ----

/// 皮肤文件校验
///
/// 检查 NoteImage#L、KeyImage#D、KeyImage# 引用的图片是否存在。
#[tauri::command]
pub fn validate_skin_files_cmd(folder_path: String) -> Result<Vec<String>, String> {
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
}

// ---- Helpers ----

use std::time::{SystemTime, UNIX_EPOCH};

/// 生成备份时间戳目录名（UTC+8，一次操作只调一次）
fn backup_timestamp() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + 28800; // UTC+8
    let secs_of_day = ts % 86400;
    let days_since_epoch = ts / 86400;
    let (y, m, d) = epoch_to_date(days_since_epoch);
    let h = secs_of_day / 3600;
    let min = (secs_of_day % 3600) / 60;
    let s = secs_of_day % 60;
    format!("{:04}-{:02}-{:02}_{:02}-{:02}-{:02}", y, m, d, h, min, s)
}

fn backup_file(skin_dir: &PathBuf, file_path: &PathBuf, do_backup: bool, ts_dir: &str) -> Result<(), String> {
    if !do_backup {
        return Ok(());
    }

    let backup_root = skin_dir.join("_backup").join(ts_dir);
    fs::create_dir_all(&backup_root).map_err(|e| format!("创建备份目录失败: {}", e))?;

    let relative = file_path
        .strip_prefix(skin_dir)
        .unwrap_or_else(|_| Path::new(file_path.file_name().unwrap_or_default()));
    let flat_name = relative
        .to_string_lossy()
        .replace(['/', '\\'], "-");

    let backup_path = backup_root.join(&flat_name);
    if backup_path.exists() {
        return Ok(());
    }
    fs::copy(file_path, &backup_path).map_err(|e| format!("备份失败: {}", e))?;
    Ok(())
}

/// 粗糙的 Unix epoch days → (year, month, day)
fn epoch_to_date(days: u64) -> (u64, u64, u64) {
    // 以 1970-01-01 为起点计算
    let mut y = 1970u64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let month_days = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1u64;
    for md in month_days.iter() {
        if remaining < *md {
            break;
        }
        remaining -= *md;
        m += 1;
    }
    let d = remaining as u64 + 1;
    (y, m, d)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}
