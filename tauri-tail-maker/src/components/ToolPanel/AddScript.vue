<template>
  <div class="add-script">
    <!-- 描述卡片 -->
    <div class="desc-card">
      <p class="desc-text">为皮肤添加一个独立的“一键修改面尾”程序，放在皮肤根目录的 <code>scripts</code> 文件夹下。双击程序即可快速切换投长度或使用其他预设，免去手动替换文件的麻烦。</p>
    </div>

    <!-- 配置区域 -->
    <div class="config-group">
      <!-- 皮肤文件夹 -->
      <div class="field">
        <label class="field-label">皮肤文件夹</label>
        <div class="path-group">
          <div class="path-display">
            <svg class="path-icon" width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M1.5 3.5h4l1.5 2h5.5a1 1 0 011 1v5a1 1 0 01-1 1h-11a1 1 0 01-1-1v-7a1 1 0 011-1z" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            <span class="path-text" :class="{ placeholder: !filePath }">
              {{ filePath || '请选择皮肤文件夹' }}
            </span>
          </div>
          <button class="browse-btn" @click="handleBrowse">
            <span>浏览</span>
          </button>
        </div>
      </div>

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
        <span class="field-hint">可选，选择要写入脚本的预设，脚本运行时可从这些预设中切换</span>
      </div>

      <!-- 操作按钮 -->
      <div class="field">
        <button class="btn btn-primary btn-full" @click="handleAddScript" :disabled="!canExecute || executing">
          <svg v-if="!executing" width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M3.5 5L7 8.5l3.5-3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"
              stroke-linejoin="round" />
            <path d="M7 1.5v7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            <path d="M2 12.5h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
          <span v-if="executing">处理中...</span>
          <span v-else>添加脚本</span>
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
import { useConfig } from '../../composables/useConfig'

const { presets } = useConfig()

const filePath = ref('')
const selectedPresetNames = ref(new Set<string>())
const executing = ref(false)
const logContainer = ref<HTMLDivElement>()

interface LogEntry {
  time: string
  message: string
  type: 'info' | 'success' | 'warning' | 'error'
}

const logs = ref<LogEntry[]>([])

const canExecute = computed(() => {
  return !!filePath.value
})

function togglePreset(name: string) {
  if (selectedPresetNames.value.has(name)) {
    selectedPresetNames.value.delete(name)
  } else {
    selectedPresetNames.value.add(name)
  }
}

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
    const selected = await open({
      multiple: false,
      directory: true,
    })

    if (selected) {
      const path = Array.isArray(selected) ? selected[0] : selected
      const { invoke } = await import('@tauri-apps/api/core')
      const valid = await invoke('check_skin_ini', { folderPath: path })
      if (!valid) {
        addLog(`✗ 所选文件夹不包含 skin.ini，请选择有效的皮肤目录`, 'error')
        return
      }
      filePath.value = path
      addLog(`已选择：${path}`, 'info')
    }
  } catch (e) {
    addLog(`文件选择失败：${e}`, 'error')
  }
}

async function handleAddScript() {
  if (!canExecute.value || executing.value) return

  executing.value = true

  const selectedPresets = presets.value.filter(p => selectedPresetNames.value.has(p.name))

  addLog('开始添加脚本任务...', 'info')
  addLog(`文件路径：${filePath.value}`, 'info')
  addLog(`选中预设：${selectedPresets.map(p => p.name).join('、')}`, 'info')

  try {
    const { invoke } = await import('@tauri-apps/api/core')

    // 导出预设图片（完整分辨率）
    const presetImages: [string, number[]][] = []
    if (selectedPresets.length > 0) {
      addLog('正在导出预设图片...', 'info')
      for (const preset of selectedPresets) {
        try {
          const base64 = await invoke<string>('export_image_bytes', { config: preset.config })
          // 将 base64 转换为字节数组
          const binaryStr = atob(base64)
          const bytes = Array.from(binaryStr, c => c.charCodeAt(0))
          presetImages.push([preset.name, bytes])
          addLog(`  ✓ 导出预设：${preset.name}`, 'info')
        } catch (e) {
          addLog(`  ✗ 导出预设 ${preset.name} 失败：${e}`, 'warning')
        }
      }
    }

    addLog('正在复制外部工具...', 'info')
    const result = await invoke('copy_external_tool_with_presets', {
      targetPath: filePath.value,
      presetImages: presetImages,
    })
    addLog(`✓ 外部工具已复制到：${result}`, 'success')

    if (presetImages.length > 0) {
      addLog(`✓ 已添加 ${presetImages.length} 个预设图片`, 'success')
    }

    addLog('✓ 脚本添加完成！', 'success')
  } catch (e) {
    addLog(`✗ 操作失败：${e}`, 'error')
  } finally {
    executing.value = false
  }
}
</script>

<style scoped>
.add-script {
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

.desc-text code {
  padding: 1px 5px;
  font-size: 11px;
  font-family: 'JetBrains Mono', monospace;
  color: var(--accent-purple);
  background: rgba(183, 108, 241, 0.1);
  border-radius: 4px;
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
