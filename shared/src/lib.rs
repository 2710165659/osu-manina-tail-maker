pub mod backup;
pub mod image_utils;
pub mod keyd_repair;
pub mod lazer_repair;
pub mod logger;
pub mod preset_loader;
pub mod skin_finder;
pub mod skin_ini;
pub mod tail_repair;
pub mod tail_toolbox;
pub mod throw_cache;
pub mod throw_info;
pub mod throw_length;

/// 用默认浏览器打开 URL
///
/// 每个 Tauri app 用 `#[tauri::command]` 包装此函数来注册为 IPC 命令。
pub fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn()
            .map_err(|e| format!("打开链接失败: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("打开链接失败: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("打开链接失败: {}", e))?;
    }
    Ok(())
}
