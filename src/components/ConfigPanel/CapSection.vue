<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useConfig } from '../../composables/useConfig'
import { CAP_SHAPE_LABELS, CAP_SHAPE_ORDER, rgbaToHex, hexToRgba, isCapFieldDefault } from '../../types/config'
import RevertButton from './RevertButton.vue'

const { config, setCapProp, resetCapField } = useConfig()
const shapes = CAP_SHAPE_ORDER

const capScaleModel = ref(config.cap.scale)
const capScaleFree = ref(false)
watch(() => config.cap.scale, (v) => { if (!capScaleFree.value) capScaleModel.value = v })
function applyScale(v: number) {
  capScaleFree.value = v > 500
  setCapProp('scale', Math.max(1, v))
}

const capHex = ref(rgbaToHex(config.cap.color))
watch(() => config.cap.color, (c) => { capHex.value = rgbaToHex(c) })
function applyCapHex(v: string) {
  const clean = v.replace('#', '').trim()
  if (/^[0-9a-fA-F]{6}$/.test(clean)) config.cap.color = hexToRgba('#' + clean, config.cap.color.a)
}

const opacityModel = computed({
  get: () => config.cap.opacity,
  set: (v: number) => setCapProp('opacity', Math.max(0, Math.min(255, v))),
})
const opacityPct = computed(() => Math.round((config.cap.opacity / 255) * 100))

function toggleIndependent() {
  const next = !config.cap.independentSettings
  setCapProp('independentSettings', next)
  config.cap.color = { ...config.globalColor }
  config.cap.opacity = config.globalOpacity
}
</script>

<template>
  <section class="config-section">
    <h3 class="section-title">
      <svg width="14" height="14" viewBox="0 0 14 14" class="section-icon-svg">
        <polygon points="7,1 13,7 7,13 1,7" fill="none" stroke="currentColor" stroke-width="1.2"/>
        <circle cx="7" cy="5" r="1" fill="currentColor"/>
      </svg>
      顶端
    </h3>

    <div class="field">
      <div class="label-row">
        <label class="field-label">顶端形状</label>
        <RevertButton :visible="!isCapFieldDefault(config, 'shape')" @revert="resetCapField('shape')" />
      </div>
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
      <div class="label-row">
        <label class="field-label">顶端缩放</label>
        <RevertButton :visible="!isCapFieldDefault(config, 'scale')" @revert="resetCapField('scale')" />
      </div>
      <div class="scale-row">
        <input v-model.number="capScaleModel" type="range" min="1" max="500" class="slider" @input="applyScale(capScaleModel)" />
        <input v-model.number="capScaleModel" type="number" :min="1" class="num-input scale-num" @change="applyScale(capScaleModel)" />
      </div>
    </div>

    <!-- 独立设置（颜色 + 透明度） -->
    <div class="field">
      <div class="toggle-row">
        <label class="field-label toggle-label">更改顶端颜色和透明度</label>
        <div class="toggle-right">
          <button :class="['toggle', { on: config.cap.independentSettings }]" @click="toggleIndependent">
            <span class="toggle-knob"></span>
          </button>
        </div>
      </div>

      <div v-if="config.cap.independentSettings" class="sub-settings fade-in">
        <div class="sub-label-row">
          <span class="sub-label">颜色</span>
        </div>
        <div class="color-row">
          <input type="color" :value="rgbaToHex(config.cap.color)" class="color-picker" @input="applyCapHex(($event.target as HTMLInputElement).value)" />
          <input v-model="capHex" class="hex-input" maxlength="7" @change="applyCapHex(capHex)" @blur="applyCapHex(capHex)" />
        </div>

        <div class="opacity-label-row">
          <span class="sub-label">透明度</span>
        </div>
        <div class="slider-row">
          <input v-model.number="opacityModel" type="range" min="0" max="255" class="slider" />
          <span class="slider-val">{{ opacityPct }}%</span>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.section-icon-svg { color: var(--accent-purple); flex-shrink: 0; }
.label-row { display: flex; align-items: center; justify-content: space-between; gap: 6px; }
.toggle-row { display: flex; align-items: center; justify-content: space-between; }
.toggle-label { margin-bottom: 0 !important; }
.toggle-right { display: flex; align-items: center; gap: 6px; }
.shape-selector { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; }
.shape-btn { display: flex; align-items: center; gap: 8px; padding: 8px 10px; background: var(--bg-surface); border: 1px solid var(--border-color); border-radius: var(--radius-md); color: var(--text-muted); cursor: pointer; transition: all 0.2s ease; font-size: 12px; }
.shape-btn:hover { border-color: var(--accent-purple); color: var(--text-primary); background: var(--bg-elevated); }
.shape-btn.active { background: var(--accent-purple-bg); border-color: rgba(183,108,241,0.4); color: var(--accent-purple); box-shadow: 0 0 12px rgba(183,108,241,0.12); }
.shape-preview-svg { flex-shrink: 0; }
.scale-row { display: flex; align-items: center; gap: 8px; }
.scale-row .slider { flex: 1; }
.scale-num { width: 60px; }
.sub-settings { margin-top: 8px; padding: 10px; background: var(--bg-input); border-radius: var(--radius-md); border: 1px solid var(--border-color); border-left: 2px solid var(--accent-purple); }
.sub-label-row, .opacity-label-row { display: flex; align-items: center; justify-content: space-between; margin-bottom: 4px; }
.opacity-label-row { margin-top: 10px; }
.sub-label { font-size: 11px; color: var(--text-secondary); font-weight: 500; }
.color-row { display: flex; align-items: center; gap: 6px; }
.color-picker { width: 30px; height: 30px; border: 2px solid var(--border-color); border-radius: var(--radius-sm); cursor: pointer; background: transparent; padding: 2px; }
.color-picker::-webkit-color-swatch-wrapper { padding: 0; }
.color-picker::-webkit-color-swatch { border-radius: 2px; border: none; }
.hex-input { width: 72px; padding: 4px 6px; background: var(--bg-input); border: 1px solid var(--border-color); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 11px; font-family: 'JetBrains Mono', monospace; outline: none; letter-spacing: 0.5px; }
.hex-input:focus { border-color: var(--accent-purple); }
.slider-row { display: flex; align-items: center; gap: 8px; }
.slider { flex: 1; }
.slider:disabled { opacity: 0.35; cursor: not-allowed; }
.opacity-row { display: flex; align-items: center; gap: 6px; }
.opacity-row .slider { flex: 1; }
.opacity-independent-btn { padding: 2px 6px; font-size: 10px; font-family: inherit; border: 1px solid var(--border-color); border-radius: var(--radius-sm); background: var(--bg-surface); color: var(--text-muted); cursor: pointer; white-space: nowrap; transition: all 0.15s; flex-shrink: 0; }
.opacity-independent-btn.on { background: var(--accent-purple-bg); border-color: var(--accent-purple); color: var(--accent-purple); }
.fade-in { animation: fadeSlideIn 0.25s ease-out; }
@keyframes fadeSlideIn { from { opacity: 0; transform: translateY(-4px); } to { opacity: 1; transform: translateY(0); } }
</style>
