<template>
  <div class="one-click-length">
    <div class="desc-card">
      <p class="desc-text">一键修改皮肤中面尾的投机取巧长度。选择皮肤文件夹，为不同键数设定各自的目标长度，批量修改。已 lazer 适配的皮肤可在此随意调整，不受影响。</p>
    </div>

    <div class="config-group">
      <!-- 皮肤文件夹 -->
      <div class="field">
        <label class="field-label">皮肤文件夹</label>
        <div class="path-group">
          <div class="path-display">
            <svg class="path-icon" width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M1.5 7.5v3.5a1 1 0 001 1h9a1 1 0 001-1V7.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
              <path d="M7 1.5v6M4.5 5L7 7.5 9.5 5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            <span class="path-text" :class="{ placeholder: !folderPath }">
              {{ folderPath || '请选择皮肤所在文件夹' }}
            </span>
          </div>
          <button class="browse-btn" @click="handleBrowse">
            <span>浏览</span>
          </button>
        </div>
      </div>

      <!-- 键数 + 投长度 -->
      <div class="field">
        <label class="field-label">目标键数 &amp; 投机取巧长度</label>
        <div class="key-length-list" v-if="skinInfo.length > 0">
          <div v-for="info in uniqueKeyInfos" :key="info.keys" :class="['kl-row', { active: throwMap.has(info.keys), invalid: !info.valid }]">
            <label class="kl-check">
              <input type="checkbox" :checked="throwMap.has(info.keys)" :disabled="!info.valid" @change="toggleKey(info.keys)" />
              <span class="kl-label">{{ info.keys }}k</span>
            </label>
            <span v-if="!info.valid" class="kl-badge" title="图片高度不满足 >5000 要求">不合规</span>
            <span v-else class="kl-current">{{ info.current_throw }}px <span v-if="info.is_2x" class="kl-2x">(@2x)</span></span>
            <div class="kl-input-wrap" v-if="throwMap.has(info.keys) && info.valid">
              <input
                type="number"
                class="kl-input"
                :value="throwMap.get(info.keys)"
                @input="e => throwMap.set(info.keys, Number((e.target as HTMLInputElement).value))"
                placeholder="px"
                min="1"
              />
              <span class="kl-suffix">px</span>
            </div>
          </div>
        </div>
        <div v-else class="key-length-empty">
          <span v-if="!folderPath">请先选择皮肤文件夹</span>
          <span v-else>未检测到 NoteImage#L 面尾定义</span>
        </div>
        <span class="field-hint">勾选需要修改的键数，输入目标投长度。默认为当前投长度。</span>
      </div>

      <!-- 备份 -->
      <div class="field">
        <label class="field-label">是否备份原始文件</label>
        <div class="radio-cards">
          <label :class="['radio-card', { active: doBackup }]" @click="doBackup = true">
            <div class="radio-dot"><div class="radio-dot-inner"></div></div>
            <div class="radio-content">
              <span class="radio-title">备份</span>
              <span class="radio-desc">覆盖前将原图备份到 _backup 文件夹。</span>
            </div>
          </label>
          <label :class="['radio-card', { active: !doBackup }]" @click="doBackup = false">
            <div class="radio-dot"><div class="radio-dot-inner"></div></div>
            <div class="radio-content">
              <span class="radio-title">不备份</span>
              <span class="radio-desc">直接覆盖原始图片。</span>
            </div>
          </label>
        </div>
      </div>

      <div class="field">
        <button class="btn btn-primary btn-full" @click="handleModify" :disabled="!canModify">
          <span>{{ modifying ? '修改中...' : '开始修改' }}</span>
        </button>
      </div>
    </div>

    <!-- 日志 -->
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
import { ref, reactive, computed, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const folderPath = ref('')
const throwMap = reactive(new Map<number, number>())
const skinInfo = ref<SkinThrowInfo[]>([])
const doBackup = ref(true)
const modifying = ref(false)
const logContainer = ref<HTMLDivElement>()

interface SkinThrowInfo {
  keys: number
  stem: string
  current_throw: number
  height: number
  valid: boolean
  is_2x: boolean
}

function toggleKey(k: number) {
  const info = skinInfo.value.find(s => s.keys === k)
  if (!info?.valid) return  // 不合规不可勾选
  if (throwMap.has(k)) {
    throwMap.delete(k)
  } else {
    throwMap.set(k, info.current_throw)
  }
}

const canModify = computed(() => {
  if (!folderPath.value) return false
  if (modifying.value) return false
  if (throwMap.size === 0) return false
  for (const v of throwMap.values()) {
    if (!v || v < 1) return false
  }
  return true
})

/// 按 keys 去重的 skinInfo（取第一个匹配的 stem/height/valid）
const uniqueKeyInfos = computed(() => {
  const seen = new Set<number>()
  return skinInfo.value.filter(s => {
    if (seen.has(s.keys)) return false
    seen.add(s.keys)
    return true
  }).sort((a, b) => a.keys - b.keys)
})

interface LogEntry { time: string; message: string; type: 'info' | 'success' | 'warning' | 'error' }
const logs = ref<LogEntry[]>([])

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
      folderPath.value = Array.isArray(selected) ? selected[0] : selected
      addLog(`已选择：${folderPath.value}`, 'info')

      // 加载 skin.ini 信息
      try {
        const info: SkinThrowInfo[] = await invoke('get_skin_throw_info', {
          folderPath: folderPath.value,
        })
        skinInfo.value = info

        // 按 keys 分组去重
        const keySet = new Set(info.map(s => s.keys))
        const keys = [...keySet].sort((a, b) => a - b)

        // 清空旧的 throwMap，预填合规键数
        throwMap.clear()
        if (keys.length > 0) {
          addLog(`检测到键数: ${keys.map(k => k + 'k').join(', ')}`, 'info')
          for (const k of keys) {
            const s = info.find(i => i.keys === k)
            if (s?.valid) {
              throwMap.set(k, s.current_throw)
            }
          }
          // 不合规的键数提示
          const invalid = info.filter(s => !s.valid)
          for (const s of invalid) {
            addLog(`⚠ ${s.keys}k ${s.stem}: 高度 ${s.height}px，不满足 >5000，不可修改`, 'warning')
          }
        } else {
          addLog('未找到任何 NoteImage#L 面尾定义', 'warning')
        }
      } catch (e) {
        addLog(`读取 skin.ini 失败：${e}`, 'error')
      }
    }
  } catch (e) {
    addLog(`文件夹选择失败：${e}`, 'error')
  }
}

async function handleModify() {
  if (!canModify.value) return

  modifying.value = true
  const entries = [...throwMap.entries()].sort((a, b) => a[0] - b[0])
  addLog('开始修改投机取巧长度...', 'info')
  for (const [k, v] of entries) {
    addLog(`  ${k}k → ${v}px`, 'info')
  }
  addLog(`备份：${doBackup.value ? '是' : '否'}`, 'info')

  try {
    const logLines: string[] = await invoke('modify_skin_throw_length', {
      folderPath: folderPath.value,
      keys: entries.map(([k]) => k),
      throws: entries.map(([, v]) => v),
      backup: doBackup.value,
    })
    for (const line of logLines) {
      const type: LogEntry['type'] = line.startsWith('  ✓') ? 'success' : line.startsWith('⚠') ? 'warning' : 'info'
      addLog(line, type)
    }
    addLog('修改完成！', 'success')
  } catch (e) {
    addLog(`修改失败：${e}`, 'error')
  } finally {
    modifying.value = false
  }
}
</script>

<style scoped>
.one-click-length { display: flex; flex-direction: column; gap: 16px; }
.desc-card { display: flex; gap: 12px; padding: 12px 14px; background: rgba(183,108,241,0.04); border: 1px solid rgba(183,108,241,0.12); border-radius: 10px; }
.desc-text { margin: 0; font-size: 12px; line-height: 1.7; color: var(--text-secondary); }
.config-group { display: flex; flex-direction: column; gap: 14px; }
.field { display: flex; flex-direction: column; gap: 8px; }
.field-label { font-size: 12px; font-weight: 500; color: var(--text-secondary); }
.field-hint { font-size: 10px; color: var(--text-muted); padding-left: 2px; }
/* Path */
.path-group { display: flex; gap: 8px; }
.path-display { flex: 1; display: flex; align-items: center; gap: 8px; padding: 10px 12px; background: var(--bg-panel); border: 1px solid var(--border-color); border-radius: 8px; min-width: 0; }
.path-icon { flex-shrink: 0; color: var(--text-muted); }
.path-text { font-size: 12px; color: var(--text-primary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.path-text.placeholder { color: var(--text-muted); }
.browse-btn { flex-shrink: 0; padding: 10px 16px; background: var(--bg-panel); border: 1px solid var(--border-color); border-radius: 8px; color: var(--text-secondary); font-size: 12px; font-family: inherit; cursor: pointer; transition: all 0.15s ease; }
.browse-btn:hover { background: var(--bg-elevated); border-color: var(--accent-purple); color: var(--accent-purple); }
/* Key-length list */
.key-length-list { display: flex; flex-direction: column; gap: 6px; }
.kl-row { display: flex; align-items: center; gap: 10px; padding: 6px 10px; background: var(--bg-panel); border: 1px solid var(--border-color); border-radius: 8px; transition: all 0.15s; }
.kl-row.active { border-color: rgba(183,108,241,0.3); background: rgba(183,108,241,0.03); }
.kl-row.invalid { border-color: rgba(255,170,68,0.3); background: rgba(255,170,68,0.03); }
.kl-check { display: flex; align-items: center; gap: 6px; cursor: pointer; min-width: 48px; flex: 1; min-width: 0; }
.kl-check input { accent-color: var(--accent-purple); flex-shrink: 0; }
.kl-label { font-size: 13px; font-weight: 500; color: var(--text-primary); flex-shrink: 0; }
.kl-badge { font-size: 9px; padding: 1px 6px; border-radius: 4px; background: rgba(255,170,68,0.15); color: #ffaa44; flex-shrink: 0; }
.kl-current { font-size: 11px; color: var(--text-muted); flex-shrink: 0; }
.kl-2x { font-size: 10px; color: var(--accent-purple); opacity: 0.8; }
.kl-input-wrap { display: flex; align-items: center; background: var(--bg-surface); border: 1px solid var(--border-color); border-radius: 6px; overflow: hidden; flex-shrink: 0; }
.kl-input { width: 60px; padding: 5px 8px; background: transparent; border: none; color: var(--text-primary); font-size: 12px; font-family: 'JetBrains Mono', monospace; outline: none; text-align: right; }
.kl-input::-webkit-inner-spin-button, .kl-input::-webkit-outer-spin-button { -webkit-appearance: none; margin: 0; }
.kl-input { -moz-appearance: textfield; }
.kl-suffix { padding: 0 8px; font-size: 10px; color: var(--text-muted); border-left: 1px solid var(--border-color); }
.key-length-empty { font-size: 12px; color: var(--text-muted); padding: 12px; text-align: center; }
/* Radio */
.radio-cards { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
.radio-card { display: flex; align-items: flex-start; gap: 10px; padding: 10px 12px; background: var(--bg-panel); border: 1px solid var(--border-color); border-radius: 8px; cursor: pointer; transition: all 0.2s ease; }
.radio-card:hover { border-color: rgba(183,108,241,0.3); background: rgba(183,108,241,0.03); }
.radio-card.active { border-color: var(--accent-purple); background: rgba(183,108,241,0.06); }
.radio-dot { width: 16px; height: 16px; border-radius: 50%; border: 1.5px solid var(--border-color); display: flex; align-items: center; justify-content: center; flex-shrink: 0; margin-top: 1px; transition: all 0.2s ease; }
.radio-card.active .radio-dot { border-color: var(--accent-purple); }
.radio-dot-inner { width: 8px; height: 8px; border-radius: 50%; background: transparent; transition: all 0.2s ease; }
.radio-card.active .radio-dot-inner { background: var(--accent-purple); }
.radio-content { display: flex; flex-direction: column; gap: 2px; }
.radio-title { font-size: 12px; font-weight: 500; color: var(--text-primary); }
.radio-desc { font-size: 10px; color: var(--text-muted); }
/* Button */
.btn { display: flex; align-items: center; justify-content: center; gap: 8px; padding: 10px 16px; border-radius: 8px; border: none; font-size: 12px; font-weight: 500; font-family: inherit; cursor: pointer; transition: all 0.2s ease; flex-shrink: 0; }
.btn-full { width: 100%; }
.btn-primary { background: linear-gradient(135deg, var(--accent-purple), var(--accent-purple-light)); color: white; box-shadow: 0 2px 8px rgba(183,108,241,0.3); }
.btn-primary:hover:not(:disabled) { box-shadow: 0 4px 16px rgba(183,108,241,0.4); transform: translateY(-1px); }
.btn-primary:active:not(:disabled) { transform: translateY(0); }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; box-shadow: none; }
/* Log */
.log-section { display: flex; flex-direction: column; gap: 8px; }
.log-header { display: flex; align-items: center; gap: 6px; font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.8px; }
.log-header svg { opacity: 0.6; }
.log-content { height: 160px; overflow-y: auto; padding: 12px; background: var(--bg-panel); border: 1px solid var(--border-color); border-radius: 8px; font-family: 'JetBrains Mono', monospace; font-size: 11px; line-height: 1.8; }
.log-content::-webkit-scrollbar { width: 4px; }
.log-content::-webkit-scrollbar-track { background: transparent; }
.log-content::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.08); border-radius: 2px; }
.log-empty { display: flex; align-items: center; gap: 8px; color: var(--text-muted); font-style: italic; }
.log-empty-icon { color: var(--accent-purple); opacity: 0.5; }
.log-line { display: flex; align-items: baseline; gap: 8px; }
.log-time { color: var(--text-muted); opacity: 0.6; flex-shrink: 0; }
.log-marker { color: var(--accent-purple); opacity: 0.4; flex-shrink: 0; }
.log-msg { flex: 1; word-break: break-all; }
.log-line.info .log-msg { color: var(--text-secondary); }
.log-line.success .log-msg { color: #44ee88; }
.log-line.warning .log-msg { color: #ffaa44; }
.log-line.error .log-msg { color: #ff4466; }
</style>
