<template>
  <div class="export-bar">
    <!-- 客户端切换提示 -->
    <Transition name="tip-fade">
      <div v-if="clientTip" class="client-tip-banner">
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <circle cx="6" cy="6" r="5" stroke="currentColor" stroke-width="1" />
          <path
            d="M5.4 5.4a.6.6 0 0 1 1.2 0c0 .66-.6.9-.6 1.5v.3h.6v-.3c0-.66.6-.9.6-1.5a1.2 1.2 0 1 0-2.4 0h.6ZM5.4 9h1.2v-1.2h-1.2z"
            fill="currentColor" />
        </svg>
        {{ clientTip }}
      </div>
    </Transition>

    <!-- 二次确认弹窗 -->
    <Transition name="confirm-fade">
      <div v-if="showConfirm" class="confirm-overlay" @mousedown.self="cancelExport">
        <div class="confirm-dialog">
          <div class="confirm-icon">⚠</div>
          <p class="confirm-message">{{ confirmMessage }}</p>
          <div class="confirm-actions">
            <button class="btn btn-sm" @click="cancelExport">取消</button>
            <button class="btn btn-sm btn-primary" @click="confirmExport">继续导出</button>
          </div>
        </div>
      </div>
    </Transition>

    <div class="export-info">
      <span class="export-filename">{{ displayFilename }}</span>
      <span class="export-dims">{{ config.image.width }}×{{ config.image.height }}</span>
    </div>
    <div class="export-actions">
      <!-- 客户端模式切换 -->
      <div class="client-mode-switch">
        <button class="client-btn" :class="{ active: clientMode === 'lazer' }"
          @click="clientMode = 'lazer'">lazer</button>
        <button class="client-btn" :class="{ active: clientMode === 'stable' }"
          @click="clientMode = 'stable'">stable</button>
      </div>
      <!-- 2x 选项 -->
      <label class="scale-option" title="勾选后文件名添加 @2x 后缀（不进行实际放大）">
        <input type="checkbox" v-model="is2x" class="scale-checkbox" />
        <span class="scale-label">2x</span>
      </label>
      <span v-if="exportStatus === 'success'" class="export-msg success">✓ {{ exportMessage }}</span>
      <span v-if="exportStatus === 'error'" class="export-msg error">✗ {{ exportMessage }}</span>
      <button class="btn btn-primary btn-export" :disabled="exporting" @click="handleExport">
        <span v-if="exporting" class="spinner"></span>
        <span v-else>⬇</span>
        导出 PNG
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useConfig } from '../../composables/useConfig'

const { config, exportImage, setImageProp } = useConfig()

const exporting = ref(false)
const exportStatus = ref<'idle' | 'success' | 'error'>('idle')
const exportMessage = ref('')

// 客户端模式：lazer / stable
const clientMode = ref<'lazer' | 'stable'>('lazer')
// 客户端切换提示
const clientTip = ref('')

// 2x 选项
const is2x = ref(false)

// 初始化：检测 filename 是否已包含 @2x
if (config.image.filename.endsWith('@2x')) {
  is2x.value = true
}

// lazer 推荐高度
const LAZER_HEIGHT = 32800
// stable 推荐高度
const STABLE_HEIGHT = 32767

// 切换客户端模式时显示提示
watch(clientMode, (mode) => {
  if (mode === 'lazer') {
    clientTip.value = '在使用 lazer 客户端时，为避免图片变形，图片宽度需设置为 ColumnWidth 值的 1.6 倍，高度固定为 32800。'
  } else {
    // stable：仅在高度不符合时提示
    if (config.image.height !== STABLE_HEIGHT) {
      clientTip.value = `Stable 客户端建议图片高度为 ${STABLE_HEIGHT}。`
    } else {
      clientTip.value = ''
    }
  }
  // 3 秒后自动隐藏提示
  setTimeout(() => { clientTip.value = '' }, 5000)
}, { immediate: false })

// 切换 2x 时同步修改 filename
watch(is2x, (val) => {
  let name = config.image.filename
  if (val) {
    // 添加 @2x（如果还没有）
    if (!name.endsWith('@2x')) {
      setImageProp('filename', name + '@2x')
    }
  } else {
    // 移除 @2x
    if (name.endsWith('@2x')) {
      setImageProp('filename', name.slice(0, -3))
    }
  }
})

// 监听外部 filename 变化，同步 is2x 状态
watch(() => config.image.filename, (name) => {
  is2x.value = name.endsWith('@2x')
})

// 检查高度是否符合当前客户端规范
function isHeightValid(): boolean {
  if (clientMode.value === 'lazer') {
    return config.image.height === LAZER_HEIGHT
  } else {
    return config.image.height === STABLE_HEIGHT
  }
}

// 获取当前客户端推荐高度
function getRecommendedHeight(): number {
  return clientMode.value === 'lazer' ? LAZER_HEIGHT : STABLE_HEIGHT
}

// 显示的文件名（带 @2x 后缀）
const displayFilename = computed(() => {
  return config.image.filename + '.png'
})

// 确认弹窗状态
const showConfirm = ref(false)
const confirmMessage = ref('')

async function handleExport() {
  // 高度校验
  if (!isHeightValid()) {
    confirmMessage.value = `当前图片高度 ${config.image.height}，${clientMode.value === 'lazer' ? 'Lazer' : 'Stable'} 推荐 ${getRecommendedHeight()}，是否继续？`
    showConfirm.value = true
    return
  }
  await doExport()
}

function confirmExport() {
  showConfirm.value = false
  doExport()
}

function cancelExport() {
  showConfirm.value = false
}

async function doExport() {
  exporting.value = true
  exportStatus.value = 'idle'
  try {
    // 使用 Tauri 原生文件保存对话框
    const { save } = await import('@tauri-apps/plugin-dialog')
    const filePath = await save({
      defaultPath: displayFilename.value,
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

<style scoped>
.export-bar {
  position: relative;
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
  gap: 10px;
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
  to {
    transform: rotate(360deg);
  }
}

/* 客户端模式切换 */
.client-mode-switch {
  display: flex;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.client-btn {
  padding: 5px 10px;
  font-size: 11px;
  font-weight: 600;
  font-family: 'JetBrains Mono', 'Cascadia Code', monospace;
  background: var(--bg-elevated);
  color: var(--text-muted);
  border: none;
  cursor: pointer;
  transition: all 0.2s ease;
  text-transform: lowercase;
}

.client-btn:first-child {
  border-right: 1px solid var(--border-color);
}

.client-btn:hover {
  color: var(--text-secondary);
  background: var(--bg-surface);
}

.client-btn.active {
  background: var(--accent-purple);
  color: #fff;
  box-shadow: 0 0 8px rgba(183, 108, 241, 0.3);
}

/* 2x 选项 */
.scale-option {
  display: flex;
  align-items: center;
  gap: 5px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  transition: background 0.2s ease;
}

.scale-option:hover {
  background: var(--bg-surface);
}

.scale-checkbox {
  width: 14px;
  height: 14px;
  accent-color: var(--accent-purple);
  cursor: pointer;
}

.scale-label {
  font-family: 'JetBrains Mono', 'Cascadia Code', monospace;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  user-select: none;
}

/* 客户端切换提示 */
.client-tip-banner {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-bottom: 8px;
  padding: 8px 16px;
  background: rgba(15, 17, 29, 0.97);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-primary);
  white-space: nowrap;
  z-index: 100;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  gap: 8px;
}

.client-tip-banner svg {
  color: var(--accent-purple);
  flex-shrink: 0;
}

.tip-fade-enter-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.tip-fade-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.tip-fade-enter-from {
  opacity: 0;
  transform: translateX(-50%) translateY(4px);
}

.tip-fade-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(4px);
}

/* 二次确认弹窗 */
.confirm-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.confirm-dialog {
  background: var(--bg-elevated);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 24px 28px;
  max-width: 420px;
  text-align: center;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.confirm-icon {
  font-size: 28px;
  margin-bottom: 12px;
}

.confirm-message {
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-primary);
  margin-bottom: 20px;
}

.confirm-actions {
  display: flex;
  justify-content: center;
  gap: 10px;
}

.confirm-fade-enter-active {
  transition: opacity 0.2s ease;
}

.confirm-fade-leave-active {
  transition: opacity 0.15s ease;
}

.confirm-fade-enter-from,
.confirm-fade-leave-to {
  opacity: 0;
}
</style>
