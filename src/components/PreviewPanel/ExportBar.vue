<script setup lang="ts">
import { ref } from 'vue'
import { useConfig } from '../../composables/useConfig'

const { config, exportImage } = useConfig()

const exporting = ref(false)
const exportStatus = ref<'idle' | 'success' | 'error'>('idle')
const exportMessage = ref('')

async function handleExport() {
  exporting.value = true
  exportStatus.value = 'idle'
  try {
    // 使用 Tauri 原生文件保存对话框
    const { save } = await import('@tauri-apps/plugin-dialog')
    const filePath = await save({
      defaultPath: `${config.image.filename}.png`,
      filters: [{ name: 'PNG Image', extensions: ['png'] }],
    })
    if (!filePath) {
      exporting.value = false
      return
    }
    await exportImage(filePath)
    exportStatus.value = 'success'
    exportMessage.value = `已导出到 ${filePath}`
  } catch (e: any) {
    exportStatus.value = 'error'
    exportMessage.value = String(e)
  } finally {
    exporting.value = false
    setTimeout(() => { exportStatus.value = 'idle' }, 4000)
  }
}
</script>

<template>
  <div class="export-bar">
    <div class="export-info">
      <span class="export-filename">{{ config.image.filename }}.png</span>
      <span class="export-dims">{{ config.image.width }}×{{ config.image.height }}</span>
    </div>
    <div class="export-actions">
      <span v-if="exportStatus === 'success'" class="export-msg success">✓ {{ exportMessage }}</span>
      <span v-if="exportStatus === 'error'" class="export-msg error">✗ {{ exportMessage }}</span>
      <button
        class="btn btn-primary btn-export"
        :disabled="exporting"
        @click="handleExport"
      >
        <span v-if="exporting" class="spinner"></span>
        <span v-else>⬇</span>
        导出 PNG
      </button>
    </div>
  </div>
</template>

<style scoped>
.export-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  background: var(--bg-panel);
  border-top: 1px solid var(--border-color);
  flex-shrink: 0;
  height: 56px;
}
.export-info {
  display: flex;
  align-items: center;
  gap: 12px;
}
.export-filename {
  font-family: 'JetBrains Mono', 'Cascadia Code', monospace;
  font-size: 13px;
  color: var(--text-primary);
  font-weight: 600;
}
.export-dims {
  font-size: 11px;
  color: var(--text-muted);
  font-family: 'JetBrains Mono', 'Cascadia Code', monospace;
}
.export-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}
.export-msg {
  font-size: 12px;
}
.export-msg.success {
  color: #44ee88;
}
.export-msg.error {
  color: #ff4466;
}
.btn-export {
  padding: 8px 20px;
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.3px;
  display: flex;
  align-items: center;
  gap: 6px;
}
.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid transparent;
  border-top-color: currentColor;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
