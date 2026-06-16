export type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error';

export interface LogEntry {
  level: LogLevel
  message: string
  target: string
  timestamp: string
}

export interface LogConfig {
  /** 最低日志级别，默认 "info" */
  minLevel?: LogLevel
}
