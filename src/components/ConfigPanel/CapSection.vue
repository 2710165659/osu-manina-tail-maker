<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useConfig } from '../../composables/useConfig'
import { CAP_SHAPE_LABELS, CAP_SHAPE_ORDER, rgbaToHex, hexToRgba } from '../../types/config'

const { config, setCapProp } = useConfig()
const shapes = CAP_SHAPE_ORDER

const throwMax = computed(() => Math.max(0, config.image.height - 1))
const throwModel = computed({
  get: () => config.throwLength,
  set: (v: number) => (config.throwLength = Math.max(0, Math.min(throwMax.value, v))),
})
const capScaleModel = ref(config.cap.scale)
const capScaleFree = ref(false)
watch(() => config.cap.scale, (v) => { if (!capScaleFree.value) capScaleModel.value = v })
function applyScale(v: number) {
  capScaleFree.value = v > 500
  setCapProp('scale', Math.max(1, v))
}

// 颜色 hex 可编辑
const capHex = ref(rgbaToHex(config.cap.color))
watch(() => config.cap.color, (c) => { capHex.value = rgbaToHex(c) })
function applyCapHex(v: string) {
  const clean = v.replace('#', '').trim()
  if (/^[0-9a-fA-F]{6}$/.test(clean)) {
    config.cap.color = hexToRgba('#' + clean, config.cap.color.a)
  }
}

const opacityModel = computed({
  get: () => config.cap.opacity,
  set: (v: number) => setCapProp('opacity', Math.max(0, Math.min(255, v))),
})
const opacityPct = computed(() => Math.round((config.cap.opacity / 255) * 100))

const tipText = ref('')
function showTip(text: string) { tipText.value = text }
function hideTip() { tipText.value = '' }
</script>

<template>
  <section class="config-section cap-section">
    <h3 class="section-title">
      <svg width="14" height="14" viewBox="0 0 14 14" class="section-icon-svg">
        <polygon points="7,1 13,7 7,13 1,7" fill="none" stroke="currentColor" stroke-width="1.2"/>
        <circle cx="7" cy="5" r="1" fill="currentColor"/>
      </svg>
      投皮头
    </h3>

    <div class="field">
      <label class="field-label">投的长度 <span class="unit">px</span></label>
      <div class="input-wrap">
        <input v-model.number="throwModel" type="number" :min="0" :max="throwMax" class="num-input" />
      </div>
    </div>

    <div class="field">
      <label class="field-label">顶端形状</label>
      <div class="shape-selector">
        <button v-for="s in shapes" :key="s" :class="['shape-btn', { active: config.cap.shape === s }]" @click="setCapProp('shape', s)">
          <svg class="shape-preview-svg" width="20" height="16" viewBox="0 0 20 16" fill="none">
            <template v-if="s === 'ball'">
              <ellipse cx="10" cy="0" rx="6" ry="8" stroke="currentColor" stroke-width="1.2" fill="currentColor" fill-opacity="0.3"/>
            </template>
            <template v-else-if="s === 'diamond'">
              <polygon points="10,0 16,8 4,8" stroke="currentColor" stroke-width="1.2" fill="currentColor" fill-opacity="0.3"/>
            </template>
            <template v-else-if="s === 'rect'">
              <rect x="4" y="0" width="12" height="16" rx="1" fill="currentColor" opacity="0.3"/>
            </template>
            <template v-else>
              <rect x="4" y="0" width="12" height="16" rx="1" fill="url(#gf)" opacity="0.4"/>
              <defs><linearGradient id="gf" x1="0" y1="0" x2="0" y2="1"><stop offset="0%" stop-color="currentColor" stop-opacity="0"/><stop offset="100%" stop-color="currentColor" stop-opacity="1"/></linearGradient></defs>
            </template>
          </svg>
          {{ CAP_SHAPE_LABELS[s] }}
        </button>
      </div>
    </div>

    <div class="field">
      <div class="label-wrap">
        <label class="field-label">
          顶端缩放
          <span class="help-icon" @mouseenter="showTip('值越小圆越扁')" @mouseleave="hideTip">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><circle cx="6" cy="6" r="5" stroke="currentColor" stroke-width="1"/><path d="M5.4 5.4a.6.6 0 0 1 1.2 0c0 .66-.6.9-.6 1.5v.3h.6v-.3c0-.66.6-.9.6-1.5a1.2 1.2 0 1 0-2.4 0h.6ZM5.4 9h1.2v-1.2h-1.2z" fill="currentColor"/></svg>
          </span>
        </label>
        <div v-if="tipText" class="help-tip-banner">{{ tipText }}</div>
      </div>
      <div class="scale-row">
        <input v-model.number="capScaleModel" type="range" min="1" max="500" class="slider" @input="applyScale(capScaleModel)" />
        <input
          v-model.number="capScaleModel"
          type="number"
          :min="1"
          class="num-input scale-num"
          @change="applyScale(capScaleModel)"
        />
      </div>
    </div>

    <div class="field">
      <label class="field-label">顶端颜色</label>
      <div class="color-row">
        <input type="color" :value="rgbaToHex(config.cap.color)" class="color-picker" @input="applyCapHex(($event.target as HTMLInputElement).value)" />
        <input
          v-model="capHex"
          class="hex-input"
          maxlength="7"
          @change="applyCapHex(capHex)"
          @blur="applyCapHex(capHex)"
        />
      </div>
    </div>

    <div class="field">
      <div class="toggle-row">
        <label class="field-label toggle-label">独立透明度</label>
        <button :class="['toggle', { on: config.cap.independentOpacity }]" @click="() => { const next = !config.cap.independentOpacity; setCapProp('independentOpacity', next); if (!next) setCapProp('opacity', 255) }">
          <span class="toggle-knob"></span>
        </button>
      </div>

      <div v-if="config.cap.independentOpacity" class="opacity-settings fade-in">
        <div class="slider-row" style="margin-top:6px">
          <span class="unit">透明度</span>
          <input v-model.number="opacityModel" type="range" min="0" max="255" class="slider" />
          <span class="slider-val">{{ opacityPct }}%</span>
        </div>
      </div>
    </div>

  </section>
</template>

<style scoped>
.section-icon-svg { color: var(--accent-cyan); flex-shrink: 0; }
.shape-selector { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; }
.shape-btn { display: flex; align-items: center; gap: 8px; padding: 8px 10px; background: var(--bg-surface); border: 1px solid var(--border-color); border-radius: var(--radius-md); color: var(--text-muted); cursor: pointer; transition: all 0.2s ease; font-size: 12px; }
.shape-btn:hover { border-color: var(--accent-cyan); color: var(--text-primary); background: var(--bg-elevated); transform: translateY(-1px); box-shadow: 0 2px 8px rgba(0,0,0,0.3); }
.shape-btn.active { background: var(--accent-cyan-bg); border-color: oklch(0.7 0.16 196 / 0.5); color: var(--accent-cyan); box-shadow: 0 0 12px oklch(0.7 0.16 196 / 0.12); }
.shape-preview-svg { flex-shrink: 0; }
.scale-row { display: flex; align-items: center; gap: 8px; }
.scale-row .slider { flex: 1; }
.scale-num { width: 60px; }
.help-icon { display: inline-flex; align-items: center; justify-content: center; width: 14px; height: 14px; border-radius: 50%; color: var(--text-muted); cursor: help; vertical-align: middle; margin-left: 3px; transition: color .15s }
.help-icon:hover { color: var(--accent-cyan) }
.label-wrap { position: relative; }
.help-tip-banner {
  position: absolute;
  bottom: 100%;
  left: -30px;                   /* 扩展到 panel-scroll 内容区左边界 */
  right: -30px;                  /* 扩展到 panel-scroll 内容区右边界 */
  margin-bottom: 2px;            /* 紧挨 label 上方 */
  padding: 4px 14px;             /* 左右 padding 与 panel-scroll 一致 */
  background: rgba(15,17,29,0.97);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 11px;
  line-height: 1.55;
  color: var(--text-primary);
  text-align: left;
  white-space: normal;
  word-break: break-all;
  z-index: 100;
  box-shadow: 0 4px 16px rgba(0,0,0,0.5);
  pointer-events: none;
}
.color-row { display: flex; align-items: center; gap: 6px; }
.color-picker { width: 30px; height: 30px; border: 2px solid var(--border-color); border-radius: var(--radius-sm); cursor: pointer; background: transparent; padding: 2px; }
.color-picker::-webkit-color-swatch-wrapper { padding: 0; }
.color-picker::-webkit-color-swatch { border-radius: 2px; border: none; }
.hex-input { width: 72px; padding: 4px 6px; background: var(--bg-input); border: 1px solid var(--border-color); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 11px; font-family: 'JetBrains Mono', monospace; outline: none; letter-spacing: 0.5px; }
.hex-input:focus { border-color: var(--accent-cyan); }
.toggle-row { display: flex; align-items: center; justify-content: space-between; }
.toggle-label { margin-bottom: 0 !important; }
.toggle { width: 40px; height: 22px; border-radius: 11px; background: var(--bg-input); border: 1px solid rgba(255,255,255,0.08); position: relative; cursor: pointer; transition: all 0.2s; }
.toggle.on { background: var(--accent-cyan); border-color: var(--accent-cyan); }
.toggle-knob { position: absolute; top: 2px; left: 2px; width: 16px; height: 16px; border-radius: 50%; background: #555; box-shadow: 0 1px 3px rgba(0,0,0,0.4); transition: all 0.2s; }
.toggle.on .toggle-knob { transform: translateX(18px); background: #fff; box-shadow: none; }
.fade-in { animation: fadeSlideIn 0.25s ease-out; }
.opacity-settings { margin-top: 8px; padding: 10px; background: var(--bg-input); border-radius: var(--radius-md); border: 1px solid var(--border-color); border-left: 2px solid var(--accent-cyan); }
@keyframes fadeSlideIn { from { opacity: 0; transform: translateY(-4px); } to { opacity: 1; transform: translateY(0); } }
</style>
