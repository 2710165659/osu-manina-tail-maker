import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { LogEntry, LogConfig, LogLevel } from '../types/logger'

interface AppEventPayload {
  level: string
  target: string
  message: string
  data?: unknown
}

class Logger {
  private initialized = false
  private buffer: LogEntry[] = []
  private flushTimer: ReturnType<typeof setTimeout> | null = null
  private listeners: Array<(entry: LogEntry) => void> = []

  async init(config: LogConfig = {}) {
    if (this.initialized) return

    await invoke('init_logger', { config })

    // 监听统一后端事件
    await listen<AppEventPayload>('app:event', (event) => {
      const { level, target, message } = event.payload
      const entry: LogEntry = {
        level: level as LogLevel,
        message,
        target,
        timestamp: new Date().toISOString(),
      }
      // 回显到浏览器 console
      const fn = (console as any)[level] ?? console.log
      fn(`[${target}] ${message}`)
      // 通知外部监听者
      this.listeners.forEach((cb) => cb(entry))
    })

    this.initialized = true
    this.flushBuffer()
  }

  private flushBuffer() {
    if (this.buffer.length === 0) return
    const entries = [...this.buffer]
    this.buffer = []
    invoke('emit_logs', { entries }).catch(console.error)
  }

  private scheduleFlush() {
    if (this.flushTimer) clearTimeout(this.flushTimer)
    this.flushTimer = setTimeout(() => this.flushBuffer(), 50)
  }

  log(level: LogLevel, message: string, target = 'frontend') {
    const entry: LogEntry = {
      level,
      message,
      target,
      timestamp: new Date().toISOString(),
    }
    this.buffer.push(entry)
    this.scheduleFlush()
  }

  info(message: string, target?: string) { this.log('info', message, target) }
  warn(message: string, target?: string) { this.log('warn', message, target) }
  error(message: string, target?: string) { this.log('error', message, target) }
  debug(message: string, target?: string) { this.log('debug', message, target) }
  trace(message: string, target?: string) { this.log('trace', message, target) }

  onLog(fn: (entry: LogEntry) => void): () => void {
    this.listeners.push(fn)
    return () => {
      const idx = this.listeners.indexOf(fn)
      if (idx !== -1) this.listeners.splice(idx, 1)
    }
  }

  flush() {
    if (this.flushTimer) {
      clearTimeout(this.flushTimer)
      this.flushTimer = null
    }
    this.flushBuffer()
  }
}

export const logger = new Logger()
