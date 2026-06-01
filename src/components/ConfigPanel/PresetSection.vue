<script setup lang="ts">
import { ref } from 'vue'
import { useConfig } from '../../composables/useConfig'

const { presets, loadPreset, savePreset, deletePreset, resetConfig } = useConfig()

const newPresetName = ref('')
const showSaveInput = ref(false)
const saveError = ref('')

function handleSave() {
  const name = newPresetName.value.trim()
  if (!name) {
    saveError.value = '请输入预设名称'
    return
  }
  try {
    savePreset(name)
    newPresetName.value = ''
    showSaveInput.value = false
    saveError.value = ''
  } catch (e: any) {
    saveError.value = e.message || String(e)
  }
}

function handleDelete(name: string) {
  try {
    deletePreset(name)
  } catch (e: any) {
    saveError.value = e.message || String(e)
  }
}
</script>

<template>
  <section class="config-section">
    <h3 class="section-title">
      <span class="section-icon">★</span> 预设
    </h3>

    <div class="preset-list">
      <div
        v-for="preset in presets"
        :key="preset.name"
        class="preset-item"
        @click="loadPreset(preset)"
      >
        <div class="preset-info">
          <span class="preset-name">{{ preset.name }}</span>
          <span v-if="preset.builtin" class="preset-badge">内置</span>
        </div>
        <button
          v-if="!preset.builtin"
          class="preset-delete"
          @click.stop="handleDelete(preset.name)"
          title="删除预设"
        >
          ×
        </button>
      </div>
    </div>

    <div v-if="showSaveInput" class="save-row fade-in">
      <div class="input-wrap" style="flex:1">
        <input
          v-model="newPresetName"
          type="text"
          class="text-input"
          placeholder="预设名称..."
          @keyup.enter="handleSave"
        />
      </div>
      <button class="btn btn-sm btn-primary" @click="handleSave">保存</button>
      <button class="btn btn-sm" @click="showSaveInput = false">取消</button>
    </div>
    <p v-if="saveError" class="error-text">{{ saveError }}</p>

    <div class="preset-actions">
      <button v-if="!showSaveInput" class="btn btn-sm" @click="showSaveInput = true">
        + 保存为预设
      </button>
      <button class="btn btn-sm btn-ghost" @click="resetConfig">
        ↺ 重置
      </button>
    </div>
  </section>
</template>

<style scoped>
.preset-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 160px;
  overflow-y: auto;
  margin-bottom: 8px;
}
.preset-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  background: var(--bg-elevated);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
  border: 1px solid transparent;
}
.preset-item:hover {
  background: var(--bg-surface);
  border-color: var(--accent-cyan);
}
.preset-info {
  display: flex;
  align-items: center;
  gap: 8px;
}
.preset-name {
  font-size: 13px;
  color: var(--text-primary);
  font-weight: 500;
}
.preset-badge {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 3px;
  background: var(--accent-cyan-bg);
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.preset-delete {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}
.preset-delete:hover {
  background: oklch(0.5 0.16 16 / 0.2);
  color: #ff4466;
}
.save-row {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-bottom: 8px;
}
.preset-actions {
  display: flex;
  gap: 6px;
}
.error-text {
  color: #ff4466;
  font-size: 12px;
  margin: 4px 0;
}
.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}
</style>
