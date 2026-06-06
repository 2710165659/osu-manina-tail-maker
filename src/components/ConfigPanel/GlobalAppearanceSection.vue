<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useConfig } from '../../composables/useConfig'
import { isFieldDefault, rgbaToHex, hexToRgba } from '../../types/config'
import RevertButton from './RevertButton.vue'

const { config, resetField, setBodyProp, resetBodyField } = useConfig()

const marginMax = computed(() => Math.floor((config.image.width - 1) / 2))
const contentWidth = computed(() => config.image.width - config.margin * 2)
const marginModel = computed({
  get: () => config.margin,
  set: (v: number) => (config.margin = Math.max(0, Math.min(marginMax.value, v))),
})

const throwMax = computed(() => Math.max(0, config.image.height - 1))
const throwModel = computed({
  get: () => config.throwLength,
  set: (v: number) => (config.throwLength = Math.max(0, Math.min(throwMax.value, v))),
})

const colorHex = ref(rgbaToHex(config.globalColor))
watch(() => config.globalColor, (c) => { colorHex.value = rgbaToHex(c) })
function applyHex(v: string) {
  const clean = v.replace('#', '').trim()
  if (/^[0-9a-fA-F]{6}$/.test(clean)) config.globalColor = hexToRgba('#' + clean)
}

const opacityModel = computed({
  get: () => config.globalOpacity,
  set: (v: number) => (config.globalOpacity = Math.max(0, Math.min(255, v))),
})
const opacityPct = computed(() => Math.round((config.globalOpacity / 255) * 100))

// 边框
const bHex = ref(rgbaToHex(config.body.borderColor))
watch(() => config.body.borderColor, (c) => { bHex.value = rgbaToHex(c) })
function applyBorderHex(v: string) {
  const clean = v.replace('#', '').trim()
  if (/^[0-9a-fA-F]{6}$/.test(clean)) config.body.borderColor = hexToRgba('#' + clean, config.body.borderColor.a)
}
const borderOpacityModel = computed({
  get: () => config.body.borderOpacity,
  set: (v: number) => setBodyProp('borderOpacity', Math.max(0, Math.min(255, v))),
})
const borderOpacityPct = computed(() => Math.round((config.body.borderOpacity / 255) * 100))
const borderWidthModel = computed({
  get: () => config.body.borderWidth,
  set: (v: number) => setBodyProp('borderWidth', Math.max(1, v)),
})
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

    <div class="field">
      <div class="label-row">
        <label class="field-label">留白 <span class="unit">px</span> <span class="field-hint">（左右对称）</span></label>
        <RevertButton :visible="!isFieldDefault(config, 'margin')" @revert="resetField('margin')" />
      </div>
      <div class="input-wrap">
        <input v-model.number="marginModel" type="number" :min="0" :max="marginMax" class="num-input" />
      </div>
      <div class="field-info">内容区宽度: <strong>{{ contentWidth }}px</strong></div>
    </div>

    <div class="field">
      <div class="label-row">
        <label class="field-label">投的长度 <span class="unit">px</span></label>
        <RevertButton :visible="!isFieldDefault(config, 'throwLength')" @revert="resetField('throwLength')" />
      </div>
      <div class="input-wrap">
        <input v-model.number="throwModel" type="number" :min="0" :max="throwMax" class="num-input" />
      </div>
    </div>

    <div class="field">
      <div class="label-row">
        <label class="field-label">颜色</label>
        <RevertButton :visible="!isFieldDefault(config, 'globalColor')" @revert="resetField('globalColor')" />
      </div>
      <div class="color-row">
        <input type="color" :value="rgbaToHex(config.globalColor)" class="color-picker" @input="applyHex(($event.target as HTMLInputElement).value)" />
        <input v-model="colorHex" class="hex-input" maxlength="7" @change="applyHex(colorHex)" @blur="applyHex(colorHex)" />
      </div>
    </div>

    <div class="field">
      <div class="label-row">
        <label class="field-label">透明度</label>
        <RevertButton :visible="!isFieldDefault(config, 'globalOpacity')" @revert="resetField('globalOpacity')" />
      </div>
      <div class="slider-row">
        <input v-model.number="opacityModel" type="range" min="0" max="255" class="slider" />
        <span class="slider-val">{{ opacityPct }}%</span>
      </div>
    </div>

    <!-- 边框 -->
    <div class="subsection">
      <div class="toggle-row">
        <label class="field-label toggle-label">内边框</label>
        <div class="toggle-right">
          <button :class="['toggle', { on: config.body.borderEnabled }]"
            @click="() => { const next = !config.body.borderEnabled; setBodyProp('borderEnabled', next); resetBodyField('borderColor'); resetBodyField('borderOpacity'); resetBodyField('borderOpacityIndependent'); resetBodyField('borderWidth') }">
            <span class="toggle-knob"></span>
          </button>
        </div>
      </div>

      <div v-if="config.body.borderEnabled" class="sub-settings fade-in">
        <div class="sub-label-row">
          <span class="sub-label">颜色</span>
        </div>
        <div class="color-row">
          <input type="color" :value="rgbaToHex(config.body.borderColor)" class="color-picker"
            @input="applyBorderHex(($event.target as HTMLInputElement).value)" />
          <input v-model="bHex" class="hex-input" maxlength="7" @change="applyBorderHex(bHex)"
            @blur="applyBorderHex(bHex)" />
        </div>

        <div class="opacity-label-row">
          <span class="sub-label">透明度</span>
        </div>
        <div class="slider-row">
          <button :class="['opacity-independent-btn', { on: config.body.borderOpacityIndependent }]"
            @click="setBodyProp('borderOpacityIndependent', !config.body.borderOpacityIndependent)">独立</button>
          <input v-model.number="borderOpacityModel" type="range" min="0" max="255" class="slider"
            :disabled="!config.body.borderOpacityIndependent" />
          <span class="slider-val">{{ borderOpacityPct }}%</span>
        </div>

        <div class="other-label">粗细 <span class="unit">px</span></div>
        <div class="input-wrap">
          <input v-model.number="borderWidthModel" type="number" min="1" class="num-input narrow" />
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.section-icon-svg { color: var(--accent-purple); flex-shrink: 0; }
.label-row { display: flex; align-items: center; justify-content: space-between; gap: 6px; }
.slider-row { display: flex; align-items: center; gap: 8px; }
.slider { flex: 1; }
.slider:disabled { opacity: 0.35; cursor: not-allowed; }
.color-row { display: flex; align-items: center; gap: 6px; }
.color-picker { width: 30px; height: 30px; border: 2px solid var(--border-color); border-radius: var(--radius-sm); cursor: pointer; background: transparent; padding: 2px; }
.color-picker::-webkit-color-swatch-wrapper { padding: 0; }
.color-picker::-webkit-color-swatch { border-radius: 2px; border: none; }
.hex-input { width: 72px; padding: 4px 6px; background: var(--bg-input); border: 1px solid var(--border-color); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 11px; font-family: 'JetBrains Mono', monospace; outline: none; letter-spacing: 0.5px; }
.hex-input:focus { border-color: var(--accent-purple); }

.subsection {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px solid var(--border-color);
}

.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.toggle-label {
  margin-bottom: 0 !important;
}

.toggle-right {
  display: flex;
  align-items: center;
  gap: 6px;
}

.toggle:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.sub-settings {
  margin-top: 8px;
  padding: 10px;
  background: var(--bg-input);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
  border-left: 2px solid var(--accent-purple);
}

.sub-label-row,
.opacity-label-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 4px;
}

.opacity-label-row {
  margin-top: 10px;
}

.sub-label {
  font-size: 11px;
  color: var(--text-secondary);
  font-weight: 500;
}

.other-label {
  font-size: 11px;
  color: var(--text-secondary);
  font-weight: 500;
  margin-top: 10px;
  margin-bottom: 4px;
}

.opacity-independent-btn {
  padding: 2px 6px;
  font-size: 10px;
  font-family: inherit;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-surface);
  color: var(--text-muted);
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.15s;
  flex-shrink: 0;
}

.opacity-independent-btn:hover {
  border-color: var(--text-muted);
  color: var(--text-secondary);
}

.opacity-independent-btn.on {
  background: var(--accent-purple-bg);
  border-color: var(--accent-purple);
  color: var(--accent-purple);
}

.opacity-independent-btn.on:hover {
  background: var(--accent-purple);
  color: #fff;
}

.narrow {
  max-width: 80px;
}

.input-wrap {
  position: relative;
  display: flex;
  align-items: center;
  gap: 6px;
}

.input-wrap .num-input {
  flex: 1;
}

.fade-in {
  animation: fadeSlideIn 0.25s ease-out;
}

@keyframes fadeSlideIn {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
