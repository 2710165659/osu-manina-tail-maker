use serde::Serialize;
use std::path::PathBuf;

/// Mania 配置信息
#[derive(Debug, Serialize, Clone)]
pub struct ManiaConfig {
    /// 键数
    pub keys: u32,
    /// 在文件中的起始行号（从0开始）
    pub line_start: usize,
}

/// 查找结果
#[derive(Debug, Serialize)]
pub struct KeyFinderResult {
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
    /// 找到的键数列表（已排序去重）
    pub keys: Vec<u32>,
    /// 详细的 Mania 配置
    pub configs: Vec<ManiaConfig>,
}

/// 从 skin.ini 中查找所有 [Mania] 小节的键数
///
/// # 参数
/// - `skin_root`: 皮肤根目录路径
///
/// # 返回
/// 查找结果，包含所有找到的键数
#[tauri::command]
pub fn find_keys(skin_root: String) -> KeyFinderResult {
    let skin_path = PathBuf::from(&skin_root);
    let skin_ini_path = skin_path.join("skin.ini");

    // 检查 skin.ini 是否存在
    if !skin_ini_path.exists() {
        return KeyFinderResult {
            success: false,
            message: "skin.ini 不存在".to_string(),
            keys: vec![],
            configs: vec![],
        };
    }

    // 读取文件内容
    let content = match std::fs::read_to_string(&skin_ini_path) {
        Ok(c) => c,
        Err(e) => {
            return KeyFinderResult {
                success: false,
                message: format!("读取 skin.ini 失败: {}", e),
                keys: vec![],
                configs: vec![],
            };
        }
    };

    // 解析 [Mania] 小节
    let configs = parse_mania_configs(&content);

    // 提取去重排序后的键数列表
    let mut keys: Vec<u32> = configs.iter().map(|c| c.keys).collect();
    keys.sort();
    keys.dedup();

    KeyFinderResult {
        success: true,
        message: format!("找到 {} 个 [Mania] 配置", configs.len()),
        keys,
        configs,
    }
}

/// 解析 skin.ini 内容，提取所有 [Mania] 小节的键数
fn parse_mania_configs(content: &str) -> Vec<ManiaConfig> {
    let mut configs = vec![];
    let mut in_mania = false;
    let mut current_keys: Option<u32> = None;
    let mut section_start = 0;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // 检测小节开始
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            // 保存上一个 Mania 小节的配置
            if in_mania {
                if let Some(keys) = current_keys {
                    configs.push(ManiaConfig {
                        keys,
                        line_start: section_start,
                    });
                }
            }

            // 判断是否进入 [Mania] 小节
            in_mania = trimmed.eq_ignore_ascii_case("[Mania]");
            current_keys = None;

            if in_mania {
                section_start = line_num;
            }
            continue;
        }

        // 在 [Mania] 小节中查找 Keys
        if in_mania {
            // 支持 Keys: 和 keys: 两种格式
            if let Some(keys_str) = trimmed
                .strip_prefix("Keys:")
                .or_else(|| trimmed.strip_prefix("keys:"))
            {
                if let Ok(keys) = keys_str.trim().parse::<u32>() {
                    current_keys = Some(keys);
                }
            }
        }
    }

    // 处理最后一个 [Mania] 小节
    if in_mania {
        if let Some(keys) = current_keys {
            configs.push(ManiaConfig {
                keys,
                line_start: section_start,
            });
        }
    }

    configs
}
