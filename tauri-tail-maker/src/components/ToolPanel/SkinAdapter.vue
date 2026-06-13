<template>
  <div class="skin-adapter">
    <div class="desc-card">
      <p class="desc-text">修复投皮转换为 lazer 后，面尾拉伸或 KeyD 等图片拉伸的问题。此操作会将原始文件备份到皮肤根目录下的 _backup 文件夹。</p>
    </div>

    <div class="config-group">
      <!-- 皮肤文件夹路径 -->
      <div class="field">
        <label class="field-label">皮肤文件夹</label>
        <div class="path-group">
          <div class="path-display">
            <svg class="path-icon" width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M1.5 7.5v3.5a1 1 0 001 1h9a1 1 0 001-1V7.5" stroke="currentColor" stroke-width="1.1"
                stroke-linecap="round" />
              <path d="M7 1.5v6M4.5 5L7 7.5 9.5 5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round"
                stroke-linejoin="round" />
            </svg>
            <span class="path-text" :class="{ placeholder: !filePath }">{{ filePath || '请选择皮肤所在文件夹' }}</span>
          </div>
          <button class="browse-btn" @click="handleBrowse"><span>浏览</span></button>
        </div>
      </div>

      <!-- 要修复的图片列表（3列网格） -->
      <div class="field">
        <label class="field-label" v-if="filePath && repairStems.length > 0">要修复的图片 ({{ checkedCount }}/{{
          repairStems.length }})</label>
        <template v-if="filePath && repairStems.length > 0">
          <span class="field-hint">共 {{ repairStems.length }} 张待修复图片</span>
          <div class="repair-scroll">
            <div class="repair-grid">
              <label v-for="item in repairStems" :key="item.stem" :class="['repair-item', { active: item.checked }]">
                <input type="checkbox" v-model="item.checked" />
                <span class="ri-stem">{{ item.stem }}</span>
                <span
                  :class="['ri-tag', item.kind === 'tail' ? 'ri-tail' : item.kind === 'keyd' ? 'ri-keyd' : 'ri-key']">{{
                    item.kind === 'tail' ? '面尾' : item.kind === 'keyd' ? 'KeyD' : 'Key' }}</span>
              </label>
            </div>
          </div>
          <span class="field-hint">勾选需要修复的图片，默认全选。</span>
        </template>
        <div v-else-if="filePath && !loadingInfo" class="repair-placeholder">未找到需要修复的图片</div>
        <div v-else class="repair-placeholder">请先选择皮肤文件夹路径</div>
      </div>

      <div class="field">
        <button class="btn btn-primary btn-full" @click="handleRepair" :disabled="!canRepair">
          <span>{{ repairing ? '修复中...' : '开始修复' }}</span>
        </button>
      </div>
    </div>

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
          <div class="log-empty"><span class="log-empty-icon">~</span><span>等待操作...</span></div>
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const filePath = ref('')
const loadingInfo = ref(false)

interface RepairStemItem { stem: string; kind: 'tail' | 'key' | 'keyd'; checked: boolean }
const repairStems = ref<RepairStemItem[]>([])

const checkedCount = computed(() => repairStems.value.filter(s => s.checked).length)
const repairing = ref(false)
const logContainer = ref<HTMLDivElement>()

interface LogEntry { time: string; message: string; type: 'info' | 'success' | 'warning' | 'error' }
const logs = ref<LogEntry[]>([])

const canRepair = computed(() => filePath.value && !repairing.value && checkedCount.value > 0)

function addLog(msg: string, type: LogEntry['type'] = 'info') {
  const now = new Date()
  const time = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`
  logs.value.push({ time, message: msg, type })
  nextTick(() => { if (logContainer.value) logContainer.value.scrollTop = logContainer.value.scrollHeight })
}

async function handleBrowse() {
  const { open } = await import('@tauri-apps/plugin-dialog')
  try {
    const selected = await open({ multiple: false, directory: true })
    if (selected) {
      filePath.value = Array.isArray(selected) ? selected[0] : selected
      addLog(`已选择：${filePath.value}`, 'info')
      await loadRepairInfo()
    }
  } catch (e) { addLog(`文件选择失败：${e}`, 'error') }
}

async function loadRepairInfo() {
  loadingInfo.value = true
  repairStems.value = []
  addLog('正在读取皮肤信息...', 'info')
  try {
    const seen = new Set<string>()
    // NoteImage#L (tail)
    try {
      const tails: { stem: string; image_path: string }[] = await invoke('get_image_key_info', { folderPath: filePath.value })
      for (const t of tails) {
        if (seen.has(t.stem)) continue
        seen.add(t.stem)
        repairStems.value.push({ stem: t.stem, kind: 'tail', checked: true })
      }
    } catch (e) { addLog(`面尾列表加载失败: ${e}`, 'warning') }
    // Key + KeyD
    try {
      const kds: { stem: string; as_key: number[]; as_keyd: number[] }[] = await invoke('get_keyd_list', { folderPath: filePath.value })
      for (const kd of kds) {
        if (kd.as_key.length > 0 && !seen.has(kd.stem)) {
          seen.add(kd.stem)
          repairStems.value.push({ stem: kd.stem, kind: 'key', checked: true })
        }
        if (kd.as_keyd.length > 0 && !seen.has(kd.stem)) {
          seen.add(kd.stem)
          repairStems.value.push({ stem: kd.stem, kind: 'keyd', checked: true })
        }
      }
    } catch (e) { addLog(`Key/KeyD 列表加载失败: ${e}`, 'warning') }
    addLog(`已加载 ${repairStems.value.length} 个待修复图片`, 'success')
  } catch (e) { addLog(`加载失败: ${e}`, 'error') }
  finally { loadingInfo.value = false }
}

function classifyLog(msg: string): LogEntry['type'] {
  if (msg.startsWith('  ✓') || msg.includes('完成')) return 'success'
  if (msg.includes('⚠') || msg.startsWith('  ✗')) return 'warning'
  return 'info'
}

async function handleRepair() {
  if (!canRepair.value) return
  repairing.value = true
  addLog(`文件：${filePath.value}`, 'info')
  addLog('开始修复任务...', 'info')

  try {
    addLog('--- 面尾修复 ---', 'info')
    const logLines1: string[] = await invoke('repair_lazer_tail_folder', { folderPath: filePath.value })
    for (const line of logLines1) { addLog(line, classifyLog(line)) }

    addLog('--- Key/KeyD 修复 ---', 'info')
    const logLines2: string[] = await invoke('repair_key_image_folder', { folderPath: filePath.value, mode: 'all' })
    for (const line of logLines2) { addLog(line, classifyLog(line)) }

    addLog('全部修复任务完成！', 'success')
  } catch (e) { addLog(`修复失败: ${e}`, 'error') }
  finally { repairing.value = false }
}
</script>

<style scoped>
.skin-adapter {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

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

.config-group {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

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

.field-hint {
  font-size: 10px;
  color: var(--text-muted);
  padding-left: 2px;
}

/* Repair placeholder */
.repair-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 120px;
  padding: 24px;
  font-size: 13px;
  color: var(--text-muted);
  text-align: center;
  border: 1px dashed var(--border-color);
  border-radius: 8px;
  background: var(--bg-panel);
}

/* Path */
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

/* Repair grid (3 per row, scrollable) */
.repair-scroll {
  max-height: 320px;
  overflow-y: auto;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 6px;
  background: var(--bg-panel);
}

.repair-scroll::-webkit-scrollbar {
  width: 4px;
}

.repair-scroll::-webkit-scrollbar-track {
  background: transparent;
}

.repair-scroll::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.08);
  border-radius: 2px;
}

.repair-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 3px;
}

.repair-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 10px;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.repair-item.active {
  border-color: rgba(183, 108, 241, 0.3);
  background: rgba(183, 108, 241, 0.03);
}

.repair-item input {
  accent-color: var(--accent-purple);
  margin: 0;
  width: 12px;
  height: 12px;
  flex-shrink: 0;
}

.ri-stem {
  font-size: 12px;
  color: var(--text-primary);
  font-family: 'JetBrains Mono', monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.ri-tag {
  font-size: 10px;
  padding: 2px 5px;
  border-radius: 3px;
  flex-shrink: 0;
}

.ri-tail {
  background: rgba(100, 200, 255, 0.15);
  color: #64c8ff;
}

.ri-key {
  background: rgba(100, 255, 160, 0.15);
  color: #64ffa0;
}

.ri-keyd {
  background: rgba(255, 170, 68, 0.15);
  color: #ffaa44;
}

/* Button */
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

/* Log */
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
