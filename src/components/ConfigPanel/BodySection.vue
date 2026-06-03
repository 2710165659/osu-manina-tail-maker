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
const fillOpacityPct = computed(() => Math.round((config.body.fillOpacity / 255) * 100))

// 独立设置关闭时，显示投皮头的颜色
const effectiveColor = computed(() => {
  if (config.body.independentFill) return config.body.fillColor
  return config.cap.color
})
</script>

<template>
  <section class="config-section">
    <h3 class="section-title">
      <svg width="14" height="14" viewBox="0 0 14 14" class="section-icon-svg">
        <rect x="2" y="2" width="10" height="10" rx="2" fill="none" stroke="currentColor" stroke-width="1.2"/>
        <rect x="4" y="4" width="6" height="6" rx="1" fill="currentColor" opacity="0.3"/>
      </svg>
      投皮身
    </h3>

    <div class="subsection">
      <div class="toggle-row">
        <label class="field-label toggle-label">独立设置</label>
        <button :class="['toggle', { on: config.body.independentFill }]" @click="() => { const next = !config.body.independentFill; setBodyProp('independentFill', next); if (!next) { config.body.fillColor = { ...config.cap.color }; setBodyProp('fillOpacity', 255) } }">
          <span class="toggle-knob"></span>
        </button>
      </div>

      <div v-if="config.body.independentFill" class="fill-settings fade-in">
        <label class="field-label">填充颜色</label>
        <div class="color-row" style="margin-top:6px">
          <input type="color" :value="rgbaToHex(config.body.fillColor)" class="color-picker" @input="applyFillHex(($event.target as HTMLInputElement).value)" />
          <input v-model="fHex" class="hex-input" maxlength="7" @change="applyFillHex(fHex)" @blur="applyFillHex(fHex)" />
        </div>
        <div class="slider-row" style="margin-top:6px">
          <span class="unit">透明度</span>
          <input v-model.number="fillOpacityModel" type="range" min="0" max="255" class="slider" />
          <span class="slider-val">{{ fillOpacityPct }}%</span>
        </div>
      </div>

      <div v-else class="sync-hint fade-in">
        <div class="hint-color-row">
          <span class="hint-swatch" :style="{ background: rgbaToHex(effectiveColor) }"></span>
          <span class="hint-text">颜色与投皮头一致</span>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.section-icon-svg { color: var(--accent-cyan); flex-shrink: 0; }
.subsection { margin-bottom: 14px; }
.subsection:last-child { margin-bottom: 0; }
.fill-settings { margin-top: 8px; padding: 10px; background: var(--bg-input); border-radius: var(--radius-md); border: 1px solid var(--border-color); border-left: 2px solid var(--accent-cyan); }
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
@keyframes fadeSlideIn { from { opacity: 0; transform: translateY(-4px); } to { opacity: 1; transform: translateY(0); } }
.sync-hint { margin-top: 8px; padding: 8px 10px; background: var(--bg-input); border-radius: var(--radius-md); border: 1px solid var(--border-color); }
.hint-color-row { display: flex; align-items: center; gap: 8px; }
.hint-swatch { width: 16px; height: 16px; border-radius: 3px; border: 1px solid var(--border-color); flex-shrink: 0; }
.hint-text { font-size: 11px; color: var(--text-muted); }
</style>
