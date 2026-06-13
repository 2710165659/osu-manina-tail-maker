/// osk 文件操作
///
/// osk 文件（zip 格式）的解压、修复、重新打包。
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::lazer_repair;

/// 递归将目录下文件添加到 zip（跳过 _backup）。
pub fn add_files_to_zip(
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
            let buf = fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))?;
            writer
                .write_all(&buf)
                .map_err(|e| format!("写入 zip 条目失败: {}", e))?;
        }
    }
    Ok(())
}

/// osk 文件解压、修复后再打包。
///
/// `repair_modes`: 包含 "tail" 和/或 "keyd" 的集合。
/// `backup_dir`: 备份根目录（将在其下创建时间戳子目录）。
pub fn repair_osk(
    osk_path: &Path,
    repair_modes: &HashSet<&str>,
    backup_dir: &Path,
    log: &mut Vec<String>,
) -> Result<(), String> {
    let parent = osk_path.parent().unwrap_or_else(|| Path::new("."));
    let work_dir = parent.join(format!(
        "_osk_repair_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));

    // 1. 解压 osk → work_dir
    let osk_file = fs::File::open(osk_path).map_err(|e| format!("打开 osk 文件失败: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(osk_file).map_err(|e| format!("读取 zip 失败: {}", e))?;

    fs::create_dir_all(&work_dir).map_err(|e| format!("创建临时目录失败: {}", e))?;

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
    log.push("已解压到临时目录".to_string());

    // 检测 osk 解压后是否有一层嵌套目录
    let extract_root = work_dir.clone();
    let mut work_dir = extract_root.clone();
    let ini_path = work_dir.join("skin.ini");
    if !ini_path.exists() {
        let subdirs: Vec<PathBuf> = fs::read_dir(&work_dir)
            .map_err(|e| format!("读取临时目录失败: {}", e))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().is_dir() && !e.file_name().to_string_lossy().starts_with('_')
            })
            .map(|e| e.path())
            .collect();
        if subdirs.len() == 1 && subdirs[0].join("skin.ini").exists() {
            work_dir = subdirs[0].clone();
            log.push(format!(
                "检测到嵌套目录: {}",
                work_dir.file_name().unwrap_or_default().to_string_lossy()
            ));
        }
    }

    // 在 work_dir 上修复
    let ini_path = work_dir.join("skin.ini");
    if ini_path.exists() {
        // 面尾修复
        if repair_modes.contains("tail") {
            log.push("--- 面尾修复 ---".to_string());
            match lazer_repair::execute_lazer_tail_repair(&work_dir, backup_dir) {
                Ok(tail_log) => log.extend(tail_log),
                Err(e) => log.push(format!("面尾修复失败: {}", e)),
            }
        }

        // Key 修复
        if repair_modes.contains("keyd") {
            log.push("--- Key + KeyD 修复 ---".to_string());
            match lazer_repair::execute_lazer_key_repair(&work_dir, backup_dir, "all") {
                Ok(key_log) => log.extend(key_log),
                Err(e) => log.push(format!("Key 修复失败: {}", e)),
            }
        }
    }

    // 3. 重新打包 → 覆盖原 osk
    let temp_osk = osk_path.with_extension("osk.tmp");
    let temp_file =
        fs::File::create(&temp_osk).map_err(|e| format!("创建临时 osk 失败: {}", e))?;
    let mut writer = zip::ZipWriter::new(temp_file);

    add_files_to_zip(&work_dir, &work_dir, &mut writer)?;
    writer
        .finish()
        .map_err(|e| format!("完成 zip 写入失败: {}", e))?;

    // 清理临时目录
    let _ = fs::remove_dir_all(&extract_root);
    // 替换
    fs::rename(&temp_osk, osk_path).map_err(|e| format!("替换原 osk 文件失败: {}", e))?;
    log.push("已重新打包 osk 文件".to_string());

    Ok(())
}
