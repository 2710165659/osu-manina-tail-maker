<template>
  <div class="batch-generate">
    <!-- 描述卡片 -->
    <div class="desc-card">
      <p class="desc-text">根据预设批量生成不同投长度的面尾图片。配置起始长度、步长和终止长度，自动生成一系列图片到目标文件夹。</p>
    </div>

    <!-- 配置区域 -->
    <div class="config-group">
      <!-- 预设选择 -->
      <div class="field">
        <label class="field-label">预设选择</label>
        <div class="preset-grid">
          <label
            v-for="preset in presets"
            :key="preset.name"
            :class="['preset-card', { active: selectedPresetNames.has(preset.name) }]"
            @click="togglePreset(preset.name)"
          >
            <div class="checkbox-box">
              <svg v-if="selectedPresetNames.has(preset.name)" width="10" height="10" viewBox="0 0 10 10" fill="none">
                <path d="M2 5l2.5 2.5L8 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </div>
            <div class="preset-info">
              <span class="preset-name">{{ preset.name }}</span>
              <span v-if="preset.builtin" class="preset-badge">内置</span>
            </div>
          </label>
        </div>
        <span class="field-hint">选择预设作为批量生成的基础配置（可多选）</span>
      </div>

      <!-- 长度配置 -->
      <div class="field">
        <label class="field-label">长度配置</label>
        <div class="length-row">
          <div class="length-item">
            <span class="length-label">起始长度</span>
            <div class="input-wrapper">
              <input type="number" v-model.number="startLength" class="field-input" placeholder="10" min="1" />
              <span class="input-suffix">px</span>
            </div>
          </div>
          <div class="length-item">
            <span class="length-label">步长</span>
            <div class="input-wrapper">
              <input type="number" v-model.number="stepSize" class="field-input" placeholder="10" min="1" />
              <span class="input-suffix">px</span>
            </div>
          </div>
          <div class="length-item">
            <span class="length-label">终止长度</span>
            <div class="input-wrapper">
              <input type="number" v-model.number="endLength" class="field-input" placeholder="200" min="1" />
              <span class="input-suffix">px</span>
            </div>
          </div>
        </div>
        <span class="field-hint">
          每个预设生成 {{ generatedCount }} 张（{{ startLength }}px ~ {{ effectiveEndLength }}px），共 {{ generatedCount * selectedPresetNames.size }} 张
        </span>
      </div>

      <!-- 目标文件夹 -->
      <div class="field">
        <label class="field-label">目标文件夹</label>
        <div class="path-group">
          <div class="path-display">
            <svg class="path-icon" width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M1.5 3.5h4l1.5 2h5.5a1 1 0 011 1v5a1 1 0 01-1 1h-11a1 1 0 01-1-1v-7a1 1 0 011-1z"
                stroke="currentColor" stroke-width="1.1" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            <span class="path-text" :class="{ placeholder: !outputFolder }">
              {{ outputFolder || '请选择输出文件夹' }}
            </span>
          </div>
          <button class="browse-btn" @click="handleBrowseFolder">
            <span>浏览</span>
          </button>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="field">
        <button class="btn btn-primary btn-full" @click="handleGenerate" :disabled="!canGenerate || generating">
          <svg v-if="!generating" width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M7 1.5v11M1.5 7h11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
          <span v-if="generating">生成中... ({{ currentProgress }}/{{ generatedCount * selectedPresetNames.size }})</span>
          <span v-else>开始批量生成</span>
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
import { ref, computed, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useConfig } from '../../composables/useConfig'

const { presets } = useConfig()

const selectedPresetNames = ref(new Set<string>())
const startLength = ref(10)
const stepSize = ref(10)
const endLength = ref(200)
const outputFolder = ref('')
const generating = ref(false)
const currentProgress = ref(0)

function togglePreset(name: string) {
  if (selectedPresetNames.value.has(name)) {
    selectedPresetNames.value.delete(name)
  } else {
    selectedPresetNames.value.add(name)
  }
}

const logContainer = ref<HTMLDivElement>()

interface LogEntry {
  time: string
  message: string
  type: 'info' | 'success' | 'warning' | 'error'
}

const logs = ref<LogEntry[]>([])

// 计算有效终止长度（对齐到步长）
const effectiveEndLength = computed(() => {
  if (stepSize.value <= 0 || startLength.value <= 0) return endLength.value
  const diff = endLength.value - startLength.value
  const steps = Math.floor(diff / stepSize.value)
  return startLength.value + steps * stepSize.value
})

// 计算生成数量
const generatedCount = computed(() => {
  if (stepSize.value <= 0 || startLength.value <= 0) return 0
  const diff = effectiveEndLength.value - startLength.value
  return Math.floor(diff / stepSize.value) + 1
})

// 是否可以生成
const canGenerate = computed(() => {
  return (
    selectedPresetNames.value.size > 0 &&
    startLength.value > 0 &&
    stepSize.value > 0 &&
    endLength.value >= startLength.value &&
    outputFolder.value !== ''
  )
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

async function handleBrowseFolder() {
  const { open } = await import('@tauri-apps/plugin-dialog')
  try {
    const selected = await open({
      multiple: false,
      directory: true,
    })
    if (selected) {
      const path = Array.isArray(selected) ? selected[0] : selected
      outputFolder.value = path
      addLog(`已选择输出文件夹：${path}`, 'info')
    }
  } catch (e) {
    addLog(`文件夹选择失败：${e}`, 'error')
  }
}

async function handleGenerate() {
  if (!canGenerate.value || generating.value) return

  generating.value = true
  currentProgress.value = 0

  const selectedPresets = presets.value.filter(p => selectedPresetNames.value.has(p.name))
  if (selectedPresets.length === 0) {
    addLog('未找到选中的预设', 'error')
    generating.value = false
    return
  }

  const totalPerPreset = generatedCount.value
  const totalTasks = totalPerPreset * selectedPresets.length

  addLog('开始批量生成任务...', 'info')
  addLog(`使用预设：${selectedPresets.map(p => p.name).join('、')}`, 'info')
  addLog(`长度范围：${startLength.value}px ~ ${effectiveEndLength.value}px，步长 ${stepSize.value}px`, 'info')
  addLog(`每个预设生成 ${totalPerPreset} 张，共 ${totalTasks} 张`, 'info')
  addLog(`输出文件夹：${outputFolder.value}`, 'info')

  const lengths: number[] = []
  for (let len = startLength.value; len <= effectiveEndLength.value; len += stepSize.value) {
    lengths.push(len)
  }

  let successCount = 0
  let failCount = 0

  for (const preset of selectedPresets) {
    addLog(`── 预设：${preset.name} ──`, 'info')

    for (const throwLength of lengths) {
      currentProgress.value = successCount + failCount + 1

      const genConfig = JSON.parse(JSON.stringify(preset.config))
      genConfig.throwLength = throwLength

      const baseName = genConfig.image.filename || 'mania-noteL'
      const filename = `${baseName}_${throwLength}px.png`
      const outputPath = `${outputFolder.value}\\${filename}`

      try {
        addLog(`[${currentProgress.value}/${totalTasks}] 正在生成 ${filename}...`, 'info')
        await invoke('export_image', {
          config: genConfig,
          outputPath,
        })
        successCount++
        addLog(`✓ ${filename} 生成成功`, 'success')
      } catch (e) {
        failCount++
        addLog(`✗ ${filename} 生成失败：${e}`, 'error')
      }
    }
  }

  addLog('─'.repeat(30), 'info')
  addLog(`批量生成完成！成功 ${successCount} 张，失败 ${failCount} 张`, successCount > 0 ? 'success' : 'error')

  generating.value = false
  currentProgress.value = 0
}
</script>

<style scoped>
.batch-generate {
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

/* Preset Grid */
.preset-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  max-height: 200px;
  overflow-y: auto;
  padding-right: 4px;
}

.preset-grid::-webkit-scrollbar {
  width: 4px;
}

.preset-grid::-webkit-scrollbar-track {
  background: transparent;
}

.preset-grid::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.08);
  border-radius: 2px;
}

.preset-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.preset-card:hover {
  border-color: rgba(183, 108, 241, 0.3);
  background: rgba(183, 108, 241, 0.03);
}

.preset-card.active {
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
  transition: all 0.2s ease;
  color: white;
}

.preset-card.active .checkbox-box {
  border-color: var(--accent-purple);
  background: var(--accent-purple);
}

.preset-info {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.preset-name {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.preset-badge {
  flex-shrink: 0;
  padding: 1px 6px;
  font-size: 9px;
  font-weight: 500;
  color: var(--accent-purple);
  background: rgba(183, 108, 241, 0.1);
  border-radius: 4px;
}

/* Length Row */
.length-row {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 8px;
}

.length-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.length-label {
  font-size: 11px;
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
  min-width: 0;
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

/* Buttons */
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
</style>
