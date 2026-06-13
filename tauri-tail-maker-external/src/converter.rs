/// 面尾转换命令 — thin wrapper around shared::tail_toolbox
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// 转换配置
#[derive(Debug, Deserialize)]
pub struct ConvertConfig {
    /// 皮肤根目录
    pub skin_root: String,
    /// 模式: "lazer" 或 "stable"
    pub mode: String,
    /// 键数 → 目标投长度
    pub throws: Vec<(u32, u32)>,
    /// (stem, preset_name) 的列表
    pub presets: Vec<(String, String)>,
    /// 需要修复的 Key/KeyD stem 列表
    pub keyd_stems: Vec<String>,
}

/// 转换结果
#[derive(Debug, Serialize)]
pub struct ConvertResult {
    pub success: bool,
    pub message: String,
    pub logs: Vec<String>,
}

/// 转换命令
#[tauri::command]
pub fn convert_tail(config: ConvertConfig) -> ConvertResult {
    let skin_dir = PathBuf::from(&config.skin_root);
    if !skin_dir.is_dir() {
        return ConvertResult {
            success: false,
            message: "皮肤目录不存在".to_string(),
            logs: vec![],
        };
    }

    let backup_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| skin_dir.clone())
        .join("_backup");

    match shared::tail_toolbox::execute_toolbox(
        &skin_dir,
        &config.mode,
        &config.throws,
        &config.presets,
        &config.keyd_stems,
        &backup_dir,
    ) {
        Ok(log) => ConvertResult {
            success: true,
            message: "转换完成".to_string(),
            logs: log,
        },
        Err(e) => ConvertResult {
            success: false,
            message: e,
            logs: vec![],
        },
    }
}
