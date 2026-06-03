<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useConfig } from '../../composables/useConfig'
import { rgbaToHex, hexToRgba } from '../../types/config'

const { config, setBodyProp } = useConfig()

// ── 留白 ──
const marginMax = computed(() => Math.floor((config.image.width - 1) / 2))
const contentWidth = computed(() => config.image.width - config.margin * 2)
const marginModel = computed({
  get: () => config.margin,
  set: (v: number) => (config.margin = Math.max(0, Math.min(marginMax.value, v))),
})

// ── 边框 ──
const bHex = ref(rgbaToHex(config.body.borderColor))
watch(() => config.body.borderColor, (c) => { bHex.value = rgbaToHex(c) })
function applyBorderHex(v: string) {
  const clean = v.replace('#', '').trim()
  if (/^[0-9a-fA-F]{6}$/.test(clean)) {
    config.body.borderColor = hexToRgba('#' + clean, config.body.borderColor.a)
  }
}
const borderOpacityModel = computed({
  get: () => config.body.borderOpacity,
  set: (v: number) => setBodyProp('borderOpacity', Math.max(0, Math.min(255, v))),
})
const borderWidthModel = computed({
  get: () => config.body.borderWidth,
  set: (v: number) => setBodyProp('borderWidth', Math.max(1, v)),
})

// ── 整体透明度 ──
const opacityModel = computed({
  get: () => config.globalOpacity,
  set: (v: number) => (config.globalOpacity = Math.max(0, Math.min(255, v))),
})
const pct = computed(() => Math.round((config.globalOpacity / 255) * 100))
</script>

<template>
  <section class="config-section">
    <h3 class="section-title">
      <svg width="14" height="14" viewBox="0 0 14 14" class="section-icon-svg">
        <circle cx="7" cy="7" r="5.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
        <rect x="4" y="4" width="6" height="6" rx="1" fill="currentColor" opacity="0.25"/>
        <line x1="2" y1="7" x2="12" y2="7" stroke="currentColor" stroke-width="0.5" opacity="0.4"/>
        <line x1="7" y1="2" x2="7" y2="12" stroke="currentColor" stroke-width="0.5" opacity="0.4"/>
      </svg>
      整体外观
    </h3>

    <!-- 留白 -->
    <div class="subsection">
      <label class="field-label">
        留白 <span class="unit">px</span>
        <span class="field-hint">（左右对称）</span>
      </label>
      <div class="input-wrap">
        <input
          v-model.number="marginModel"
          type="number"
          :min="0"
          :max="marginMax"
          class="num-input"
        />
      </div>
      <div class="field-info">
        内容区宽度: <strong>{{ contentWidth }}px</strong>
      </div>
    </div>

    <!-- 边框 -->
    <div class="subsection">
      <div class="toggle-row">
        <label class="field-label toggle-label">边框</label>
        <button :class="['toggle', { on: config.body.borderEnabled }]" @click="setBodyProp('borderEnabled', !config.body.borderEnabled)">
          <span class="toggle-knob"></span>
        </button>
      </div>

      <div v-if="config.body.borderEnabled" class="border-settings fade-in">
        <div class="color-row" style="margin-top:6px">
          <input type="color" :value="rgbaToHex(config.body.borderColor)" class="color-picker" @input="applyBorderHex(($event.target as HTMLInputElement).value)" />
          <input v-model="bHex" class="hex-input" maxlength="7" @change="applyBorderHex(bHex)" @blur="applyBorderHex(bHex)" />
        </div>
        <div class="slider-row" style="margin-top:6px">
          <span class="unit">透明度</span>
          <input v-model.number="borderOpacityModel" type="range" min="0" max="255" class="slider" />
          <span class="slider-val">{{ borderOpacityModel }}</span>
        </div>
        <div class="field" style="margin-top:6px">
          <label class="field-label">粗细 <span class="unit">px</span></label>
          <div class="input-wrap">
            <input v-model.number="borderWidthModel" type="number" min="1" class="num-input narrow" />
          </div>
        </div>
      </div>
    </div>

    <!-- 整体透明度 -->
    <div class="subsection">
      <label class="field-label">整体透明度</label>
      <div class="opacity-row">
        <input
          v-model.number="opacityModel"
          type="range"
          min="0"
          max="255"
          class="slider"
        />
        <div class="opacity-value">
          <span class="slider-val">{{ opacityModel }}</span>
          <span class="opacity-pct">/ {{ pct }}%</span>
        </div>
      </div>
      <div class="field-info">
        作用于所有非透明区域，与各区域透明度相乘
      </div>
    </div>
  </section>
</template>

<style scoped>
.section-icon-svg {
  color: var(--accent-cyan);
  flex-shrink: 0;
}
.subsection {
  margin-bottom: 14px;
}
.subsection:last-child {
  margin-bottom: 0;
}
.border-settings {
  margin-top: 8px;
  padding: 10px;
  background: var(--bg-input);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
  border-left: 2px solid var(--accent-cyan);
}
.narrow { max-width: 80px; }
.color-row { display: flex; align-items: center; gap: 6px; }
.color-picker { width: 30px; height: 30px; border: 2px solid var(--border-color); border-radius: var(--radius-sm); cursor: pointer; background: transparent; padding: 2px; }
.color-picker::-webkit-color-swatch-wrapper { padding: 0; }
.color-picker::-webkit-color-swatch { border-radius: 2px; border: none; }
.hex-input { width: 72px; padding: 4px 6px; background: var(--bg-input); border: 1px solid var(--border-color); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 11px; font-family: 'JetBrains Mono', monospace; outline: none; letter-spacing: 0.5px; }
.hex-input:focus { border-color: var(--accent-cyan); }
.toggle-row { display: flex; align-items: center; justify-content: space-between; }
.toggle-label { margin-bottom: 0 !important; }
.toggle { width: 40px; height: 22px; border-radius: 11px; background: var(--bg-surface); border: 1px solid var(--border-color); position: relative; cursor: pointer; transition: all 0.2s; }
.toggle.on { background: var(--accent-cyan); border-color: var(--accent-cyan); }
.toggle-knob { position: absolute; top: 2px; left: 2px; width: 16px; height: 16px; border-radius: 50%; background: #fff; transition: transform 0.2s; }
.toggle.on .toggle-knob { transform: translateX(18px); }
.fade-in { animation: fadeSlideIn 0.25s ease-out; }
@keyframes fadeSlideIn { from { opacity: 0; transform: translateY(-4px); } to { opacity: 1; transform: translateY(0); } }
.opacity-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.slider { flex: 1; }
.opacity-value {
  display: flex;
  align-items: baseline;
  gap: 2px;
  min-width: 56px;
  justify-content: flex-end;
}
.opacity-pct {
  font-size: 10px;
  color: var(--text-muted);
  font-family: 'JetBrains Mono', monospace;
}
</style>
