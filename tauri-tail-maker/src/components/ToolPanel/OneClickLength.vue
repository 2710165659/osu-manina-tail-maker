<template>
  <div class="one-click-length">
    <div class="desc-card">
      <p class="desc-text">一键修改面尾。为不同键数设定目标投长度，可选预设替换面尾图片、修复 Key/KeyD 拉伸。此操作会将原始文件备份到皮肤根目录下的 _backup 文件夹。</p>
    </div>

    <div class="config-group">
      <!-- 修复模式 -->
      <div class="field">
        <label class="field-label">修复模式</label>
        <div class="radio-cards">
          <label :class="['radio-card', { active: workMode === 'lazer' }]" @click="workMode = 'lazer'">
            <div class="radio-dot">
              <div class="radio-dot-inner"></div>
            </div>
            <div class="radio-content">
              <span class="radio-title">Lazer</span>
              <span class="radio-desc">修改投长度后拉伸到 ColumnWidth×1.6 32800。</span>
            </div>
          </label>
          <label :class="['radio-card', { active: workMode === 'stable' }]" @click="workMode = 'stable'">
            <div class="radio-dot">
              <div class="radio-dot-inner"></div>
            </div>
            <div class="radio-content">
              <span class="radio-title">Stable</span>
              <span class="radio-desc">仅修改投长度，不拉伸图片。</span>
            </div>
          </label>
        </div>
      </div>

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

      <!-- ===== 区块 A: Key/KeyD 修复列表（仅 Lazer） ===== -->
      <div class="field" v-if="workMode === 'lazer'">
        <label class="field-label">Key/KeyD 修复<span v-if="filePath && keydInfos.length > 0"> ({{ keydChecked.size }}/{{
          keydInfos.length }})</span></label>
        <template v-if="filePath && keydInfos.length > 0">
          <span class="field-hint">共 {{ keydInfos.length }} 张 Key/KeyD 图片</span>
          <div class="repair-scroll">
            <div class="repair-grid">
              <label v-for="kd in keydInfos" :key="kd.stem"
                :class="['repair-item', { active: keydChecked.has(kd.stem) }]">
                <input type="checkbox" :checked="keydChecked.has(kd.stem)" @change="toggleKeyd(kd.stem)" />
                <span class="ri-stem">{{ kd.stem }}</span>
                <span v-if="kd.as_key.length > 0" class="ri-tag ri-key">Key</span>
                <span v-if="kd.as_keyd.length > 0" class="ri-tag ri-keyd">KeyD</span>
              </label>
            </div>
          </div>
          <span class="field-hint">勾选需要修复的 Key/KeyD 图片。</span>
        </template>
        <div v-else-if="filePath && loadingInfo" class="repair-placeholder">正在加载...</div>
        <div v-else-if="filePath && !loadingInfo" class="repair-placeholder">未找到 Key/KeyD 图片</div>
        <div v-else class="repair-placeholder">请先选择皮肤文件夹路径</div>
      </div>

      <!-- ===== 区块 B: 预设替换 ===== -->
      <div class="field">
        <label class="field-label">预设替换<span v-if="filePath && imageKeyInfos.length > 0"> ({{ presetCount }}/{{
          imageKeyInfos.length }})</span></label>
        <template v-if="filePath && imageKeyInfos.length > 0">
          <span class="field-hint">共 {{ imageKeyInfos.length }} 张面尾图片可替换</span>
          <div class="preset-scroll">
            <div class="preset-table">
              <div v-for="ik in imageKeyInfos" :key="ik.stem" class="preset-row">
                <span class="psr-stem" :title="ik.image_path">{{ ik.stem }}</span>
                <div class="psr-usage">
                  <span v-for="u in ik.used_by" :key="u.keys" class="ps-usage-item">{{ u.keys }}k (列{{
                    u.columns.join(',') }})</span>
                </div>
                <div class="psr-preset">
                  <div v-if="stemPresets[ik.stem]" class="preset-selected" @click="openPresetDialog(ik.stem)">
                    <img v-if="stemPresets[ik.stem]?.image_path" :src="presetSrc(stemPresets[ik.stem]!.image_path)"
                      class="preset-thumb" />
                    <span class="preset-name-sm">{{ stemPresets[ik.stem]!.name }}</span>
                    <button class="preset-clear" @click.stop="stemPresets[ik.stem] = null">×</button>
                  </div>
                  <button v-else class="preset-pick-btn" @click="openPresetDialog(ik.stem)">选择预设</button>
                </div>
              </div>
            </div>
          </div>
          <span class="field-hint">为每张面尾图片选择预设替换。同一 stem 被多个键数共享时只需选一次。</span>
        </template>
        <div v-else-if="filePath && loadingInfo" class="repair-placeholder">正在加载...</div>
        <div v-else-if="filePath && !loadingInfo" class="repair-placeholder">未找到面尾图片</div>
        <div v-else class="repair-placeholder">请先选择皮肤文件夹路径</div>
      </div>

      <!-- ===== 区块 C: 修改投长度 ===== -->
      <div class="field">
        <label class="field-label">修改投长度<span v-if="filePath && uniqueKeyInfos.length > 0"> ({{ throwMap.size }}/{{
          uniqueKeyInfos.length }})</span></label>
        <template v-if="filePath && uniqueKeyInfos.length > 0">
          <span class="field-hint">共 {{ uniqueKeyInfos.length }} 个键数<span v-if="computingThrows"> —
              正在计算投长度...</span></span>
          <div class="throw-scroll">
            <div class="throw-grid">
              <label v-for="info in uniqueKeyInfos" :key="info.keys"
                :class="['throw-card', { active: throwMap.has(info.keys), invalid: !info.valid }]">
                <input type="checkbox" :checked="throwMap.has(info.keys)" :disabled="!info.valid"
                  @change="toggleKey(info.keys)" />
                <span class="tc-keys">{{ info.keys }}k</span>
                <input type="number" class="tc-input" :value="throwMap.get(info.keys) ?? ''"
                  :disabled="!throwMap.has(info.keys) || !info.valid"
                  @input="e => throwMap.set(info.keys, Number((e.target as HTMLInputElement).value))" placeholder="-"
                  min="1" />
                <span class="tc-orig">{{ info.valid ? `原: ${getModeThrow(info)}` : '不合规' }}</span>
              </label>
            </div>
          </div>
          <span class="field-hint">勾选键数并输入目标投长度。</span>
        </template>
        <div v-else-if="filePath && loadingInfo" class="repair-placeholder">正在加载...</div>
        <div v-else-if="filePath && !loadingInfo" class="repair-placeholder">未检测到 NoteImage#L 面尾定义</div>
        <div v-else class="repair-placeholder">请先选择皮肤文件夹路径</div>
      </div>

      <div class="field">
        <button class="btn btn-primary btn-full" @click="handleModify" :disabled="!canModify">
          <span>{{ modifying ? '修改中...' : '开始修改' }}</span>
        </button>
      </div>
    </div>

    <!-- 预设选择对话框 -->
    <div class="modal-overlay" v-if="presetDialogStem !== null" @mousedown.self="presetDialogStem = null">
      <div class="preset-modal">
        <div class="modal-header">
          <span class="modal-title">选择预设 - {{ presetDialogStem }}</span>
          <button class="modal-close" @click="presetDialogStem = null">×</button>
        </div>
        <div class="modal-body">
          <div class="preset-grid">
            <div v-for="preset in presets" :key="preset.name"
              :class="['preset-card', { active: stemPresets[presetDialogStem]?.name === preset.name }]"
              @click="selectPreset(presetDialogStem, preset)">
              <div class="preset-img-wrap">
                <img :src="presetSrc(preset.image_path)" class="preset-img" />
              </div>
              <span class="preset-label">{{ preset.name }}</span>
            </div>
          </div>
        </div>
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
import { ref, reactive, computed, nextTick, watch } from 'vue'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'

function presetSrc(path: string): string {
  return path.startsWith('data:') ? path : convertFileSrc(path)
}

// Folder path
const filePath = ref('')

// Work mode
const workMode = ref<'lazer' | 'stable'>('lazer')

// Recompute throwMap defaults when mode changes
watch(workMode, () => {
  addLog(`切换模式: ${workMode.value === 'lazer' ? 'Lazer' : 'Stable'}`, 'info')
  if (skinInfo.value.length === 0) return
  for (const [k] of throwMap) {
    const s = skinInfo.value.find(i => i.keys === k)
    if (s?.valid) {
      const def = getModeThrow(s)
      throwMap.set(k, typeof def === 'number' ? def : s.current_throw)
    }
  }
  // 切到 Lazer 时若投长度未计算则触发计算
  if (workMode.value === 'lazer') {
    const needCompute = skinInfo.value.some(s => s.valid && s.lazer_throw === 0)
    if (needCompute) computeAllThrows()
  }
})

// Throw info
interface SkinThrowInfo {
  keys: number; stem: string; column_width: number; current_throw: number; lazer_throw: number
  height: number; valid: boolean; is_2x: boolean
}
const skinInfo = ref<SkinThrowInfo[]>([])
const throwMap = reactive(new Map<number, number>())
const loadingInfo = ref(false)

function getModeThrow(info: SkinThrowInfo): number | string {
  if (workMode.value === 'lazer') {
    return info.lazer_throw > 0 ? info.lazer_throw : '…'
  }
  return info.current_throw
}

const computingThrows = ref(false)

// Key/KeyD info
interface KeydStemInfo { stem: string; as_key: number[]; as_keyd: number[] }
const keydInfos = ref<KeydStemInfo[]>([])
const keydChecked = reactive(new Set<string>())

function toggleKeyd(stem: string) {
  if (keydChecked.has(stem)) { keydChecked.delete(stem) }
  else { keydChecked.add(stem) }
}

// Image-key info (preset section)
interface KeyColumnEntry { keys: number; columns: number[] }
interface ImageKeyInfo { stem: string; image_path: string; used_by: KeyColumnEntry[] }
const imageKeyInfos = ref<ImageKeyInfo[]>([])

// Presets
interface PresetInfo { name: string; image_path: string }
const presets = ref<PresetInfo[]>([])
const stemPresets = reactive<Record<string, PresetInfo | null>>({})
const presetDialogStem = ref<string | null>(null)

// Log
const modifying = ref(false)
const logContainer = ref<HTMLDivElement>()
interface LogEntry { time: string; message: string; type: 'info' | 'success' | 'warning' | 'error' }
const logs = ref<LogEntry[]>([])

// Computed
const presetCount = computed(() => Object.values(stemPresets).filter(Boolean).length)

const uniqueKeyInfos = computed(() => {
  const seen = new Set<number>()
  return skinInfo.value.filter(s => {
    if (seen.has(s.keys)) return false
    seen.add(s.keys)
    return true
  }).sort((a, b) => a.keys - b.keys)
})

const canModify = computed(() => {
  if (!filePath.value || modifying.value) return false
  // Key/KeyD 修复：至少勾选一项
  if (keydChecked.size > 0) return true
  // 预设替换：至少一项分配了预设
  if (presetCount.value > 0) return true
  // 修改投长度：至少一项有效
  for (const v of throwMap.values()) { if (v && v >= 1) return true }
  return false
})

function toggleKey(k: number) {
  const info = skinInfo.value.find(s => s.keys === k)
  if (!info?.valid) return
  if (throwMap.has(k)) { throwMap.delete(k) }
  else {
    const def = getModeThrow(info)
    throwMap.set(k, typeof def === 'number' ? def : info.current_throw)
  }
}

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
      const path = Array.isArray(selected) ? selected[0] : selected
      const valid = await invoke('check_skin_ini', { folderPath: path })
      if (!valid) {
        addLog(`✗ 所选文件夹不包含 skin.ini，请选择有效的皮肤目录`, 'error')
        return
      }
      filePath.value = path
      addLog(`已选择：${filePath.value}`, 'info')
      await loadAll()
    }
  } catch (e) {
    addLog(`文件选择失败：${e}`, 'error')
  }
}

async function loadAll() {
  loadingInfo.value = true
  throwMap.clear()
  throwCache.clear()
  Object.keys(stemPresets).forEach(k => delete stemPresets[k])
  keydChecked.clear()
  keydInfos.value = []
  imageKeyInfos.value = []
  presets.value = []
  skinInfo.value = []

  // Phase 1: Key/KeyD
  await loadKeydList()
  // Phase 2: Presets
  await loadPresetList()
  // Phase 3: Throw info + computation
  await loadThrowInfo()

  loadingInfo.value = false
}

// Throw cache: key = stem → Promise<number> (column_width does not affect throw length)
const throwCache = new Map<string, Promise<number>>()

async function loadKeydList() {
  if (workMode.value !== 'lazer') return
  addLog('=== 检测 Key、KeyD ===', 'info')
  try {
    const kd: KeydStemInfo[] = await invoke('get_keyd_list', { folderPath: filePath.value })
    keydInfos.value = kd
    addLog(`已加载 ${kd.length} 个 Key/KeyD 图片`, 'success')
  } catch (e) { addLog(`Key/KeyD 列表加载失败: ${e}`, 'warning'); keydInfos.value = [] }
}

async function loadPresetList() {
  addLog('=== 加载预设 ===', 'info')
  try {
    const ik: ImageKeyInfo[] = await invoke('get_image_key_info', { folderPath: filePath.value })
    imageKeyInfos.value = ik
    addLog(`已加载 ${ik.length} 个图片关联`, 'info')
  } catch (e) { addLog(`图片关联加载失败: ${e}`, 'warning'); imageKeyInfos.value = [] }

  try {
    const p: PresetInfo[] = await invoke('load_presets', { skinRoot: filePath.value })
    presets.value = p
    if (p.length > 0) addLog(`已加载 ${p.length} 个预设`, 'success')
    else addLog('未找到预设图片', 'info')
  } catch (e) { addLog(`预设加载失败: ${e}`, 'warning') }
}

async function loadThrowInfo() {
  addLog('=== 计算投长度 ===', 'info')
  try {
    const info: SkinThrowInfo[] = await invoke('get_skin_throw_info', { folderPath: filePath.value })
    skinInfo.value = info
    addLog('皮肤信息读取完成', 'success')

    const keySet = new Set(info.map(s => s.keys))
    const keys = [...keySet].sort((a, b) => a - b)

    if (keys.length > 0) {
      addLog(`检测到键数: ${keys.map(k => k + 'k').join(', ')}`, 'info')
      for (const s of info.filter(i => !i.valid)) {
        addLog(`⚠ ${s.keys}k ${s.stem}: 高度 ${s.height}px，不满足 >5000，不可修改`, 'warning')
      }
    } else {
      addLog('未找到任何 NoteImage#L 面尾定义', 'warning')
    }

    await computeAllThrows()
  } catch (e) {
    addLog(`读取皮肤信息失败：${e}`, 'error')
  }
}

async function computeAllThrows() {
  if (workMode.value !== 'lazer') return

  // Dedup by stem only — column_width does not affect throw length
  // (resize always targets 32800 height; vertical scan is width-independent)
  const seenStems = new Set<string>()
  const tasks: { stem: string; keys: string }[] = []

  for (const s of skinInfo.value) {
    if (!s.valid) continue
    if (seenStems.has(s.stem)) continue
    seenStems.add(s.stem)

    const stem = s.stem
    const keyList = [...new Set(skinInfo.value.filter(x => x.stem === stem).map(x => x.keys))]
      .sort((a, b) => a - b).map(k => k + 'k').join(', ')

    let promise = throwCache.get(stem)
    if (!promise) {
      addLog(`计算 ${stem} 投长度...`, 'info')
      promise = invoke<number>('compute_lazer_throw_single', {
        folderPath: filePath.value,
        stem,
        columnWidth: s.column_width,
      })
      throwCache.set(stem, promise)
    }
    tasks.push({ stem, keys: keyList })
  }

  if (tasks.length === 0) {
    addLog('无需计算投长度', 'info')
    return
  }

  computingThrows.value = true

  // Run all computations in parallel
  const results = await Promise.allSettled(
    tasks.map(t => throwCache.get(t.stem)!)
  )

  computingThrows.value = false

  for (let i = 0; i < tasks.length; i++) {
    const t = tasks[i]
    const r = results[i]
    if (r.status === 'fulfilled') {
      const lt = r.value
      for (const x of skinInfo.value) {
        if (x.stem === t.stem) x.lazer_throw = lt
      }
      addLog(`  ✓ ${t.stem} (${t.keys}) 投长度: ${lt}`, 'success')
    } else {
      addLog(`  ✗ ${t.stem} 投长度计算失败: ${r.reason}`, 'warning')
    }
  }
  addLog('投长度计算完成', 'success')

  // 同步已勾选的 throwMap：计算前勾选的键数存的是 fallback 值，需更新为真实 lazer 值
  for (const [k] of throwMap) {
    const s = skinInfo.value.find(i => i.keys === k)
    if (s?.valid && s.lazer_throw > 0) {
      throwMap.set(k, s.lazer_throw)
    }
  }
}

function openPresetDialog(stem: string) {
  presetDialogStem.value = stem
}

function selectPreset(stem: string, preset: PresetInfo) {
  stemPresets[stem] = preset
  presetDialogStem.value = null
  addLog(`${stem} 选择预设: ${preset.name}`, 'info')
}

async function handleModify() {
  if (!canModify.value) return
  modifying.value = true

  addLog(`文件：${filePath.value}`, 'info')
  addLog(`开始修改... 模式: ${workMode.value}`, 'info')

  const entries = [...throwMap.entries()].sort((a, b) => a[0] - b[0])
  const throws: [number, number][] = entries.map(([k, v]) => [k, v])

  // Build presets: stem → preset_name
  const presetList: [string, string][] = Object.entries(stemPresets)
    .filter(([, v]) => v !== null && v !== undefined)
    .map(([stem, v]) => [stem, v!.name])

  // Build keyd_stems
  const keydStems: string[] = [...keydChecked]

  try {
    const result: { success: boolean; message: string; logs: string[] } = await invoke('convert_tail_toolbox', {
      folderPath: filePath.value,
      skinMode: 'folder',
      workMode: workMode.value,
      throws,
      presets: presetList,
      keydStems,
    })

    for (const line of result.logs) {
      const type: LogEntry['type'] = line.startsWith('  ✓') ? 'success'
        : line.includes('⚠') || line.startsWith('  ✗') ? 'warning'
          : 'info'
      addLog(line, type)
    }
    if (result.success) {
      addLog('修改完成！', 'success')
      // 重新加载投长度信息
      await loadThrowInfo()
      // 同步已勾选键数的 throwMap
      for (const [k] of throwMap) {
        const s = skinInfo.value.find(i => i.keys === k)
        if (s?.valid) {
          const def = getModeThrow(s)
          throwMap.set(k, typeof def === 'number' ? def : s.current_throw)
        }
      }
    } else {
      addLog(`修改失败: ${result.message}`, 'error')
    }
  } catch (e) {
    addLog(`修改失败：${e}`, 'error')
  } finally {
    modifying.value = false
  }
}
</script>

<style scoped>
.one-click-length {
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

/* Repair placeholder (shared by all 3 sections) */
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

/* Radio */
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

/* Key/KeyD grid (3 per row, scrollable) — SkinAdapter style */
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
  background: rgba(255, 255, 255, 0.18);
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

.ri-key {
  background: rgba(100, 255, 160, 0.15);
  color: #64ffa0;
}

.ri-keyd {
  background: rgba(255, 170, 68, 0.15);
  color: #ffaa44;
}

/* Preset table (scrollable) */
.preset-scroll {
  max-height: 320px;
  overflow-y: auto;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 6px;
  background: var(--bg-panel);
}

.preset-scroll::-webkit-scrollbar {
  width: 4px;
}

.preset-scroll::-webkit-scrollbar-track {
  background: transparent;
}

.preset-scroll::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.18);
  border-radius: 2px;
}

.preset-table {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.preset-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 6px;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: 8px;
}

.psr-stem {
  width: 120px;
  flex-shrink: 0;
  font-size: 11px;
  font-weight: 500;
  color: var(--text-primary);
  font-family: 'JetBrains Mono', monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.psr-usage {
  flex: 1;
  display: flex;
  flex-wrap: wrap;
  gap: 3px;
}

.ps-usage-item {
  font-size: 10px;
  color: var(--text-muted);
  background: var(--bg-surface);
  padding: 1px 5px;
  border-radius: 3px;
}

.psr-preset {
  width: 80px;
  flex-shrink: 0;
}

.preset-pick-btn {
  font-size: 10px;
  padding: 3px 6px;
  border-radius: 4px;
  border: 1px solid var(--border-color);
  background: var(--bg-surface);
  color: var(--text-muted);
  cursor: pointer;
  font-family: inherit;
}

.preset-pick-btn:hover {
  border-color: var(--accent-purple);
  color: var(--accent-purple);
}

.preset-selected {
  display: flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  position: relative;
}

.preset-thumb {
  width: 28px;
  height: 21px;
  object-fit: cover;
  border-radius: 3px;
  border: 1px solid var(--border-color);
}

.preset-name-sm {
  font-size: 9px;
  color: var(--text-secondary);
  max-width: 30px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.preset-clear {
  position: absolute;
  top: -4px;
  right: -4px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: none;
  background: var(--bg-surface);
  color: var(--text-muted);
  font-size: 10px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

/* Throw grid (3 per row, scrollable) */
.throw-scroll {
  max-height: 320px;
  overflow-y: auto;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 6px;
  background: var(--bg-panel);
}

.throw-scroll::-webkit-scrollbar {
  width: 4px;
}

.throw-scroll::-webkit-scrollbar-track {
  background: transparent;
}

.throw-scroll::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.18);
  border-radius: 2px;
}

.throw-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 4px;
}

.throw-card {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 10px;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  transition: all 0.15s;
  min-width: 0;
  cursor: pointer;
}

.throw-card.active {
  border-color: rgba(183, 108, 241, 0.4);
  background: rgba(183, 108, 241, 0.04);
}

.throw-card.invalid {
  opacity: 0.5;
}

.throw-card > input[type="checkbox"] {
  accent-color: var(--accent-purple);
  margin: 0;
  width: 12px;
  height: 12px;
  flex-shrink: 0;
}

.tc-keys {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  min-width: 28px;
}

.tc-input {
  flex: 1;
  min-width: 0;
  padding: 2px 4px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 3px;
  color: var(--text-primary);
  font-size: 10px;
  font-family: 'JetBrains Mono', monospace;
  outline: none;
  text-align: right;
}

.tc-input:disabled {
  color: var(--text-muted);
  opacity: 0.4;
  cursor: not-allowed;
}

.tc-input::-webkit-inner-spin-button,
.tc-input::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.tc-input {
  -moz-appearance: textfield;
}

.tc-orig {
  font-size: 9px;
  color: var(--text-muted);
  flex-shrink: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  text-align: right;
  user-select: none;
}

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 30000;
  background: rgba(4, 5, 10, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
}

.preset-modal {
  width: 640px;
  max-width: 92vw;
  max-height: 80vh;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid var(--border-color);
}

.modal-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.modal-close {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 16px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-close:hover {
  background: var(--bg-surface);
  color: var(--text-primary);
}

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 14px;
}

.preset-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 8px;
}

.preset-card {
  display: flex;
  flex-direction: column;
  background: var(--bg-elevated);
  border: 1px solid transparent;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
  overflow: hidden;
}

.preset-card:hover {
  border-color: rgba(183, 108, 241, 0.3);
  box-shadow: 0 0 16px rgba(183, 108, 241, 0.1);
}

.preset-card.active {
  border-color: var(--accent-purple);
  background: rgba(183, 108, 241, 0.06);
  box-shadow: 0 0 12px rgba(183, 108, 241, 0.2);
}

.preset-img-wrap {
  width: 100%;
  aspect-ratio: 3 / 4;
  overflow: hidden;
  background: #0a0b14;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.preset-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  image-rendering: pixelated;
}

.preset-label {
  font-size: 11px;
  color: var(--text-primary);
  font-weight: 500;
  text-align: center;
  padding: 7px 6px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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
