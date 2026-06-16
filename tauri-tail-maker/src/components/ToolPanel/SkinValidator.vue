<template>
  <div class="skin-validator">
    <div class="desc-card">
      <p class="desc-text">校验皮肤文件的完整性，检查 NoteImage#L（面尾）、KeyImage#D（KeyD）、KeyImage#（按键）等引用图片是否缺失。</p>
    </div>

    <div class="config-group">
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
            <span class="path-text" :class="{ placeholder: !folderPath }">
              {{ folderPath || '请选择皮肤所在文件夹' }}
            </span>
          </div>
          <button class="browse-btn" @click="handleBrowse">
            <span>浏览</span>
          </button>
        </div>
      </div>

      <div class="field">
        <button class="btn btn-primary btn-full" @click="handleValidate" :disabled="!canValidate">
          <span>{{ validating ? '校验中...' : '开始校验' }}</span>
        </button>
      </div>
    </div>

    <LogPanel :logs="logs" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useToolLogger } from '../../composables/useToolLogger'
import LogPanel from '../shared/LogPanel.vue'

const folderPath = ref('')
const validating = ref(false)

const { logs, push } = useToolLogger({ target: ['validator', 'frontend'] })

const canValidate = computed(() => !!folderPath.value && !validating.value)

async function handleBrowse() {
  const { open } = await import('@tauri-apps/plugin-dialog')
  try {
    const selected = await open({ multiple: false, directory: true })
    if (selected) {
      folderPath.value = Array.isArray(selected) ? selected[0] : selected
      push(`已选择：${folderPath.value}`, 'info')
    }
  } catch (e) {
    push(`文件夹选择失败：${e}`, 'error')
  }
}

async function handleValidate() {
  if (!canValidate.value) return

  validating.value = true
  push('开始校验...', 'info')

  try {
    await invoke('validate_skin_files_cmd', {
      folderPath: folderPath.value,
    })
    push('校验完成！', 'success')
  } catch (e) {
    push(`校验失败：${e}`, 'error')
  } finally {
    validating.value = false
  }
}
</script>

<style scoped>
.skin-validator {
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
