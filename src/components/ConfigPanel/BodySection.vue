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

const fillOpacityModel = computed({
  get: () => config.body.fillOpacity,
  set: (v: number) => setBodyProp('fillOpacity', Math.max(0, Math.min(255, v))),
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
  </section>
</template>

<style scoped>
.section-icon-svg { color: var(--accent-cyan); flex-shrink: 0; }
.subsection { margin-bottom: 14px; }
.subsection:last-child { margin-bottom: 0; }
.color-row { display: flex; align-items: center; gap: 6px; }
.color-picker { width: 30px; height: 30px; border: 2px solid var(--border-color); border-radius: var(--radius-sm); cursor: pointer; background: transparent; padding: 2px; }
.color-picker::-webkit-color-swatch-wrapper { padding: 0; }
.color-picker::-webkit-color-swatch { border-radius: 2px; border: none; }
.hex-input { width: 72px; padding: 4px 6px; background: var(--bg-input); border: 1px solid var(--border-color); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 11px; font-family: 'JetBrains Mono', monospace; outline: none; letter-spacing: 0.5px; }
.hex-input:focus { border-color: var(--accent-cyan); }
</style>
