/// 查找皮肤根目录（skin.ini 所在目录）— thin wrapper
use shared::skin_finder;

#[derive(Debug, serde::Serialize)]
pub struct SkinRootResult {
    pub success: bool,
    pub message: String,
    pub path: Option<String>,
}

#[tauri::command]
pub async fn find_skin_root() -> SkinRootResult {
    tauri::async_runtime::spawn_blocking(|| {
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

        match skin_finder::find_skin_root_from(&exe_dir) {
            Some(found) => SkinRootResult {
                success: true,
                message: "找到皮肤根目录".to_string(),
                path: Some(found.to_string_lossy().to_string()),
            },
            None => SkinRootResult {
                success: false,
                message: "未找到 skin.ini 文件".to_string(),
                path: None,
            },
        }
    }).await.unwrap_or_else(|e| SkinRootResult {
        success: false,
        message: format!("任务执行失败: {}", e),
        path: None,
    })
}
