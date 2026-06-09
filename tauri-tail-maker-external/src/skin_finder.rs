use std::path::{Path, PathBuf};

/// 查找结果
#[derive(Debug, serde::Serialize)]
pub struct SkinRootResult {
    pub success: bool,
    pub message: String,
    pub path: Option<String>,
}

/// 查找皮肤根目录（skin.ini 所在目录）
///
/// 查找策略：
/// 1. 从 exe 所在目录的上一级开始找
/// 2. 找不到再从 exe 同级找
/// 3. 找不到再从 exe 同级的子文件夹找（最多三层）
/// 找到一个就立即返回
#[tauri::command]
pub fn find_skin_root() -> SkinRootResult {
    // 获取 exe 所在目录
    let exe_dir = match std::env::current_exe() {
        Ok(exe) => match exe.parent() {
            Some(dir) => dir.to_path_buf(),
            None => {
                return SkinRootResult {
                    success: false,
                    message: "无法获取程序路径".to_string(),
                    path: None,
                };
            }
        },
        Err(e) => {
            return SkinRootResult {
                success: false,
                message: format!("无法获取程序路径: {}", e),
                path: None,
            };
        }
    };

    // 策略1: 从上一级目录找
    if let Some(parent) = exe_dir.parent() {
        if let Some(found) = search_directory(parent, "skin.ini") {
            return SkinRootResult {
                success: true,
                message: "找到皮肤根目录".to_string(),
                path: Some(found.to_string_lossy().to_string()),
            };
        }
    }

    // 策略2: 从 exe 同级目录找
    if let Some(found) = search_directory(&exe_dir, "skin.ini") {
        return SkinRootResult {
            success: true,
            message: "找到皮肤根目录".to_string(),
            path: Some(found.to_string_lossy().to_string()),
        };
    }

    // 策略3: 从 exe 同级的子文件夹找（最多三层）
    if let Some(found) = search_subdirectories(&exe_dir, "skin.ini", 3) {
        return SkinRootResult {
            success: true,
            message: "找到皮肤根目录".to_string(),
            path: Some(found.to_string_lossy().to_string()),
        };
    }

    SkinRootResult {
        success: false,
        message: "未找到 skin.ini 文件".to_string(),
        path: None,
    }
}

/// 在指定目录中查找目标文件（不递归）
fn search_directory(dir: &Path, filename: &str) -> Option<PathBuf> {
    let target = dir.join(filename);
    if target.exists() {
        Some(dir.to_path_buf())
    } else {
        None
    }
}

/// 在子目录中递归查找目标文件（限制深度）
fn search_subdirectories(dir: &Path, filename: &str, max_depth: u32) -> Option<PathBuf> {
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
