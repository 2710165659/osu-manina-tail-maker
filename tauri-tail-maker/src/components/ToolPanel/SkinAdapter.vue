<template>
  <div class="skin-adapter">
    <div class="desc-card">
      <p class="desc-text">修复投皮转换为 lazer 后，面尾拉伸或 KeyD 等图片拉伸的问题。此操作会将原始文件备份到皮肤根目录下的 _backup 文件夹。</p>
    </div>

    <div class="config-group">
      <div class="field">
        <label class="field-label">皮肤文件夹</label>
        <div class="path-group">
          <div class="path-display">
            <svg class="path-icon" width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M1.5 7.5v3.5a1 1 0 001 1h9a1 1 0 001-1V7.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
              <path d="M7 1.5v6M4.5 5L7 7.5 9.5 5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            <span class="path-text" :class="{ placeholder: !filePath }">{{ filePath || '请选择皮肤所在文件夹' }}</span>
          </div>
          <button class="browse-btn" @click="handleBrowse"><span>浏览</span></button>
        </div>
      </div>

      <div class="field">
        <label class="field-label" v-if="filePath && repairStems.length > 0">要修复的图片 ({{ checkedCount }}/{{ repairStems.length }})</label>
        <template v-if="filePath && repairStems.length > 0">
          <span class="field-hint">共 {{ repairStems.length }} 张待修复图片</span>
          <div class="repair-scroll">
            <div class="repair-grid">
              <label v-for="item in repairStems" :key="item.stem" :class="['repair-item', { active: item.checked }]">
                <input type="checkbox" v-model="item.checked" />
                <span class="ri-stem">{{ item.stem }}</span>
                <span :class="['ri-tag', item.kind === 'tail' ? 'ri-tail' : item.kind === 'keyd' ? 'ri-keyd' : 'ri-key']">{{ item.kind === 'tail' ? '面尾' : item.kind === 'keyd' ? 'KeyD' : 'Key' }}</span>
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

    <LogPanel :logs="logs" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useToolLogger } from '../../composables/useToolLogger'
import LogPanel from '../shared/LogPanel.vue'

const filePath = ref('')
const loadingInfo = ref(false)

interface RepairStemItem { stem: string; kind: 'tail' | 'key' | 'keyd'; checked: boolean }
const repairStems = ref<RepairStemItem[]>([])

const checkedCount = computed(() => repairStems.value.filter(s => s.checked).length)
const repairing = ref(false)

const seenStems = new Set<string>()

const { logs, push, clear } = useToolLogger({
  target: ['repair', 'frontend'],
  onError: () => { repairing.value = false },
  onData: (_target, data) => {
    const d = data as { done?: boolean; kind?: string; items?: Record<string, unknown>[] }
    if (d.done) {
      loadingInfo.value = false
      repairing.value = false
      return
    }
    if (d.kind === 'tails' && d.items) {
      for (const t of d.items as { stem: string }[]) {
        if (!seenStems.has(t.stem)) {
          seenStems.add(t.stem)
          repairStems.value.push({ stem: t.stem, kind: 'tail', checked: true })
        }
      }
    }
    if (d.kind === 'keyds' && d.items) {
      for (const kd of d.items as { stem: string; as_key: number[]; as_keyd: number[] }[]) {
        if (kd.as_key.length > 0 && !seenStems.has(kd.stem)) {
          seenStems.add(kd.stem)
          repairStems.value.push({ stem: kd.stem, kind: 'key', checked: true })
        }
        if (kd.as_keyd.length > 0 && !seenStems.has(kd.stem)) {
          seenStems.add(kd.stem)
          repairStems.value.push({ stem: kd.stem, kind: 'keyd', checked: true })
        }
      }
    }
  },
})

// 组件卸载时取消后端任务
onUnmounted(() => {
  if (repairing.value) {
    invoke('cancel_repair_skin_adapter')
    repairing.value = false
  }
})

const canRepair = computed(() => filePath.value && !repairing.value && checkedCount.value > 0)

async function handleBrowse() {
  const { open } = await import('@tauri-apps/plugin-dialog')
  try {
    const selected = await open({ multiple: false, directory: true })
    if (selected) {
      const path = Array.isArray(selected) ? selected[0] : selected
      const valid = await invoke('check_skin_ini', { folderPath: path })
      if (!valid) {
        push('✗ 所选文件夹不包含 skin.ini，请选择有效的皮肤目录', 'error')
        return
      }
      filePath.value = path
      push(`已选择：${path}`, 'info')
      loadRepairInfo()
    }
  } catch (e) { push(`文件选择失败：${e}`, 'error') }
}

function loadRepairInfo() {
  loadingInfo.value = true
  repairStems.value = []
  seenStems.clear()

  // 同步 fire-and-forget：后端通过 app:event 流式推送扫描结果
  invoke('scan_repair_info', {
    folderPath: filePath.value,
  }).catch((e) => {
    push(`扫描启动失败：${e}`, 'error')
    loadingInfo.value = false
  })
}

function handleRepair() {
  if (!canRepair.value) return
  repairing.value = true
  clear()

  // 同步 fire-and-forget：后端通过 app:event 流式推送进度
  invoke('repair_skin_adapter', {
    folderPath: filePath.value,
  }).catch((e) => {
    push(`修复启动失败：${e}`, 'error')
    repairing.value = false
  })
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
