<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useConfig } from '../../composables/useConfig'
import ExportBar from './ExportBar.vue'
import PresetPanel from './PresetPanel.vue'

const { config, previewBase64, previewLoading, resetConfig } = useConfig()

const container = ref<HTMLDivElement>()
const mainCanvas = ref<HTMLCanvasElement>()
const annoCanvas = ref<HTMLCanvasElement>()

const cw = ref(600)
const ch = ref(600)
const zoom = ref(100)
const pan = ref({ x: 0, y: 0 })

let dragging = false
let ds = { x: 0, y: 0, px: 0, py: 0 }

const bgModes = ['dark70', 'white', 'black', 'green'] as const
type BgMode = typeof bgModes[number]
const bgMode = ref<BgMode>('dark70')
const bgLabels: Record<BgMode, string> = {
  dark70: '暗化70%',
  white: '纯白',
  black: '纯黑',
  green: '绿幕',
}
const bgMenuOpen = ref(false)
function selectBg(m: BgMode) { bgMode.value = m; bgMenuOpen.value = false; paint() }
function toggleBgMenu() { bgMenuOpen.value = !bgMenuOpen.value }

// 预设面板
const showPresetPanel = ref(false)

// 重置（二次确认）
const resetConfirming = ref(false)
let resetTimer: ReturnType<typeof setTimeout> | null = null
function handleReset() {
  if (resetConfirming.value) {
    resetConfig()
    resetView()
    resetConfirming.value = false
    if (resetTimer) { clearTimeout(resetTimer); resetTimer = null }
  } else {
    resetConfirming.value = true
    resetTimer = setTimeout(() => { resetConfirming.value = false; resetTimer = null }, 2000)
  }
}

function resetView() {
  zoom.value = 100
  pan.value = { x: 0, y: Math.max(0, (bh.value * 2.5 - ch.value) / 2) }
  paint()
}

let bitmap: ImageBitmap | null = null
const bw = ref(0)
const bh = ref(0)

const src = computed(() => previewBase64.value ? `data:image/png;base64,${previewBase64.value}` : '')

watch(src, async (url) => {
  if (!url) return
  const old = bitmap
  const r = await fetch(url)
  bitmap = await createImageBitmap(await r.blob())
  bw.value = bitmap.width; bh.value = bitmap.height
  try { old?.close() } catch {}
  if (!old) { zoom.value = 100; pan.value = { x: 0, y: Math.max(0, (bh.value * 2.5 - ch.value) / 2) } }
  paint()
})

const s = computed(() => zoom.value / 40)

// ---- 绘制 ----

function paint() { drawMain(); drawAnno() }

function drawMain() {
  const c = mainCanvas.value; if (!c) return
  const ctx = c.getContext('2d')!; const dpr = window.devicePixelRatio || 1
  c.width = cw.value * dpr; c.height = ch.value * dpr
  c.style.width = cw.value + 'px'; c.style.height = ch.value + 'px'
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  ctx.clearRect(0, 0, cw.value, ch.value)

  // 外部区域
  ctx.fillStyle = '#070810'
  ctx.fillRect(0, 0, cw.value, ch.value)

  if (!bitmap) return
  const scale = s.value
  const dw = bw.value * scale; const dh = bh.value * scale
  const dx = (cw.value - dw) / 2 + pan.value.x
  const dy = (ch.value - dh) / 2 + pan.value.y

  // 图片区域背景
  const bm = bgMode.value
  if (bm === 'white') {
    ctx.fillStyle = '#ffffff'; ctx.fillRect(dx, dy, dw, dh)
  } else if (bm === 'black') {
    ctx.fillStyle = '#000000'; ctx.fillRect(dx, dy, dw, dh)
  } else if (bm === 'green') {
    ctx.fillStyle = '#00ff00'; ctx.fillRect(dx, dy, dw, dh)
  } else {
    // dark70 (default)
    drawPSGrid(ctx, dx, dy, dw, dh)
    ctx.save()
    ctx.beginPath(); ctx.rect(dx, dy, dw, dh); ctx.clip()
    ctx.fillStyle = 'rgba(0,0,0,0.7)'
    ctx.fillRect(dx, dy, dw, dh)
    ctx.restore()
  }

  if (dw > 0.2 && dh > 0.2) ctx.drawImage(bitmap, dx, dy, dw, dh)
}

// 模拟 PS 透明网格
function drawPSGrid(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number) {
  const sz = 10
  ctx.save()
  ctx.beginPath(); ctx.rect(x, y, w, h); ctx.clip()
  for (let row = 0; row < Math.ceil(h / sz); row++) {
    for (let col = 0; col < Math.ceil(w / sz); col++) {
      ctx.fillStyle = (row + col) % 2 === 0 ? '#b0b0b0' : '#6a6a6a'
      ctx.fillRect(x + col * sz, y + row * sz, sz, sz)
    }
  }
  ctx.restore()
}

function drawAnno() {
  const c = annoCanvas.value; if (!c || !bitmap) return
  const ctx = c.getContext('2d')!; const dpr = window.devicePixelRatio || 1
  c.width = cw.value * dpr; c.height = ch.value * dpr
  c.style.width = cw.value + 'px'; c.style.height = ch.value + 'px'
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  ctx.clearRect(0, 0, cw.value, ch.value)

  const scale = s.value
  const dw = bw.value * scale; const dh = bh.value * scale
  const dx = (cw.value - dw) / 2 + pan.value.x
  const dy = (ch.value - dh) / 2 + pan.value.y
  // 直接按位图像素 × 缩放计算，不依赖全图高度
  const throwY0 = dy
  const throwY1 = dy + config.throwLength * s.value
  const capHpx = config.cap.scale * (config.image.width - config.margin * 2) / 200
  const capEndY = dy + (config.throwLength + capHpx) * s.value

  ctx.font = '10px "JetBrains Mono", monospace'
  ctx.textBaseline = 'middle'

  const lx = Math.max(2, dx - 42) // 左侧标注线 x 位置

  // === 左侧竖线 1：投的长度（透明区域）===
  if (config.throwLength > 0 && throwY1 > throwY0 + 4) {
    const x1 = lx
    ctx.strokeStyle = 'rgba(255,45,149,0.6)'; ctx.setLineDash([]); ctx.lineWidth = 1.5
    ctx.beginPath(); ctx.moveTo(x1, throwY0); ctx.lineTo(x1, throwY1); ctx.stroke()
    // 端点
    drawDot(ctx, x1, throwY0, '#ff2d95')
    drawDot(ctx, x1, throwY1, '#ff2d95')
    ctx.textAlign = 'right'
    vlabel(ctx, `投${config.throwLength}px`, x1 - 6, (throwY0 + throwY1) / 2, '#ff2d95')
    ctx.textAlign = 'start'
  }

  // === 左侧竖线 2：Cap 区域（第一个可见像素 → Cap 结束）===
  const capH = capEndY - throwY1
  if (capH > 4 && capHpx > 0) {
    const x2 = lx + 18
    ctx.strokeStyle = 'rgba(0,212,240,0.5)'; ctx.setLineDash([]); ctx.lineWidth = 1.5
    ctx.beginPath(); ctx.moveTo(x2, throwY1); ctx.lineTo(x2, capEndY); ctx.stroke()
    drawDot(ctx, x2, throwY1, '#00d4f0')
    drawDot(ctx, x2, capEndY, '#00d4f0')
    ctx.textAlign = 'right'
    vlabel(ctx, `Cap:${Math.round(capHpx)}px`, x2 - 5, (throwY1 + capEndY) / 2, '#00d4f0')
    ctx.textAlign = 'start'
  }

  // === 底部重复提示 ===
  if (dh > 0) {
    const botY = dy + dh
    ctx.strokeStyle = 'rgba(255,255,255,0.2)'; ctx.setLineDash([2,6]); ctx.lineWidth = 1
    ctx.beginPath(); ctx.moveTo(Math.max(0, dx), botY); ctx.lineTo(Math.min(cw.value, dx + dw), botY); ctx.stroke()
    ctx.setLineDash([])
    ctx.fillStyle = 'rgba(255,255,255,0.45)'; ctx.font = '10px sans-serif'
    ctx.textAlign = 'center'; ctx.textBaseline = 'top'
    ctx.fillText(`↓ 以下同，导出完整高度 ${config.image.height}px`, (dx + Math.min(cw.value, dx + dw)) / 2, botY + 6)
    ctx.textAlign = 'start'; ctx.textBaseline = 'middle'
  }

  // 右下角：分辨率 + 缩放
  ctx.font = '12px "JetBrains Mono", monospace'
  const t = `分辨率：${config.image.width}×${config.image.height}  缩放：${zoom.value}%`
  const m = ctx.measureText(t)
  const bx = cw.value - m.width - 20; const by = ch.value - 20
  ctx.fillStyle = 'rgba(0,0,0,0.5)'; ctx.fillRect(bx, by, m.width + 14, 18)
  ctx.fillStyle = 'rgba(255,255,255,0.4)'; ctx.textBaseline = 'bottom'
  ctx.fillText(t, bx + 7, by + 16)
}

function drawDot(ctx: CanvasRenderingContext2D, x: number, y: number, color: string) {
  ctx.fillStyle = color; ctx.beginPath(); ctx.arc(x, y, 3, 0, Math.PI * 2); ctx.fill()
}

function vlabel(ctx: CanvasRenderingContext2D, t: string, x: number, y: number, color: string) {
  const m = ctx.measureText(t)
  ctx.fillStyle = 'rgba(0,0,0,0.65)'; ctx.fillRect(x - m.width - 4, y - 7, m.width + 8, 14)
  ctx.fillStyle = color; ctx.fillText(t, x - 2, y)
}

// ---- 交互 ----

function onMD(e: MouseEvent) { dragging = true; ds = { x: e.clientX, y: e.clientY, px: pan.value.x, py: pan.value.y } }
function onMM(e: MouseEvent) { if (!dragging) return; pan.value = { x: ds.px + (e.clientX - ds.x), y: ds.py + (e.clientY - ds.y) }; paint() }
function onMU() { dragging = false }
function onWh(e: WheelEvent) { e.preventDefault(); zoom.value = Math.max(1, Math.min(400, zoom.value + (e.deltaY > 0 ? -5 : 5))) }

let ro: ResizeObserver | null = null
onMounted(() => {
  if (container.value) {
    cw.value = container.value.clientWidth; ch.value = container.value.clientHeight
    ro = new ResizeObserver(() => { if (container.value) { cw.value = container.value.clientWidth; ch.value = container.value.clientHeight; nextTick(paint) } })
    ro.observe(container.value)
  }
})
onUnmounted(() => ro?.disconnect())
watch([s, cw, ch], paint, { flush: 'post' })
</script>

<template>
  <div class="preview-panel">
    <div class="topbar">
      <div class="topbar-left">
        <button class="preset-trigger" @click="showPresetPanel = true">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect x="1" y="1" width="5" height="5" rx="1" stroke="currentColor" stroke-width="1.1"/>
            <rect x="8" y="1" width="5" height="5" rx="1" stroke="currentColor" stroke-width="1.1"/>
            <rect x="1" y="8" width="5" height="5" rx="1" stroke="currentColor" stroke-width="1.1"/>
            <rect x="8" y="8" width="5" height="5" rx="1" stroke="currentColor" stroke-width="1.1"/>
          </svg>
          预设
        </button>
        <button :class="['reset-trigger', { confirming: resetConfirming }]" @click="handleReset">
          <svg width="13" height="13" viewBox="0 0 14 14" fill="none">
            <path d="M2 7a5 5 0 0 1 9.33-2.5M12 7a5 5 0 0 1-9.33 2.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            <path d="M11.5 1.5v3h-3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M2.5 12.5v-3h3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          {{ resetConfirming ? '确认重置？' : '重置' }}
        </button>
      </div>
      <span class="st"><span :class="['dot', previewLoading ? 'ld' : 'ok']"></span>{{ previewLoading ? '渲染中' : '预览' }}</span>
    </div>

    <div ref="container" class="cv" @mousedown="onMD" @mousemove="onMM" @mouseup="onMU" @mouseleave="onMU" @wheel="onWh">
      <canvas ref="mainCanvas" class="c1"></canvas>
      <canvas ref="annoCanvas" class="c2"></canvas>
      <div class="bg-wrap" @mousedown.stop @mouseup.stop>
        <button class="bg-btn" @click="resetView">恢复视角</button>
        <button class="bg-btn" @click="toggleBgMenu">切换背景</button>
        <div v-if="bgMenuOpen" class="bg-drop">
          <button v-for="m in bgModes" :key="m" :class="['bg-opt', { on: bgMode === m }]" @click="selectBg(m)">{{ bgLabels[m] }}</button>
        </div>
      </div>
      <div v-if="!previewBase64 && !previewLoading" class="ph">
        <svg width="40" height="40" viewBox="0 0 48 48" fill="none"><rect x="12" y="8" width="24" height="34" rx="3" stroke="#5a5e7a" stroke-width="1"/><path d="M12 16 A12 6 0 0 1 36 16" stroke="#5a5e7a" stroke-width="1" fill="#5a5e7a" fill-opacity="0.1"/></svg>
        <span>调整参数以生成预览</span>
      </div>
    </div>
    <ExportBar />

    <!-- 预设面板弹出层 -->
    <PresetPanel v-if="showPresetPanel" @close="showPresetPanel = false" />
  </div>
</template>

<style scoped>
.preview-panel { flex:1; display:flex; flex-direction:column; height:100vh; overflow:hidden; background:var(--bg-base) }
.topbar { display:flex; align-items:center; justify-content:space-between; padding:0 18px; height:44px; background:var(--bg-panel); border-bottom:1px solid var(--border-color); flex-shrink:0 }
.topbar-left { display:flex; align-items:center; gap:6px }
.preset-trigger {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-secondary);
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
  transition: all 0.15s;
}
.preset-trigger:hover {
  background: var(--bg-elevated);
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
}
.reset-trigger {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 5px 10px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-muted);
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
  transition: all 0.15s;
}
.reset-trigger:hover {
  background: var(--bg-elevated);
  border-color: var(--text-muted);
  color: var(--text-secondary);
}
.reset-trigger.confirming {
  border-color: #ff4466;
  color: #ff4466;
  background: oklch(0.35 0.08 16 / 0.3);
}
.reset-trigger.confirming:hover {
  background: oklch(0.4 0.1 16 / 0.4);
}
.zoom-row { display:flex; align-items:center; gap:4px }
.zi { width:52px; padding:3px 4px; text-align:center; background:var(--bg-input); border:1px solid var(--border-color); border-radius:4px; color:var(--text-primary); font-size:12px; font-family:'JetBrains Mono',monospace; outline:none }
.zi:focus { border-color:var(--accent-cyan) }
.zu { font-size:11px; color:var(--text-muted) }
.st { display:flex; align-items:center; gap:6px; font-size:11px; color:var(--text-muted) }
.dot { width:6px; height:6px; border-radius:50% }
.dot.ld { background:var(--accent-cyan); animation:pulse 1.2s ease-in-out infinite }
.dot.ok { background:#44ee88 }
@keyframes pulse { 0%,100%{opacity:1} 50%{opacity:.3} }
.cv { flex:1; position:relative; overflow:hidden; background:#070810; cursor:grab }
.cv:active { cursor:grabbing }
.c1,.c2 { position:absolute; top:0; left:0; width:100%; height:100% }
.c2 { pointer-events:none }
.ph { position:absolute; inset:0; display:flex; flex-direction:column; align-items:center; justify-content:center; gap:8px; color:var(--text-muted); font-size:13px; z-index:2 }
.ph svg { opacity:.4 }
.bg-wrap { position:absolute; top:8px; right:8px; z-index:10; display:flex; gap:4px }
.bg-btn { padding:5px 10px; background:rgba(0,0,0,0.55); border:1px solid rgba(255,255,255,0.15); border-radius:5px; color:rgba(255,255,255,0.65); font-size:12px; cursor:pointer; font-family:inherit; white-space:nowrap; line-height:1 }
.bg-btn:hover { background:rgba(0,0,0,0.75); border-color:var(--accent-cyan); color:var(--accent-cyan) }
.bg-drop { position:absolute; top:100%; right:0; margin-top:4px; background:rgba(15,17,29,0.95); border:1px solid var(--border-color); border-radius:5px; overflow:hidden; min-width:100px }
.bg-opt { display:block; width:100%; padding:6px 14px; background:transparent; border:none; color:var(--text-secondary); font-size:12px; cursor:pointer; text-align:left; font-family:inherit; white-space:nowrap }
.bg-opt:hover { background:var(--bg-surface); color:var(--text-primary) }
.bg-opt.on { color:var(--accent-cyan); background:var(--accent-cyan-bg) }
</style>
