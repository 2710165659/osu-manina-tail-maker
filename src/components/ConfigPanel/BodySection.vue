<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useConfig } from '../../composables/useConfig'
import { rgbaToHex, hexToRgba } from '../../types/config'

const { config, setBodyProp } = useConfig()

// 填充颜色 hex 可编辑
const fHex = ref(rgbaToHex(config.body.fillColor))
watch(() => config.body.fillColor, (c) => { fHex.value = rgbaToHex(c) })
function applyFillHex(v: string) {
  const clean = v.replace('#', '').trim()
  if (/^[0-9a-fA-F]{6}$/.test(clean)) {
    config.body.fillColor = hexToRgba('#' + clean, config.body.fillColor.a)
  }
}
// 边框颜色 hex 可编辑
const bHex = ref(rgbaToHex(config.body.borderColor))
watch(() => config.body.borderColor, (c) => { bHex.value = rgbaToHex(c) })
function applyBorderHex(v: string) {
  const clean = v.replace('#', '').trim()
  if (/^[0-9a-fA-F]{6}$/.test(clean)) {
    config.body.borderColor = hexToRgba('#' + clean, config.body.borderColor.a)
  }
}

const fillOpacityModel = computed({
  get: () => config.body.fillOpacity,
  set: (v: number) => setBodyProp('fillOpacity', Math.max(0, Math.min(255, v))),
})
const borderOpacityModel = computed({
  get: () => config.body.borderOpacity,
  set: (v: number) => setBodyProp('borderOpacity', Math.max(0, Math.min(255, v))),
})
const borderWidthModel = computed({
  get: () => config.body.borderWidth,
  set: (v: number) => setBodyProp('borderWidth', Math.max(1, v)),
})
</script>

<template>
  <section class="config-section">
    <h3 class="section-title">
      <svg width="14" height="14" viewBox="0 0 14 14" class="section-icon-svg">
        <rect x="2" y="2" width="10" height="10" rx="2" fill="none" stroke="currentColor" stroke-width="1.2"/>
        <rect x="4" y="4" width="6" height="6" rx="1" fill="currentColor" opacity="0.3"/>
      </svg>
      身体
    </h3>

    <div class="subsection">
      <label class="field-label">填充</label>
      <div class="color-row">
        <input type="color" :value="rgbaToHex(config.body.fillColor)" class="color-picker" @input="applyFillHex(($event.target as HTMLInputElement).value)" />
        <input v-model="fHex" class="hex-input" maxlength="7" @change="applyFillHex(fHex)" @blur="applyFillHex(fHex)" />
      </div>
      <div class="slider-row" style="margin-top:6px">
        <span class="unit">透明度</span>
        <input v-model.number="fillOpacityModel" type="range" min="0" max="255" class="slider" />
        <span class="slider-val">{{ fillOpacityModel }}</span>
      </div>
    </div>

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
  </section>
</template>

<style scoped>
.section-icon-svg { color: var(--accent-cyan); flex-shrink: 0; }
.subsection { margin-bottom: 14px; }
.subsection:last-child { margin-bottom: 0; }
.border-settings { margin-top: 8px; padding: 10px; background: var(--bg-input); border-radius: var(--radius-md); border: 1px solid var(--border-color); border-left: 2px solid var(--accent-cyan); }
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
</style>
