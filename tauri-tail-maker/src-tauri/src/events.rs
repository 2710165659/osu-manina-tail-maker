use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// 推送到前端 `app:event` 的统一负载
#[derive(Debug, Clone, Serialize)]
pub struct AppEventPayload {
    pub level: String,
    pub target: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// 纯文本日志事件
pub fn emit_log(app: &AppHandle, level: &str, target: &str, message: &str) {
    let _ = app.emit("app:event", AppEventPayload {
        level: level.into(),
        target: target.into(),
        message: message.into(),
        data: None,
    });
}

/// 带结构化数据的日志事件（投长度结果、批量进度等）
pub fn emit_data(
    app: &AppHandle,
    level: &str,
    target: &str,
    message: &str,
    data: serde_json::Value,
) {
    let _ = app.emit("app:event", AppEventPayload {
        level: level.into(),
        target: target.into(),
        message: message.into(),
        data: Some(data),
    });
}
