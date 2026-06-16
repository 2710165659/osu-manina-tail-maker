import { ref, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'

export interface ToolLogEntry {
  time: string
  message: string
  type: 'info' | 'success' | 'warning' | 'error' | 'debug' | 'done'
  target: string
}

/** 后端推送的统一事件格式 */
interface AppEventPayload {
  level: string
  target: string
  message: string
  data?: unknown
}

export interface UseToolLoggerOptions {
  /** 按 target 过滤（如 'toolbox', 'repair', 'throw', 'validator'），不传则接收所有 */
  target?: string | string[]
  /** 当收到 error 级别日志时触发的回调 */
  onError?: (entry: ToolLogEntry) => void
  /** 当事件携带 data 时触发的回调 */
  onData?: (target: string, data: unknown) => void
}

export function useToolLogger(options: UseToolLoggerOptions = {}) {
  const logs = ref<ToolLogEntry[]>([])

  const targets = options.target
    ? (Array.isArray(options.target) ? options.target : [options.target])
    : null

  function formatTime(): string {
    const d = new Date()
    return `${d.getHours().toString().padStart(2, '0')}:${d.getMinutes().toString().padStart(2, '0')}:${d.getSeconds().toString().padStart(2, '0')}`
  }

  function mapType(level: string): ToolLogEntry['type'] {
    switch (level) {
      case 'error': return 'error'
      case 'warn':
      case 'warning': return 'warning'
      case 'done': return 'done'
      case 'info': return 'info'
      case 'success': return 'success'
      case 'debug':
      case 'trace': return 'debug'
      default: return 'info'
    }
  }

  function onEvent(payload: AppEventPayload) {
    if (targets && !targets.includes(payload.target)) return

    const toolEntry: ToolLogEntry = {
      time: formatTime(),
      message: payload.message,
      type: mapType(payload.level),
      target: payload.target,
    }

    logs.value.push(toolEntry)

    if (toolEntry.type === 'error' && options.onError) {
      options.onError(toolEntry)
    }
    if (payload.data !== undefined && options.onData) {
      options.onData(payload.target, payload.data)
    }
    // done 事件也通过 onData 通知（无 data 时传 { done: true }）
    if (toolEntry.type === 'done' && options.onData) {
      options.onData(payload.target, { done: true })
    }
  }

  /** 清空日志 */
  function clear() {
    logs.value.length = 0
  }

  /** 直接推入一条前端日志（瞬时显示，不经过后端往返） */
  function push(message: string, type: ToolLogEntry['type'] = 'info') {
    const entry: ToolLogEntry = {
      time: formatTime(),
      message,
      type,
      target: 'frontend',
    }
    logs.value.push(entry)
  }

  let unsub: (() => void) | null = null

  onMounted(async () => {
    unsub = await listen<AppEventPayload>('app:event', (event) => {
      onEvent(event.payload)
    })
  })

  onUnmounted(() => {
    if (unsub) unsub()
  })

  return { logs, clear, push }
}
