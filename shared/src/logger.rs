use std::sync::{Arc, OnceLock, RwLock};
use tokio::sync::mpsc;

/// 日志级别（按严重程度递增）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// 单条日志记录
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    /// 日志来源模块（如 "frontend", "renderer", "toolbox"）
    pub target: String,
    /// ISO 8601 格式时间戳
    pub timestamp: String,
    /// 可选结构化数据（投长度、批量进度等）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl LogEntry {
    pub fn new(level: LogLevel, target: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            target: target.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: None,
        }
    }

    pub fn new_with_data(
        level: LogLevel,
        target: impl Into<String>,
        message: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            level,
            message: message.into(),
            target: target.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: Some(data),
        }
    }
}

/// 日志观察者 trait —— 实现观察者模式中的 Observer
///
/// 订阅到 [`LogDispatcher`] 后，每条日志都会被推送至此。
pub trait LogObserver: Send + Sync {
    fn on_log(&self, entry: &LogEntry);
}

/// 日志分发器 —— 观察者模式中的 Subject，同时也是消息队列的入口
///
/// - 通过 `log()` 写入日志（非阻塞，推入 channel）
/// - 通过 `subscribe()` 注册观察者
/// - 后台异步 task 调用 `notify_observers()` 消费 channel 并广播
pub struct LogDispatcher {
    observers: RwLock<Vec<Arc<dyn LogObserver>>>,
    tx: mpsc::UnboundedSender<LogEntry>,
    min_level: RwLock<LogLevel>,
}

impl std::fmt::Debug for LogDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogDispatcher").finish_non_exhaustive()
    }
}

impl LogDispatcher {
    /// 创建分发器，返回 (Arc<Self>, Receiver)
    ///
    /// Receiver 需要被传递给异步 run loop 消费。
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<LogEntry>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let dispatcher = Arc::new(Self {
            observers: RwLock::new(Vec::new()),
            tx,
            min_level: RwLock::new(LogLevel::Info),
        });
        (dispatcher, rx)
    }

    /// 注册观察者
    pub fn subscribe(&self, observer: Arc<dyn LogObserver>) {
        self.observers.write().unwrap().push(observer);
    }

    /// 设置最低日志级别（低于此级别的日志将被丢弃）
    pub fn set_min_level(&self, level: LogLevel) {
        *self.min_level.write().unwrap() = level;
    }

    /// 写入日志 —— 非阻塞，推入内部 channel
    pub fn log(&self, entry: LogEntry) {
        if entry.level >= *self.min_level.read().unwrap() {
            // unbounded send 不会阻塞，忽略 receiver 已关闭的情况
            let _ = self.tx.send(entry);
        }
    }

    /// 通知所有注册的观察者
    pub fn notify_observers(&self, entry: &LogEntry) {
        for observer in self.observers.read().unwrap().iter() {
            observer.on_log(entry);
        }
    }
}

// ── 全局单例 ───────────────────────────────────────────────

static GLOBAL_DISPATCHER: OnceLock<Arc<LogDispatcher>> = OnceLock::new();

/// 初始化全局分发器（应该在 app 启动时调用一次）
///
/// 返回 receiver 用于启动异步 run loop。
pub fn init_global_dispatcher() -> (Arc<LogDispatcher>, mpsc::UnboundedReceiver<LogEntry>) {
    let (dispatcher, rx) = LogDispatcher::new();
    GLOBAL_DISPATCHER
        .set(dispatcher.clone())
        .expect("Logger has already been initialized");
    (dispatcher, rx)
}

/// 获取全局分发器引用
pub fn global_dispatcher() -> Option<&'static Arc<LogDispatcher>> {
    GLOBAL_DISPATCHER.get()
}

// ── 异步消费循环 ────────────────────────────────────────────

/// 异步消费日志 channel，通知所有观察者
///
/// 应由 `tauri::async_runtime::spawn` 在后台运行。
pub async fn run_dispatcher(
    dispatcher: Arc<LogDispatcher>,
    mut rx: mpsc::UnboundedReceiver<LogEntry>,
) {
    while let Some(entry) = rx.recv().await {
        dispatcher.notify_observers(&entry);
    }
}

// ── 便捷函数 ────────────────────────────────────────────────

/// 记录一条 INFO 日志
pub fn log_info(target: &str, message: &str) {
    if let Some(d) = global_dispatcher() {
        d.log(LogEntry::new(LogLevel::Info, target, message));
    }
}

/// 记录一条 WARN 日志
pub fn log_warn(target: &str, message: &str) {
    if let Some(d) = global_dispatcher() {
        d.log(LogEntry::new(LogLevel::Warn, target, message));
    }
}

/// 记录一条 ERROR 日志
pub fn log_error(target: &str, message: &str) {
    if let Some(d) = global_dispatcher() {
        d.log(LogEntry::new(LogLevel::Error, target, message));
    }
}

/// 记录一条 DEBUG 日志
pub fn log_debug(target: &str, message: &str) {
    if let Some(d) = global_dispatcher() {
        d.log(LogEntry::new(LogLevel::Debug, target, message));
    }
}

/// 记录一条 TRACE 日志
pub fn log_trace(target: &str, message: &str) {
    if let Some(d) = global_dispatcher() {
        d.log(LogEntry::new(LogLevel::Trace, target, message));
    }
}

/// 记录一条带结构化数据的日志（如投长度结果、批量进度等）
pub fn log_data(level: LogLevel, target: &str, message: &str, data: serde_json::Value) {
    if let Some(d) = global_dispatcher() {
        d.log(LogEntry::new_with_data(level, target, message, data));
    }
}
