<script setup lang="ts">
import { ref, nextTick, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useConfig } from '../../composables/useConfig'
import type { Preset } from '../../types/config'

const emit = defineEmits<{ close: [] }>()

const { presets, loadPreset, savePreset, deletePreset } = useConfig()

// 缩略图缓存: preset name → cropped base64 data URL
const thumbnails = ref<Map<string, string>>(new Map())
const loadingThumbs = ref(false)

// 保存预设
const showSaveInput = ref(false)
const newPresetName = ref('')
const saveError = ref('')

function handleSave() {
  const name = newPresetName.value.trim()
  if (!name) { saveError.value = '请输入预设名称'; return }
  try {
    savePreset(name)
    newPresetName.value = ''
    showSaveInput.value = false
    saveError.value = ''
    nextTick(() => renderThumbForPreset(presets.value.find(p => p.name === name)!))
  } catch (e: any) {
    saveError.value = e.message || String(e)
  }
}

function handleDelete(name: string) {
  try {
    deletePreset(name)
    thumbnails.value.delete(name)
  } catch (e: any) {
    saveError.value = e.message || String(e)
  }
}

function handleLoad(preset: Preset) {
  loadPreset(preset)
  emit('close')
}

// 渲染单个预设的缩略图（Rust 端裁剪 + 磁盘缓存）
async function renderThumbForPreset(preset: Preset) {
  try {
    const b64 = await invoke<string>('render_preset_thumbnail', {
      config: JSON.parse(JSON.stringify(preset.config)),
    })
    thumbnails.value.set(preset.name, `data:image/png;base64,${b64}`)
  } catch (e) {
    console.warn(`渲染预设 "${preset.name}" 缩略图失败:`, e)
  }
}

// 渲染所有预设缩略图
async function renderAllThumbs() {
  loadingThumbs.value = true
  for (const preset of presets.value) {
    if (!thumbnails.value.has(preset.name)) {
      await renderThumbForPreset(preset)
    }
  }
  loadingThumbs.value = false
}

onMounted(() => {
  renderAllThumbs()
})
</script>

<template>
  <div class="preset-overlay" @click.self="emit('close')">
    <div class="preset-panel">
      <div class="preset-header">
        <span class="preset-title">预设</span>
        <button class="close-btn" @click="emit('close')">
          <svg width="14" height="14" viewBox="0 0 14 14"><path d="M3 3l8 8M11 3l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
        </button>
      </div>

      <div class="preset-grid">
        <div
          v-for="preset in presets"
          :key="preset.name"
          class="preset-card"
          @click="handleLoad(preset)"
        >
          <div class="preset-thumb">
            <div v-if="loadingThumbs && !thumbnails.has(preset.name)" class="thumb-loading">
              <span class="spinner"></span>
            </div>
            <img
              v-else-if="thumbnails.get(preset.name)"
              :src="thumbnails.get(preset.name)"
              class="thumb-img"
              :alt="preset.name"
            />
            <div v-else class="thumb-empty">—</div>
          </div>
          <div class="preset-meta">
            <span class="preset-name">{{ preset.name }}</span>
            <span v-if="preset.builtin" class="preset-badge">内置</span>
            <button
              v-if="!preset.builtin"
              class="preset-delete"
              @click.stop="handleDelete(preset.name)"
              title="删除预设"
            >
              <svg width="12" height="12" viewBox="0 0 12 12"><path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/></svg>
            </button>
          </div>
        </div>
      </div>

      <div class="preset-footer">
        <div v-if="showSaveInput" class="save-row fade-in">
          <input
            v-model="newPresetName"
            type="text"
            class="save-input"
            placeholder="预设名称..."
            @keyup.enter="handleSave"
          />
          <button class="footer-btn primary" @click="handleSave">保存</button>
          <button class="footer-btn" @click="showSaveInput = false; saveError = ''">取消</button>
        </div>
        <p v-if="saveError" class="error-text">{{ saveError }}</p>
        <div class="footer-actions">
          <button v-if="!showSaveInput" class="footer-btn" @click="showSaveInput = true">
            + 保存当前为预设
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.preset-overlay {
  position: fixed;
  inset: 0;
  z-index: 20000;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  animation: fadeIn 0.2s ease-out;
}
@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }

.preset-panel {
  width: 820px;
  max-width: 92vw;
  max-height: 85vh;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6), 0 0 1px rgba(0, 212, 240, 0.3);
  animation: slideUp 0.25s ease-out;
}
@keyframes slideUp { from { opacity: 0; transform: translateY(12px) scale(0.97); } to { opacity: 1; transform: translateY(0) scale(1); } }

.preset-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}
.preset-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: 0.3px;
}
.close-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}
.close-btn:hover {
  background: var(--bg-surface);
  color: var(--text-primary);
}

/* 5列网格布局，最多显示3行，超出滚动 */
.preset-grid {
  flex: none;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 14px;
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  grid-auto-rows: max-content;
  gap: 10px;
  align-content: start;
  /* 限制3行高度，超出滚动 */
  max-height: min(600px, calc(85vh - 140px));
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
  background: var(--bg-surface);
  border-color: var(--accent-cyan);
  box-shadow: 0 0 16px rgba(0, 212, 240, 0.1);
}

.preset-thumb {
  width: 100%;
  aspect-ratio: 2 / 3;
  overflow: hidden;
  background: #0a0b14;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}
.thumb-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  image-rendering: pixelated;
}
.thumb-loading {
  display: flex;
  align-items: center;
  justify-content: center;
}
.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid transparent;
  border-top-color: var(--accent-cyan);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
.thumb-empty {
  color: var(--text-muted);
  font-size: 12px;
}

.preset-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 8px;
  min-width: 0;
}
.preset-name {
  font-size: 11px;
  color: var(--text-primary);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  min-width: 0;
}
.preset-badge {
  font-size: 8px;
  padding: 1px 5px;
  border-radius: 3px;
  background: var(--accent-cyan-bg);
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  flex-shrink: 0;
}
.preset-delete {
  width: 18px;
  height: 18px;
  border-radius: 3px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
  flex-shrink: 0;
}
.preset-delete:hover {
  background: oklch(0.5 0.16 16 / 0.2);
  color: #ff4466;
}

.preset-footer {
  border-top: 1px solid var(--border-color);
  padding: 10px 14px;
  flex-shrink: 0;
}
.save-row {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-bottom: 6px;
}
.save-input {
  flex: 1;
  padding: 5px 10px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 12px;
  font-family: inherit;
  outline: none;
}
.save-input:focus { border-color: var(--accent-cyan); }
.error-text {
  color: #ff4466;
  font-size: 11px;
  margin: 4px 0;
}
.footer-actions {
  display: flex;
  gap: 6px;
}
.footer-btn {
  padding: 5px 12px;
  font-size: 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-surface);
  color: var(--text-secondary);
  cursor: pointer;
  font-family: inherit;
  transition: all 0.15s;
  white-space: nowrap;
}
.footer-btn:hover {
  background: var(--bg-elevated);
  color: var(--text-primary);
  border-color: var(--accent-cyan);
}
.footer-btn.primary {
  background: var(--accent-cyan);
  border-color: var(--accent-cyan);
  color: #000;
  font-weight: 600;
}
.footer-btn.primary:hover {
  background: #00e8ff;
}
.footer-btn.ghost {
  background: transparent;
  border-color: transparent;
  color: var(--text-muted);
}
.footer-btn.ghost:hover {
  color: var(--text-primary);
  background: var(--bg-surface);
}
.fade-in { animation: fadeSlideIn 0.25s ease-out; }
@keyframes fadeSlideIn { from { opacity: 0; transform: translateY(-4px); } to { opacity: 1; transform: translateY(0); } }

/* scrollbar */
.preset-grid::-webkit-scrollbar { width: 4px; }
.preset-grid::-webkit-scrollbar-track { background: transparent; }
.preset-grid::-webkit-scrollbar-thumb { background: var(--border-color); border-radius: 2px; }
</style>
