use serde::{Deserialize, Serialize};

/// 转换配置
#[derive(Debug, Deserialize)]
pub struct ConvertConfig {
    /// 皮肤根目录
    pub skin_root: String,
    /// 目标键数列表
    pub keys: Vec<u32>,
    /// 投的长度
    pub throw_length: u32,
    /// 选中的预设名（可选）
    pub preset: Option<String>,
}

/// 转换结果
#[derive(Debug, Serialize)]
pub struct ConvertResult {
    pub success: bool,
    pub message: String,
    pub processed_keys: Vec<u32>,
}

/// 转换功能（待实现）
#[tauri::command]
pub fn convert_tail(config: ConvertConfig) -> ConvertResult {
    // TODO: 等待用户说明具体转换逻辑后再实现
    ConvertResult {
        success: false,
        message: "转换功能尚未实现".to_string(),
        processed_keys: vec![],
    }
}
