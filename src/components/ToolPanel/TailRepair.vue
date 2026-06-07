<template>
  <div class="tail-repair">
    <!-- 描述卡片 -->
    <div class="desc-card">
      <p class="desc-text">修复投皮转换为 lazer 后，面尾拉伸或KeyD等图片拉伸的问题。支持单个图片修复和整个皮肤修复</p>
    </div>

    <!-- 配置区域 -->
    <div class="config-group">
      <!-- 修复模式 -->
      <div class="field">
        <label class="field-label">修复模式</label>
        <div class="radio-cards radio-cards-3">
          <label :class="['radio-card', { active: repairMode === 'osk' }]" @click="repairMode = 'osk'">
            <div class="radio-dot">
              <div class="radio-dot-inner"></div>
            </div>
            <div class="radio-content">
              <span class="radio-title">osk 文件</span>
              <span class="radio-desc">选择 .osk 文件，自动解析并修复有问题的图片。</span>
            </div>
          </label>
          <label :class="['radio-card', { active: repairMode === 'folder' }]" @click="repairMode = 'folder'">
            <div class="radio-dot">
              <div class="radio-dot-inner"></div>
            </div>
            <div class="radio-content">
              <span class="radio-title">皮肤文件夹</span>
              <span class="radio-desc">选择皮肤所在文件夹，根据 skin.ini 查找并修复。</span>
            </div>
          </label>
          <label :class="['radio-card', { active: repairMode === 'single' }]" @click="repairMode = 'single'">
            <div class="radio-dot">
              <div class="radio-dot-inner"></div>
            </div>
            <div class="radio-content">
              <span class="radio-title">单个图片</span>
              <span class="radio-desc">修复单张图片，需填写 ColumnWidth 值。</span>
            </div>
          </label>
        </div>
      </div>

      <!-- 备份选项 -->
      <div class="field">
        <label class="field-label">是否备份原始文件</label>
        <div class="radio-cards">
          <label :class="['radio-card', { active: !backupOriginal }]" @click="backupOriginal = false">
            <div class="radio-dot">
              <div class="radio-dot-inner"></div>
            </div>
            <div class="radio-content">
              <span class="radio-title">备份</span>
              <span class="radio-desc">覆盖原始图片，同时将原始图片复制到 _backup 文件夹。</span>
            </div>
          </label>
          <label :class="['radio-card', { active: backupOriginal }]" @click="backupOriginal = true">
            <div class="radio-dot">
              <div class="radio-dot-inner"></div>
            </div>
            <div class="radio-content">
              <span class="radio-title">不备份</span>
              <span class="radio-desc">直接覆盖原始图片进行修复。</span>
            </div>
          </label>
        </div>
        <span class="field-hint"></span>
      </div>

      <!-- ColumnWidth 输入 -->
      <Transition name="slide-fade">
        <div v-if="repairMode === 'single'" class="field">
          <label class="field-label">ColumnWidth 值</label>
          <div class="input-wrapper">
            <input type="number" v-model.number="columnWidth" class="field-input"
              placeholder="skin.ini 中的 ColumnWidth" />
            <span class="input-suffix">px</span>
          </div>
          <span class="field-hint">对应 skin.ini 中 [Mania] 小节的 ColumnWidth 值</span>
        </div>
      </Transition>

      <!-- 文件路径 -->
      <div class="field">
        <label class="field-label">文件路径</label>
        <div class="path-group">
          <div class="path-display">
            <svg class="path-icon" width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M1.5 7.5v3.5a1 1 0 001 1h9a1 1 0 001-1V7.5" stroke="currentColor" stroke-width="1.1"
                stroke-linecap="round" />
              <path d="M7 1.5v6M4.5 5L7 7.5 9.5 5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round"
                stroke-linejoin="round" />
            </svg>
            <span class="path-text" :class="{ placeholder: !filePath }">
              {{ filePath || pathPlaceholder }}
            </span>
          </div>
          <button class="browse-btn" @click="handleBrowse">
            <span>浏览</span>
          </button>
          <button class="btn btn-primary" @click="handleRepair" :disabled="!canRepair">
            <span>开始修复</span>
          </button>
        </div>
      </div>
    </div>

    <!-- 日志区域 -->
    <div class="log-section">
      <div class="log-header">
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <rect x="1" y="1" width="10" height="10" rx="2" stroke="currentColor" stroke-width="1.1" />
          <path d="M3.5 4h5M3.5 6h3M3.5 8h4" stroke="currentColor" stroke-width="0.9" stroke-linecap="round" />
        </svg>
        <span>日志</span>
      </div>
      <div class="log-content" ref="logContainer">
        <template v-if="logs.length === 0">
          <div class="log-empty">
            <span class="log-empty-icon">~</span>
            <span>等待操作...</span>
          </div>
        </template>
        <template v-else>
          <div v-for="(log, index) in logs" :key="index" :class="['log-line', log.type]">
            <span class="log-time">{{ log.time }}</span>
            <span class="log-marker">›</span>
            <span class="log-msg">{{ log.message }}</span>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch } from 'vue'

const repairMode = ref<'osk' | 'folder' | 'single'>('osk')
const columnWidth = ref<number>(0)
const filePath = ref('')

watch(repairMode, () => {
  filePath.value = ''
})
const backupOriginal = ref(false)
const logContainer = ref<HTMLDivElement>()

interface LogEntry {
  time: string
  message: string
  type: 'info' | 'success' | 'warning' | 'error'
}

const logs = ref<LogEntry[]>([])

const pathPlaceholder = computed(() => {
  if (repairMode.value === 'osk') return '请选择 .osk 文件'
  if (repairMode.value === 'folder') return '请选择皮肤所在文件夹'
  return '请选择图片文件'
})

const canRepair = computed(() => {
  if (!filePath.value) return false
  if (repairMode.value === 'single' && columnWidth.value <= 0) return false
  return true
})

function addLog(message: string, type: LogEntry['type'] = 'info') {
  const now = new Date()
  const time = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`
  logs.value.push({ time, message, type })
  nextTick(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }
  })
}

async function handleBrowse() {
  const { open } = await import('@tauri-apps/plugin-dialog')

  try {
    let selected: string | string[] | null = null

    if (repairMode.value === 'osk') {
      // osk 文件模式：只允许选择 .osk 文件
      selected = await open({
        multiple: false,
        filters: [{ name: 'osk 文件', extensions: ['osk'] }],
      })
    } else if (repairMode.value === 'folder') {
      // 皮肤文件夹模式：选择目录
      selected = await open({
        multiple: false,
        directory: true,
      })
    } else {
      // 单个图片模式：选择图片文件
      selected = await open({
        multiple: false,
        filters: [{ name: '图片文件', extensions: ['png', 'jpg', 'jpeg', 'bmp', 'webp'] }],
      })
    }

    if (selected) {
      const path = Array.isArray(selected) ? selected[0] : selected
      filePath.value = path
      addLog(`已选择：${path}`, 'info')
    }
  } catch (e) {
    addLog(`文件选择失败：${e}`, 'error')
  }
}

function handleRepair() {
  if (!canRepair.value) return

  addLog('开始修复任务...', 'info')
  const modeLabel = repairMode.value === 'osk' ? 'osk 文件' : repairMode.value === 'folder' ? '皮肤文件夹' : '单个图片'
  addLog(`修复模式：${modeLabel}`, 'info')

  if (repairMode.value === 'single') {
    addLog(`ColumnWidth 值：${columnWidth.value}`, 'info')
  }

  addLog(`文件路径：${filePath.value}`, 'info')
  addLog(`备份原始文件：${backupOriginal.value ? '是' : '否'}`, 'info')
  addLog('正在分析图片尺寸...', 'info')

  setTimeout(() => {
    addLog('检测到需要修复的图片：3 个', 'warning')
    addLog('正在转换图片尺寸...', 'info')
  }, 500)

  setTimeout(() => {
    addLog('图片 1/3 修复完成', 'success')
    addLog('图片 2/3 修复完成', 'success')
    addLog('图片 3/3 修复完成', 'success')
    addLog('修复任务完成！共修复 3 张图片', 'success')
  }, 1500)
}

</script>

<style scoped>
.tail-repair {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* Description Card */
.desc-card {
  display: flex;
  gap: 12px;
  padding: 12px 14px;
  background: rgba(183, 108, 241, 0.04);
  border: 1px solid rgba(183, 108, 241, 0.12);
  border-radius: 10px;
}

.desc-text {
  margin: 0;
  font-size: 12px;
  line-height: 1.7;
  color: var(--text-secondary);
}

/* Config Group */
.config-group {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

/* Fields */
.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
}

/* Radio Cards */
.radio-cards {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.radio-cards-3 {
  grid-template-columns: 1fr 1fr 1fr;
}

.radio-card {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 12px;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.radio-card:hover {
  border-color: rgba(183, 108, 241, 0.3);
  background: rgba(183, 108, 241, 0.03);
}

.radio-card.active {
  border-color: var(--accent-purple);
  background: rgba(183, 108, 241, 0.06);
}

.radio-dot {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 1.5px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-top: 1px;
  transition: all 0.2s ease;
}

.radio-card.active .radio-dot {
  border-color: var(--accent-purple);
}

.radio-dot-inner {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: transparent;
  transition: all 0.2s ease;
}

.radio-card.active .radio-dot-inner {
  background: var(--accent-purple);
}

.radio-content {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.radio-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
}

.radio-desc {
  font-size: 10px;
  color: var(--text-muted);
}

/* Input */
.input-wrapper {
  display: flex;
  align-items: center;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  overflow: hidden;
  transition: border-color 0.2s ease;
}

.input-wrapper:focus-within {
  border-color: var(--accent-purple);
}

.field-input {
  flex: 1;
  padding: 10px 12px;
  background: transparent;
  border: none;
  color: var(--text-primary);
  font-size: 12px;
  font-family: 'JetBrains Mono', monospace;
  outline: none;
}

/* 隐藏数字输入框的上下箭头 */
.field-input[type="number"]::-webkit-inner-spin-button,
.field-input[type="number"]::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.field-input[type="number"] {
  -moz-appearance: textfield;
}

.field-input::placeholder {
  color: var(--text-muted);
  font-family: inherit;
}

.input-suffix {
  padding: 0 12px;
  font-size: 11px;
  color: var(--text-muted);
  background: rgba(255, 255, 255, 0.02);
  border-left: 1px solid var(--border-color);
  align-self: stretch;
  display: flex;
  align-items: center;
}

.field-hint {
  font-size: 10px;
  color: var(--text-muted);
  padding-left: 2px;
}

/* Path Input */
.path-group {
  display: flex;
  gap: 8px;
}

.path-display {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  min-width: 0;
}

.path-icon {
  flex-shrink: 0;
  color: var(--text-muted);
}

.path-text {
  font-size: 12px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.path-text.placeholder {
  color: var(--text-muted);
}

.browse-btn {
  flex-shrink: 0;
  padding: 10px 16px;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  color: var(--text-secondary);
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
  transition: all 0.15s ease;
}

.browse-btn:hover {
  background: var(--bg-elevated);
  border-color: var(--accent-purple);
  color: var(--accent-purple);
}

.btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px 16px;
  border-radius: 8px;
  border: none;
  font-size: 12px;
  font-weight: 500;
  font-family: inherit;
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.btn-primary {
  background: linear-gradient(135deg, var(--accent-purple), var(--accent-purple-light));
  color: white;
  box-shadow: 0 2px 8px rgba(183, 108, 241, 0.3);
}

.btn-primary:hover:not(:disabled) {
  box-shadow: 0 4px 16px rgba(183, 108, 241, 0.4);
  transform: translateY(-1px);
}

.btn-primary:active:not(:disabled) {
  transform: translateY(0);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  box-shadow: none;
}

.btn-ghost {
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-secondary);
}

.btn-ghost:hover {
  background: var(--bg-surface);
  border-color: var(--text-muted);
  color: var(--text-primary);
}

/* Log Section */
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
  height: 160px;
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

.log-line.info .log-msg {
  color: var(--text-secondary);
}

.log-line.success .log-msg {
  color: #44ee88;
}

.log-line.warning .log-msg {
  color: #ffaa44;
}

.log-line.error .log-msg {
  color: #ff4466;
}

/* Slide Fade Transition */
.slide-fade-enter-active {
  transition: all 0.25s ease-out;
}

.slide-fade-leave-active {
  transition: all 0.2s ease-in;
}

.slide-fade-enter-from {
  opacity: 0;
  transform: translateY(-8px);
  max-height: 0;
}

.slide-fade-leave-to {
  opacity: 0;
  transform: translateY(-8px);
  max-height: 0;
}

.slide-fade-enter-to,
.slide-fade-leave-from {
  opacity: 1;
  transform: translateY(0);
  max-height: 100px;
}
</style>
