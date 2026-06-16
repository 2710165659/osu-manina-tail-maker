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
          <label v-for="preset in presets" :key="preset.name"
            :class="['preset-card', { active: selectedPresetNames.has(preset.name) }]"
            @click="togglePreset(preset.name)">
            <div class="checkbox-box">
              <svg v-if="selectedPresetNames.has(preset.name)" width="10" height="10" viewBox="0 0 10 10" fill="none">
                <path d="M2 5l2.5 2.5L8 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"
                  stroke-linejoin="round" />
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
          每个预设生成 {{ generatedCount }} 张（{{ startLength }}px ~ {{ effectiveEndLength }}px），共 {{ generatedCount *
            selectedPresetNames.size }} 张
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
          <span v-if="generating">生成中... ({{ currentProgress }}/{{ totalCount }})</span>
          <span v-else>开始批量生成</span>
        </button>
      </div>
    </div>

    <!-- 日志区域 -->
    <LogPanel :logs="logs" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useConfig } from '../../composables/useConfig'
import { useToolLogger } from '../../composables/useToolLogger'
import LogPanel from '../shared/LogPanel.vue'

const { presets } = useConfig()

const selectedPresetNames = ref(new Set<string>())
const startLength = ref(10)
const stepSize = ref(10)
const endLength = ref(200)
const outputFolder = ref('')
const generating = ref(false)
const currentProgress = ref(0)
const totalCount = ref(0)

function togglePreset(name: string) {
  if (selectedPresetNames.value.has(name)) {
    selectedPresetNames.value.delete(name)
  } else {
    selectedPresetNames.value.add(name)
  }
}

const { logs, push, clear } = useToolLogger({
  target: 'batch',
  onData: (_target, data) => {
    const d = data as { index?: number; total?: number }
    if (d.index !== undefined) currentProgress.value = d.index
    if (d.total !== undefined) totalCount.value = d.total
  },
})

// 组件卸载时取消后端任务
onUnmounted(() => {
  if (generating.value) {
    invoke('cancel_batch_export')
    generating.value = false
  }
})

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
    outputFolder.value !== '' &&
    !generating.value
  )
})

async function handleBrowseFolder() {
  const { open } = await import('@tauri-apps/plugin-dialog')
  try {
    const selected = await open({ multiple: false, directory: true })
    if (selected) {
      const path = Array.isArray(selected) ? selected[0] : selected
      outputFolder.value = path
      push(`已选择输出文件夹：${path}`, 'info')
    }
  } catch (e) {
    push(`文件夹选择失败：${e}`, 'error')
  }
}

function handleGenerate() {
  if (!canGenerate.value || generating.value) return

  generating.value = true
  currentProgress.value = 0
  clear()

  const selectedPresets = presets.value.filter(p => selectedPresetNames.value.has(p.name))
  if (selectedPresets.length === 0) {
    push('未找到选中的预设', 'error')
    generating.value = false
    return
  }

  // 收集所有配置项
  const configs: any[] = []
  const filenames: string[] = []
  const presetNames: string[] = selectedPresets.map(p => p.name)

  const lengths: number[] = []
  for (let len = startLength.value; len <= effectiveEndLength.value; len += stepSize.value) {
    lengths.push(len)
  }

  for (const preset of selectedPresets) {
    for (const throwLength of lengths) {
      const genConfig = JSON.parse(JSON.stringify(preset.config))
      genConfig.throwLength = throwLength
      const filename = `${preset.name}_${throwLength}px.png`
      configs.push(genConfig)
      filenames.push(filename)
    }
  }

  // 同步 fire-and-forget：后端通过 app:event 流式推送进度
  invoke('batch_export_images', {
    configs,
    outputFolder: outputFolder.value,
    filenames,
    presetNames,
  }).catch((e) => {
    push(`批量导出启动失败：${e}`, 'error')
    generating.value = false
  })
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
</style>
