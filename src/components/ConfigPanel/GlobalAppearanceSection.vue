<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useConfig } from '../../composables/useConfig'
import { isFieldDefault, rgbaToHex, hexToRgba } from '../../types/config'
import RevertButton from './RevertButton.vue'

const { config, resetField } = useConfig()

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
  </section>
</template>

<style scoped>
.section-icon-svg { color: var(--accent-purple); flex-shrink: 0; }
.label-row { display: flex; align-items: center; justify-content: space-between; gap: 6px; }
.slider-row { display: flex; align-items: center; gap: 8px; }
.slider { flex: 1; }
.color-row { display: flex; align-items: center; gap: 6px; }
.color-picker { width: 30px; height: 30px; border: 2px solid var(--border-color); border-radius: var(--radius-sm); cursor: pointer; background: transparent; padding: 2px; }
.color-picker::-webkit-color-swatch-wrapper { padding: 0; }
.color-picker::-webkit-color-swatch { border-radius: 2px; border: none; }
.hex-input { width: 72px; padding: 4px 6px; background: var(--bg-input); border: 1px solid var(--border-color); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 11px; font-family: 'JetBrains Mono', monospace; outline: none; letter-spacing: 0.5px; }
.hex-input:focus { border-color: var(--accent-purple); }
</style>
