<template>
  <div class="log-section">
    <div class="log-header">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
        <rect x="1" y="1" width="10" height="10" rx="2" stroke="currentColor" stroke-width="1.1" />
        <path d="M3.5 4h5M3.5 6h3M3.5 8h4" stroke="currentColor" stroke-width="0.9" stroke-linecap="round" />
      </svg>
      <span>日志</span>
    </div>
    <div class="log-content" ref="logContainerRef" :style="{ height: maxHeight }">
      <template v-if="logs.length === 0">
        <div class="log-empty">
          <span class="log-empty-icon">~</span>
          <span>{{ emptyText }}</span>
        </div>
      </template>
      <template v-else>
        <div v-for="(log, i) in logs" :key="i" :class="['log-line', log.type]">
          <span class="log-time">{{ log.time }}</span>
          <span class="log-marker">›</span>
          <span class="log-msg">{{ log.message }}</span>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'

export interface LogEntry {
  time: string
  message: string
  type: 'info' | 'success' | 'warning' | 'error' | 'debug' | 'done'
  target: string
}

const props = withDefaults(defineProps<{
  logs: LogEntry[]
  maxHeight?: string
  emptyText?: string
}>(), {
  maxHeight: '160px',
  emptyText: '等待操作...',
})

const logContainerRef = ref<HTMLDivElement>()

// 自动滚动
watch(() => props.logs.length, () => {
  nextTick(() => {
    if (logContainerRef.value) {
      logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight
    }
  })
})
</script>

<style scoped>
.log-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.log-header {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.8px;
}

.log-header svg {
  opacity: 0.6;
}

.log-content {
  overflow-y: auto;
  padding: 12px;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  line-height: 1.8;
}

.log-content::-webkit-scrollbar {
  width: 4px;
}

.log-content::-webkit-scrollbar-track {
  background: transparent;
}

.log-content::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.08);
  border-radius: 2px;
}

.log-empty {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-muted);
  font-style: italic;
}

.log-empty-icon {
  color: var(--accent-purple);
  opacity: 0.5;
}

.log-line {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.log-time {
  color: var(--text-muted);
  opacity: 0.6;
  flex-shrink: 0;
}

.log-marker {
  color: var(--accent-purple);
  opacity: 0.4;
  flex-shrink: 0;
}

.log-msg {
  flex: 1;
  word-break: break-all;
}

.log-line.info .log-msg    { color: var(--text-secondary); }
.log-line.success .log-msg { color: #44ee88; }
.log-line.warning .log-msg { color: #ffaa44; }
.log-line.error .log-msg   { color: #ff4466; }
.log-line.done .log-msg    { color: #64c8ff; }
.log-line.debug .log-msg   { color: var(--text-muted); }
</style>
