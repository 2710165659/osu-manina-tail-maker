<template>
  <section class="config-section">
    <h3 class="section-title">
      <svg width="14" height="14" viewBox="0 0 14 14" class="section-icon-svg">
        <circle cx="7" cy="7" r="5.5" fill="none" stroke="currentColor" stroke-width="1.2" />
        <rect x="4" y="4" width="6" height="6" rx="1" fill="currentColor" opacity="0.25" />
        <line x1="2" y1="7" x2="12" y2="7" stroke="currentColor" stroke-width="0.5" opacity="0.4" />
        <line x1="7" y1="2" x2="7" y2="12" stroke="currentColor" stroke-width="0.5" opacity="0.4" />
      </svg>
      整体外观
    </h3>

    <div class="field">
      <div class="label-row">
        <label class="field-label">留白 <span class="unit">px</span> <span class="field-hint">（左右对称）</span></label>
        <RevertButton :visible="!isFieldDefault('margin')" @revert="resetField('margin')" />
      </div>
      <div class="input-wrap">
        <input v-model="marginStr" type="text" inputmode="numeric" class="num-input" @blur="applyMargin"
          @keyup.enter="applyMargin" />
      </div>
      <div class="field-info">内容区宽度: <strong>{{ contentWidth }}px</strong></div>
    </div>

    <div class="field">
      <div class="label-row">
        <label class="field-label">投的长度 <span class="unit">px</span></label>
        <RevertButton :visible="!isFieldDefault('throwLength')" @revert="resetField('throwLength')" />
      </div>
      <div class="input-wrap">
        <input v-model="throwStr" type="text" inputmode="numeric" class="num-input" @blur="applyThrow"
          @keyup.enter="applyThrow" />
      </div>
    </div>

    <div class="field">
      <div class="label-row">
        <label class="field-label">颜色</label>
        <RevertButton :visible="!isFieldDefault('globalColor')" @revert="resetField('globalColor')" />
      </div>
      <div class="color-row">
        <ColorPicker v-model:pureColor="colorHex" format="hex" @pureColorChange="debounceApplyHex" :disableAlpha="true" />
        <input v-model="colorHex" class="hex-input" maxlength="7" @blur="applyHex(colorHex)"
          @keyup.enter="applyHex(colorHex)" />
      </div>
    </div>

    <div class="field">
      <div class="label-row">
        <label class="field-label">透明度</label>
        <RevertButton :visible="!isFieldDefault('globalOpacity')" @revert="resetField('globalOpacity')" />
      </div>
      <div class="slider-row">
        <input v-model.number="opacityVal" type="range" min="0" max="255" class="slider" @change="applyOpacity" />
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
          <ColorPicker v-model:pureColor="bHex" format="hex" @pureColorChange="debounceApplyBorderHex" :disableAlpha="true" />
          <input v-model="bHex" class="hex-input" maxlength="7" @blur="applyBorderHex(bHex)"
            @keyup.enter="applyBorderHex(bHex)" />
        </div>

        <div class="opacity-label-row">
          <span class="sub-label">透明度</span>
        </div>
        <div class="slider-row">
          <button :class="['opacity-independent-btn', { on: config.body.borderOpacityIndependent }]"
            @click="setBodyProp('borderOpacityIndependent', !config.body.borderOpacityIndependent)">独立</button>
          <input v-model.number="borderOpacityVal" type="range" min="0" max="255" class="slider"
            :disabled="!config.body.borderOpacityIndependent" @change="applyBorderOpacity" />
          <span class="slider-val">{{ borderOpacityPct }}%</span>
        </div>

        <div class="other-label">粗细 <span class="unit">px</span></div>
        <div class="input-wrap">
          <input v-model="borderStr" type="text" inputmode="numeric" class="num-input narrow" @blur="applyBorderWidth"
            @keyup.enter="applyBorderWidth" />
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useConfig } from '../../composables/useConfig'
import { rgbaToHex, hexToRgba } from '../../types/config'
import { debounce } from '../../utils/debounce'
import RevertButton from './RevertButton.vue'

const { config, resetField, setBodyProp, resetBodyField, isFieldDefault } = useConfig()

const marginMax = computed(() => Math.floor((config.image.width - 1) / 2))
const contentWidth = computed(() => config.image.width - config.margin * 2)
const marginStr = ref(String(config.margin))
watch(() => config.margin, (v) => { marginStr.value = String(v) })
function applyMargin() {
  const v = parseInt(marginStr.value)
  if (!isNaN(v)) config.margin = Math.max(0, Math.min(marginMax.value, v))
  else marginStr.value = String(config.margin)
}

const throwMax = computed(() => Math.max(0, config.image.height - 1))
const throwStr = ref(String(config.throwLength))
watch(() => config.throwLength, (v) => { throwStr.value = String(v) })
function applyThrow() {
  const v = parseInt(throwStr.value)
  if (!isNaN(v)) config.throwLength = Math.max(0, Math.min(throwMax.value, v))
  else throwStr.value = String(config.throwLength)
}

const colorHex = ref(rgbaToHex(config.globalColor))
watch(() => config.globalColor, (c) => { colorHex.value = rgbaToHex(c) })
function applyHex(v: string) {
  let clean = v.replace('#', '').replace(/[^0-9a-fA-F]/g, '')
  if (clean.length > 6) clean = clean.slice(0, 6)
  if (clean.length === 1) clean = clean.repeat(6)
  else if (clean.length === 2) clean = clean.repeat(3)
  else if (clean.length === 3) clean = clean[0] + clean[0] + clean[1] + clean[1] + clean[2] + clean[2]
  else if (clean.length >= 4 && clean.length < 6) clean = clean.padEnd(6, '0')
  if (clean.length === 6) {
    config.globalColor = hexToRgba('#' + clean)
    colorHex.value = '#' + clean
  }
}
const debounceApplyHex = debounce(applyHex, 500)

const opacityVal = ref(config.globalOpacity)
watch(() => config.globalOpacity, (v) => { opacityVal.value = v })
function applyOpacity() { config.globalOpacity = Math.max(0, Math.min(255, opacityVal.value)) }
const opacityPct = computed(() => Math.round((opacityVal.value / 255) * 100))

// 边框
const bHex = ref(rgbaToHex(config.body.borderColor))
watch(() => config.body.borderColor, (c) => { bHex.value = rgbaToHex(c) })
function applyBorderHex(v: string) {
  let clean = v.replace('#', '').replace(/[^0-9a-fA-F]/g, '')
  if (clean.length > 6) clean = clean.slice(0, 6)
  if (clean.length === 1) clean = clean.repeat(6)
  else if (clean.length === 2) clean = clean.repeat(3)
  else if (clean.length === 3) clean = clean[0] + clean[0] + clean[1] + clean[1] + clean[2] + clean[2]
  else if (clean.length >= 4 && clean.length < 6) clean = clean.padEnd(6, '0')
  if (clean.length === 6) {
    config.body.borderColor = hexToRgba('#' + clean, config.body.borderColor.a)
    bHex.value = '#' + clean
  }
}
const debounceApplyBorderHex = debounce(applyBorderHex, 500)
const borderOpacityVal = ref(config.body.borderOpacity)
watch(() => config.body.borderOpacity, (v) => { borderOpacityVal.value = v })
function applyBorderOpacity() { setBodyProp('borderOpacity', Math.max(0, Math.min(255, borderOpacityVal.value))) }
const borderOpacityPct = computed(() => Math.round((borderOpacityVal.value / 255) * 100))
const borderStr = ref(String(config.body.borderWidth))
watch(() => config.body.borderWidth, (v) => { borderStr.value = String(v) })
function applyBorderWidth() {
  const v = parseInt(borderStr.value)
  if (!isNaN(v)) setBodyProp('borderWidth', Math.max(1, v))
  else borderStr.value = String(config.body.borderWidth)
}
</script>

<style scoped>
.section-icon-svg {
  color: var(--accent-purple);
  flex-shrink: 0;
}

.label-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}

.slider-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.slider {
  flex: 1;
}

.slider:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.color-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.color-row :deep(.vc-color-wrap) {
  width: 30px;
  height: 30px;
  min-width: 30px;
  border-radius: var(--radius-sm);
  border: 2px solid var(--border-color);
}

.hex-input {
  width: 72px;
  padding: 4px 6px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 11px;
  font-family: 'JetBrains Mono', monospace;
  outline: none;
  letter-spacing: 0.5px;
}

.hex-input:focus {
  border-color: var(--accent-purple);
}

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
