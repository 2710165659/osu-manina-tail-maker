use shared::logger::{self, LogEntry, LogLevel, LogObserver};
use std::sync::Arc;
use tauri::AppHandle;

use crate::events;

// ── 前端观察者 ──────────────────────────────────────────────

/// 把日志通过统一的 `app:event` 推送到前端
struct FrontendLogObserver {
    app: AppHandle,
}

impl LogObserver for FrontendLogObserver {
    fn on_log(&self, entry: &LogEntry) {
        let level = format!("{:?}", entry.level).to_lowercase();
        if let Some(ref data) = entry.data {
            events::emit_data(&self.app, &level, &entry.target, &entry.message, data.clone());
        } else {
            events::emit_log(&self.app, &level, &entry.target, &entry.message);
        }
    }
}

// ── Tauri 命令 ──────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct LogConfig {
    /// 最低日志级别（"trace" | "debug" | "info" | "warn" | "error"），默认 info
    pub min_level: Option<String>,
}

/// 初始化日志系统 —— 前端启动时调用一次
///
/// 配置日志级别 + 注册前端观察者，后续所有后端日志会通过 `log:entry` 事件推送到前端。
#[tauri::command]
pub fn init_logger(app: AppHandle, config: LogConfig) -> Result<(), String> {
    let dispatcher = logger::global_dispatcher().ok_or("Logger not initialized")?;

    // 设置最低级别
    if let Some(ref level_str) = config.min_level {
        let level = parse_level(level_str)?;
        dispatcher.set_min_level(level);
    }

    // 订阅前端观察者
    let observer = Arc::new(FrontendLogObserver { app });
    dispatcher.subscribe(observer);

    Ok(())
}

/// 前端批量推送日志到后端队列
///
/// 前端同步写入的日志通过此命令进入后端异步分发通道。
#[tauri::command]
pub fn emit_logs(entries: Vec<LogEntry>) {
    if let Some(d) = logger::global_dispatcher() {
        for entry in entries {
            d.log(entry);
        }
    }
}

// ── 辅助函数 ────────────────────────────────────────────────

fn parse_level(s: &str) -> Result<LogLevel, String> {
    match s.to_lowercase().as_str() {
        "trace" => Ok(LogLevel::Trace),
        "debug" => Ok(LogLevel::Debug),
        "info" => Ok(LogLevel::Info),
        "warn" => Ok(LogLevel::Warn),
        "error" => Ok(LogLevel::Error),
        other => Err(format!("Invalid log level: {}", other)),
    }
}
