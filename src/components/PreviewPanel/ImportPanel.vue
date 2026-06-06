<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { useConfig } from '../../composables/useConfig'
import { useNotification } from '../../composables/useNotification'

const emit = defineEmits<{ close: [] }>()
const { savePreset, config } = useConfig()
const { notify } = useNotification()

const loading = ref(false)
const previewUrl = ref('')
const renderPreview = ref('')
const parsedConfig = ref<any>(null)
const warnings = ref<string[]>([])
const error = ref('')
const presetName = ref('')

// 列高度同步
const colMidRef = ref<HTMLElement | null>(null)
const colLeftRef = ref<HTMLElement | null>(null)
const colRightRef = ref<HTMLElement | null>(null)
const midHeight = ref(220) // 解析前默认高度
let resizeObserver: ResizeObserver | null = null

function syncHeight() {
  if (colMidRef.value) {
    midHeight.value = colMidRef.value.offsetHeight
  }
}

onMounted(() => {
  resizeObserver = new ResizeObserver(() => syncHeight())
  if (colMidRef.value) resizeObserver.observe(colMidRef.value)
  nextTick(() => syncHeight())
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
})

// 解析完成后重新同步
watch(parsedConfig, () => {
  nextTick(() => syncHeight())
})

async function handleSelectFile() {
  if (loading.value) return

  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      multiple: false,
      filters: [
        { name: '图片文件', extensions: ['png', 'jpg', 'jpeg', 'bmp', 'webp'] }
      ]
    })
    if (!selected) return

    loading.value = true
    error.value = ''
    warnings.value = []
    parsedConfig.value = null
    renderPreview.value = ''
    presetName.value = ''

    const { invoke } = await import('@tauri-apps/api/core')

    // 获取图片顶部预览
    try {
      const b64 = await invoke<string>('get_image_preview_top', { imagePath: selected })
      previewUrl.value = `data:image/png;base64,${b64}`
    } catch { /* ignore */ }

    // 解析配置
    const [cfg, warns] = await invoke<[any, string[]]>('parse_image_to_preset', { imagePath: selected })
    parsedConfig.value = cfg
    warnings.value = warns
    presetName.value = cfg.image.filename

    // 渲染效果预览
    try {
      const b64 = await invoke<string>('render_preview', { config: cfg })
      renderPreview.value = `data:image/png;base64,${b64}`
    } catch { /* ignore */ }
  } catch (err) {
    error.value = String(err)
  } finally {
    loading.value = false
  }
}

async function handleSave() {
  if (!parsedConfig.value || !presetName.value) return
  try {
    await savePreset(presetName.value, parsedConfig.value)
    notify(`已保存为预设: ${presetName.value}`, 'success')
    Object.assign(config, parsedConfig.value)
    emit('close')
  } catch (err) {
    notify(String(err), 'error', 5000)
  }
}

function formatShape(shape: string): string {
  return ({ ball: '球皮', diamond: '菱形', rect: '矩形', gradient: '渐变' } as any)[shape] || shape
}

function colorStr(c: { r: number; g: number; b: number }) {
  return `rgb(${c.r}, ${c.g}, ${c.b})`
}
</script>

<template>
  <div class="panel-overlay" @click.self="emit('close')">
    <div class="import-panel">
      <div class="panel-header">
        <span class="panel-title">导入图片</span>
        <button class="close-btn" @click="emit('close')">
          <svg width="14" height="14" viewBox="0 0 14 14">
            <path d="M3 3l8 8M11 3l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </button>
      </div>

      <div class="panel-body">
        <!-- 左列：原图 -->
        <div ref="colLeftRef" class="col col-left" :style="{ height: midHeight + 'px' }">
          <div class="col-title">原图顶部</div>
          <div class="img-area" @click="handleSelectFile">
            <img v-if="previewUrl" :src="previewUrl" class="img-preview" />
            <div v-else class="img-placeholder">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
                <path d="M12 5v10M8 11l4 4 4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"
                  stroke-linejoin="round" />
                <path d="M4 17v2a1 1 0 001 1h14a1 1 0 001-1v-2" stroke="currentColor" stroke-width="1.5"
                  stroke-linecap="round" />
              </svg>
              <span>点击选择图片</span>
            </div>
          </div>
        </div>

        <!-- 中列：警告 + 解析信息 -->
        <div ref="colMidRef" class="col col-mid">
          <div class="col-title">解析信息</div>
          <div class="info-scroll">
            <!-- 未上传时提示 -->
            <div v-if="!parsedConfig && !loading && !error" class="info-notice">
              <p>请先上传图片以进行解析。</p>
              <p>只能解析图片尺寸和整体外观。</p>
              <p>边框，球皮检测效果不是很好，需手动调整。</p>
            </div>

            <!-- 加载中 -->
            <div v-if="loading" class="info-loading">
              <div class="spinner"></div>
              <span>解析中...</span>
            </div>

            <!-- 错误 -->
            <div v-if="error" class="info-error">{{ error }}</div>

            <!-- 警告 -->
            <div v-if="warnings.length" class="info-section warn-section">
              <div class="sec-title">⚠️ 警告</div>
              <div v-for="(w, i) in warnings" :key="i" class="warn-item">{{ w }}</div>
            </div>

            <!-- 解析结果 -->
            <template v-if="parsedConfig">
              <!-- 图片尺寸 -->
              <div class="info-section">
                <div class="sec-title">图片尺寸</div>
                <div class="kv"><span class="k">宽度</span><span class="v">{{ parsedConfig.image.width }}px</span></div>
                <div class="kv"><span class="k">高度</span><span class="v">{{ parsedConfig.image.height }}px</span></div>
              </div>

              <!-- 整体外观 -->
              <div class="info-section">
                <div class="sec-title">整体外观</div>
                <div class="kv"><span class="k">留白</span><span class="v">{{ parsedConfig.margin }}px</span></div>
                <div class="kv"><span class="k">投的长度</span><span class="v">{{ parsedConfig.throwLength }}px</span></div>
                <div class="kv">
                  <span class="k">颜色</span>
                  <span class="v"><span class="color-dot"
                      :style="{ background: colorStr(parsedConfig.globalColor) }"></span>{{ parsedConfig.globalColor.r
                      }}, {{ parsedConfig.globalColor.g }}, {{ parsedConfig.globalColor.b }}</span>
                </div>
                <div class="kv"><span class="k">透明度</span><span class="v">{{ Math.round(parsedConfig.globalOpacity / 255
                  * 100) }}%</span></div>
                <div class="kv"><span class="k">边框</span><span class="v">{{ parsedConfig.body.borderEnabled ?
                  `${parsedConfig.body.borderWidth}px` : '无' }}</span></div>
              </div>

              <!-- 顶端 -->
              <div class="info-section">
                <div class="sec-title">顶端</div>
                <div class="kv"><span class="k">形状</span><span class="v">{{ formatShape(parsedConfig.cap.shape)
                }}</span></div>
                <div class="kv"><span class="k">缩放</span><span class="v">{{ parsedConfig.cap.scale }}%</span></div>
              </div>
            </template>
          </div>
        </div>

        <!-- 右列：渲染效果 -->
        <div ref="colRightRef" class="col col-right" :style="{ height: midHeight + 'px' }">
          <div class="col-title">渲染效果</div>
          <div class="img-area">
            <img v-if="renderPreview" :src="renderPreview" class="img-preview" />
            <div v-else class="img-placeholder dim">
              <span>待解析</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 底部 -->
      <div class="panel-footer">
        <div class="footer-left">
          <label class="name-label">预设名称</label>
          <input v-model="presetName" class="name-input" type="text" placeholder="输入预设名称" />
        </div>
        <div class="footer-right">
          <button class="btn cancel" @click="emit('close')">取消</button>
          <button class="btn save" :disabled="!parsedConfig || !presetName" @click="handleSave">保存到预设</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.panel-overlay {
  position: fixed;
  inset: 0;
  z-index: 20000;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  animation: fadeIn 0.15s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }

  to {
    opacity: 1;
  }
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.98);
  }

  to {
    opacity: 1;
    transform: none;
  }
}

.import-panel {
  width: 680px;
  max-width: 94vw;
  max-height: 82vh;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5), 0 0 1px rgba(0, 212, 240, 0.3);
  animation: slideUp 0.2s ease-out;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.panel-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  width: 26px;
  height: 26px;
  border-radius: 5px;
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

/* 三列主体 */
.panel-body {
  position: relative;
  padding: 10px 14px;
  overflow: hidden;
}

.col {
  display: block;
}

.col-title {
  font-size: 11px;
  color: var(--text-muted);
  height: 18px;
  line-height: 18px;
  text-align: center;
}

.col-left {
  position: absolute;
  left: 14px;
  top: 10px;
  width: 160px;
  display: flex;
  flex-direction: column;
}

.col-mid {
  margin-left: 170px;
  margin-right: 190px;
}

.col-right {
  position: absolute;
  right: 14px;
  top: 10px;
  width: 180px;
  display: flex;
  flex-direction: column;
}

/* 图片区域 */
.img-area {
  flex: 1;
  min-height: 0;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  position: relative;
}

.col-right .img-area {
  cursor: default;
}

.img-preview {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.img-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  color: var(--text-muted);
  font-size: 11px;
}

.img-placeholder.dim {
  opacity: 0.5;
}

.img-placeholder svg {
  opacity: 0.5;
}

/* 中列信息 — 内容决定高度，左右列自适应 */
.info-scroll {
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.info-notice {
  font-size: 11px;
  color: var(--text-muted);
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 5px;
  padding: 8px 10px;
  line-height: 1.6;
}

.info-notice p {
  margin: 0;
  padding: 2px 0;
}

.info-notice p:first-child {
  color: var(--text-secondary);
}

.info-loading {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-muted);
  font-size: 12px;
  padding: 12px 0;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-color);
  border-top-color: var(--accent-purple);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.info-error {
  font-size: 11px;
  color: #ff4466;
  background: rgba(255, 68, 102, 0.08);
  border: 1px solid rgba(255, 68, 102, 0.2);
  border-radius: 5px;
  padding: 8px 10px;
}

.info-section {
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 5px;
  padding: 8px 10px;
}

.warn-section {
  background: rgba(255, 170, 0, 0.06);
  border-color: rgba(255, 170, 0, 0.25);
}

.sec-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 5px;
}

.warn-item {
  font-size: 11px;
  color: rgba(255, 170, 0, 0.85);
  padding: 1px 0 1px 10px;
  position: relative;
}

.warn-item::before {
  content: '•';
  position: absolute;
  left: 0;
}

.kv {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 2px 0;
}

.k {
  font-size: 11px;
  color: var(--text-muted);
}

.v {
  font-size: 11px;
  color: var(--text-primary);
  font-family: 'JetBrains Mono', monospace;
  display: flex;
  align-items: center;
  gap: 5px;
}

.color-dot {
  width: 10px;
  height: 10px;
  border-radius: 2px;
  border: 1px solid var(--border-color);
  display: inline-block;
}

/* 底部 */
.panel-footer {
  border-top: 1px solid var(--border-color);
  padding: 10px 14px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-shrink: 0;
}

.footer-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.name-label {
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
}

.name-input {
  flex: 1;
  min-width: 0;
  padding: 5px 8px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 5px;
  color: var(--text-primary);
  font-size: 12px;
  font-family: inherit;
  outline: none;
  transition: border-color 0.15s;
}

.name-input:focus {
  border-color: var(--accent-purple);
}

.footer-right {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.btn {
  padding: 5px 14px;
  border-radius: 5px;
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
  transition: all 0.15s;
  border: 1px solid transparent;
}

.btn.cancel {
  background: var(--bg-surface);
  border-color: var(--border-color);
  color: var(--text-secondary);
}

.btn.cancel:hover {
  background: var(--bg-elevated);
}

.btn.save {
  background: var(--accent-purple);
  color: #fff;
}

.btn.save:hover {
  background: #8b5cf6;
}

.btn.save:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>