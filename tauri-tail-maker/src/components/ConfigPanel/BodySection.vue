<template>
  <section class="config-section">
    <h3 class="section-title">
      <svg width="14" height="14" viewBox="0 0 14 14" class="section-icon-svg">
        <rect x="2" y="2" width="10" height="10" rx="2" fill="none" stroke="currentColor" stroke-width="1.2" />
        <rect x="4" y="4" width="6" height="6" rx="1" fill="currentColor" opacity="0.3" />
      </svg>
      面身
    </h3>

    <div class="field">
      <div class="toggle-row">
        <label class="field-label toggle-label">更改面身颜色和透明度</label>
        <div class="toggle-right">
          <button :class="['toggle', { on: config.body.independentSettings }]" @click="toggleIndependent">
            <span class="toggle-knob"></span>
          </button>
        </div>
      </div>

      <div v-if="config.body.independentSettings" class="sub-settings fade-in">
        <div class="sub-label-row">
          <span class="sub-label">颜色</span>
        </div>
        <div class="color-row">
          <ColorPicker v-model:pureColor="bodyHex" format="hex" @pureColorChange="debounceApplyBodyHex" :disableAlpha="true" />
          <input v-model="bodyHex" class="hex-input" maxlength="7" @blur="applyBodyHex(bodyHex)"
            @keyup.enter="applyBodyHex(bodyHex)" />
        </div>

        <div class="opacity-label-row">
          <span class="sub-label">透明度</span>
        </div>
        <div class="slider-row">
          <input v-model.number="opacityVal" type="range" min="0" max="255" class="slider" @change="applyBodyOpacity" />
          <span class="slider-val">{{ opacityPct }}%</span>
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

const { config, setBodyProp } = useConfig()

const bodyHex = ref(rgbaToHex(config.body.color))
watch(() => config.body.color, (c) => { bodyHex.value = rgbaToHex(c) })
function applyBodyHex(v: string) {
  let clean = v.replace('#', '').replace(/[^0-9a-fA-F]/g, '')
  if (clean.length > 6) clean = clean.slice(0, 6)
  if (clean.length === 1) clean = clean.repeat(6)
  else if (clean.length === 2) clean = clean.repeat(3)
  else if (clean.length === 3) clean = clean[0] + clean[0] + clean[1] + clean[1] + clean[2] + clean[2]
  else if (clean.length >= 4 && clean.length < 6) clean = clean.padEnd(6, '0')
  if (clean.length === 6) {
    config.body.color = hexToRgba('#' + clean, config.body.color.a)
    bodyHex.value = '#' + clean
  }
}
const debounceApplyBodyHex = debounce(applyBodyHex, 500)

const opacityVal = ref(config.body.opacity)
watch(() => config.body.opacity, (v) => { opacityVal.value = v })
function applyBodyOpacity() { setBodyProp('opacity', Math.max(0, Math.min(255, opacityVal.value))) }
const opacityPct = computed(() => Math.round((opacityVal.value / 255) * 100))

function toggleIndependent() {
  const next = !config.body.independentSettings
  setBodyProp('independentSettings', next)
  config.body.color = { ...config.globalColor }
  config.body.opacity = config.globalOpacity
}
</script>

<style scoped>
.section-icon-svg {
  color: var(--accent-purple);
  flex-shrink: 0;
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

.opacity-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.opacity-row .slider {
  flex: 1;
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

.opacity-independent-btn.on {
  background: var(--accent-purple-bg);
  border-color: var(--accent-purple);
  color: var(--accent-purple);
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
