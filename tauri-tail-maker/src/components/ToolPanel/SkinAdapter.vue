<template>
  <div class="skin-adapter">
    <!-- 描述卡片 -->
    <div class="desc-card">
      <p class="desc-text">修复投皮转换为 lazer 后，面尾拉伸或 KeyD 等图片拉伸的问题。支持按皮肤或 osk 文件批量修复。</p>
    </div>

    <!-- 配置区域 -->
    <div class="config-group">
      <!-- 皮肤选择 -->
      <div class="field">
        <label class="field-label">皮肤选择</label>
        <div class="radio-cards">
          <label :class="['radio-card', { active: skinMode === 'folder' }]" @click="skinMode = 'folder'">
            <div class="radio-dot">
              <div class="radio-dot-inner"></div>
            </div>
            <div class="radio-content">
              <span class="radio-title">皮肤文件夹</span>
              <span class="radio-desc">选择皮肤所在文件夹，根据 skin.ini 查找并修复。</span>
            </div>
          </label>
          <label :class="['radio-card', { active: skinMode === 'osk' }]" @click="skinMode = 'osk'">
            <div class="radio-dot">
              <div class="radio-dot-inner"></div>
            </div>
            <div class="radio-content">
              <span class="radio-title">osk 文件</span>
              <span class="radio-desc">选择 .osk 文件，自动解析并修复有问题的图片。</span>
            </div>
          </label>
        </div>
      </div>

      <!-- 修复模式 (可多选) -->
      <div class="field">
        <label class="field-label">修复模式</label>
        <div class="checkbox-cards">
          <label :class="['check-card', { active: repairModes.has('tail') }]" @click="toggleRepairMode('tail')">
            <div class="checkbox-box">
              <svg v-if="repairModes.has('tail')" width="10" height="10" viewBox="0 0 10 10" fill="none">
                <path d="M2 5l2.5 2.5L8 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </div>
            <div class="checkbox-content">
              <span class="checkbox-title">面尾</span>
              <span class="checkbox-desc">修复面尾图片因 ColumnWidth 导致的拉伸问题。</span>
            </div>
          </label>
          <label :class="['check-card', { active: repairModes.has('keyd') }]" @click="toggleRepairMode('keyd')">
            <div class="checkbox-box">
              <svg v-if="repairModes.has('keyd')" width="10" height="10" viewBox="0 0 10 10" fill="none">
                <path d="M2 5l2.5 2.5L8 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </div>
            <div class="checkbox-content">
              <span class="checkbox-title">Key + KeyD</span>
              <span class="checkbox-desc">修复 KeyImage# 和 KeyImage#D 图片因尺寸不一致导致的问题。</span>
            </div>
          </label>
        </div>
      </div>

      <!-- 备份选项 -->
      <div class="field">
        <label class="field-label">是否备份原始文件</label>
        <div class="radio-cards">
          <label :class="['radio-card', { active: doBackup }]" @click="doBackup = true">
            <div class="radio-dot">
              <div class="radio-dot-inner"></div>
            </div>
            <div class="radio-content">
              <span class="radio-title">备份</span>
              <span class="radio-desc">覆盖前将原始图片备份到 _backup 文件夹。</span>
            </div>
          </label>
          <label :class="['radio-card', { active: !doBackup }]" @click="doBackup = false">
            <div class="radio-dot">
              <div class="radio-dot-inner"></div>
            </div>
            <div class="radio-content">
              <span class="radio-title">不备份</span>
              <span class="radio-desc">直接覆盖原始图片。</span>
            </div>
          </label>
        </div>
      </div>

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
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="field">
        <button class="btn btn-primary btn-full" @click="handleRepair" :disabled="!canRepair">
          <span>{{ repairing ? '修复中...' : '开始修复' }}</span>
        </button>
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
import { ref, computed, reactive, nextTick, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const skinMode = ref<'osk' | 'folder'>('folder')
const filePath = ref('')

watch(skinMode, () => {
  filePath.value = ''
})

// 修复模式：面尾、KeyD，默认全选
const repairModes = reactive(new Set<string>(['tail', 'keyd']))

function toggleRepairMode(mode: string) {
  if (repairModes.has(mode)) {
    repairModes.delete(mode)
  } else {
    repairModes.add(mode)
  }
}

const doBackup = ref(true)
const repairing = ref(false)
const logContainer = ref<HTMLDivElement>()

interface LogEntry {
  time: string
  message: string
  type: 'info' | 'success' | 'warning' | 'error'
}

const logs = ref<LogEntry[]>([])

const pathPlaceholder = computed(() => {
  if (skinMode.value === 'osk') return '请选择 .osk 文件'
  return '请选择皮肤所在文件夹'
})

const canRepair = computed(() => {
  if (!filePath.value) return false
  if (repairing.value) return false
  if (repairModes.size === 0) return false
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

    if (skinMode.value === 'osk') {
      selected = await open({
        multiple: false,
        filters: [{ name: 'osk 文件', extensions: ['osk'] }],
      })
    } else {
      selected = await open({
        multiple: false,
        directory: true,
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

async function handleRepair() {
  if (!canRepair.value) return

  // osk 模式：一键命令
  if (skinMode.value === 'osk') {
    const modes: string[] = []
    if (repairModes.has('tail')) modes.push('tail')
    if (repairModes.has('keyd')) modes.push('keyd')

    repairing.value = true
    addLog('开始 osk 文件修复...', 'info')
    addLog(`文件：${filePath.value}`, 'info')
    addLog(`修复模式：${[...repairModes].join('、')}`, 'info')
    addLog(`备份：${doBackup.value ? '是' : '否'}`, 'info')

    try {
      const logLines: string[] = await invoke('repair_lazer_osk', {
        oskPath: filePath.value,
        backup: doBackup.value,
        modes,
      })
      for (const line of logLines) {
        const type = classifyLog(line)
        addLog(line, type)
      }
      addLog('osk 修复完成！', 'success')
    } catch (e) {
      addLog(`修复失败: ${e}`, 'error')
    } finally {
      repairing.value = false
    }
    return
  }

  // 文件夹模式
  repairing.value = true
  addLog('开始修复任务...', 'info')
  addLog(`皮肤目录：${filePath.value}`, 'info')

  const repairLabels: string[] = []
  if (repairModes.has('tail')) repairLabels.push('面尾')
  if (repairModes.has('keyd')) repairLabels.push('KeyD')
  addLog(`修复模式：${repairLabels.join('、')}`, 'info')
  addLog(`备份原始文件：${doBackup.value ? '是' : '否'}`, 'info')

  try {
    // 面尾修复
    if (repairModes.has('tail')) {
      addLog('--- 面尾修复 ---', 'info')
      const logLines: string[] = await invoke('repair_lazer_tail_folder', {
        folderPath: filePath.value,
        backup: doBackup.value,
      })
      for (const line of logLines) {
        const type = classifyLog(line)
        addLog(line, type)
      }
    }

    // Key 图片修复 (KeyImage# + KeyImage#D)
    if (repairModes.has('keyd')) {
      addLog('--- Key 图片修复 ---', 'info')
      const logLines: string[] = await invoke('repair_key_image_folder', {
        folderPath: filePath.value,
        backup: doBackup.value,
        mode: 'all',
      })
      for (const line of logLines) {
        const type = classifyLog(line)
        addLog(line, type)
      }
    }

    addLog('全部修复任务完成！', 'success')
  } catch (e) {
    addLog(`修复失败: ${e}`, 'error')
  } finally {
    repairing.value = false
  }
}

/// 根据日志内容猜测日志类型
function classifyLog(msg: string): LogEntry['type'] {
  if (msg.startsWith('  ✓') || msg.includes('完成') || msg.includes('均存在')) return 'success'
  if (msg.startsWith('  ✗') || msg.startsWith('发现') || msg.includes('⚠')) return 'warning'
  return 'info'
}

</script>

<style scoped>
.skin-adapter {
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

/* Checkbox Cards — multi-select */
.checkbox-cards {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.check-card {
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

.check-card:hover {
  border-color: rgba(183, 108, 241, 0.3);
  background: rgba(183, 108, 241, 0.03);
}

.check-card.active {
  border-color: var(--accent-purple);
  background: rgba(183, 108, 241, 0.06);
}

.checkbox-box {
  width: 16px;
  height: 16px;
  border-radius: 4px;
  border: 1.5px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-top: 1px;
  transition: all 0.2s ease;
  color: var(--accent-purple);
}

.check-card.active .checkbox-box {
  border-color: var(--accent-purple);
  background: var(--accent-purple);
  color: white;
}

.checkbox-content {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.checkbox-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
}

.checkbox-desc {
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

.btn-full {
  width: 100%;
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
