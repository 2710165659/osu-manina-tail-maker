/// 皮肤根目录检测
///
/// 在指定基础目录附近搜索 skin.ini 文件，确定皮肤根目录位置。
use std::path::{Path, PathBuf};

/// 在指定基础目录附近查找 skin.ini 所在目录。
///
/// 查找策略：
/// 1. 从 base_dir 的上一级开始找
/// 2. 找不到再从 base_dir 同级找
/// 3. 找不到再从 base_dir 同级的子文件夹找（最多三层）
/// 找到一个就立即返回。
pub fn find_skin_root_from(base_dir: &Path) -> Option<PathBuf> {
    // 策略1: 从上一级目录找
    if let Some(parent) = base_dir.parent() {
        if let Some(found) = search_directory(parent, "skin.ini") {
            return Some(found);
        }
    }

    // 策略2: 从同级目录找
    if let Some(found) = search_directory(base_dir, "skin.ini") {
        return Some(found);
    }

    // 策略3: 从同级的子文件夹找（最多三层）
    if let Some(found) = search_subdirectories(base_dir, "skin.ini", 3) {
        return Some(found);
    }

    None
}

/// 在指定目录中查找目标文件（不递归），返回文件所在目录。
pub fn search_directory(dir: &Path, filename: &str) -> Option<PathBuf> {
    let target = dir.join(filename);
    if target.exists() {
        Some(dir.to_path_buf())
    } else {
        None
    }
}

/// 在子目录中递归查找目标文件（限制深度），返回文件所在目录。
pub fn search_subdirectories(dir: &Path, filename: &str, max_depth: u32) -> Option<PathBuf> {
    if max_depth == 0 {
        return None;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return None,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        // 检查当前子目录
        let target = path.join(filename);
        if target.exists() {
            return Some(path);
        }

        // 递归查找更深的子目录
        if let Some(found) = search_subdirectories(&path, filename, max_depth - 1) {
            return Some(found);
        }
    }

    None
}
