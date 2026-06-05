<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useConfig } from '../../composables/useConfig'
import { rgbaToHex, hexToRgba } from '../../types/config'

const { config, setBodyProp, setEffectProp, resetBodyField, resetEffectField } = useConfig()

// 边框颜色
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

// 暗化重复
const echoHex = ref(rgbaToHex(config.effect.echoColor))
watch(() => config.effect.echoColor, (c) => { echoHex.value = rgbaToHex(c) })
function applyEchoHex(v: string) {
  const clean = v.replace('#', '').trim()
  if (/^[0-9a-fA-F]{6}$/.test(clean)) config.effect.echoColor = hexToRgba('#' + clean, config.effect.echoColor.a)
}
const echoOpacityModel = computed({
  get: () => config.effect.echoOpacity,
  set: (v: number) => setEffectProp('echoOpacity', Math.max(0, Math.min(255, v))),
})
const echoOpacityPct = computed(() => Math.round((config.effect.echoOpacity / 255) * 100))
const throwMax = computed(() => Math.max(0, config.image.height - 1))
const echoLengthModel = computed({
  get: () => config.effect.echoLength,
  set: (v: number) => setEffectProp('echoLength', Math.max(0, v)),
})

const isGradient = computed(() => config.cap.shape === 'gradient')

function toggleEcho() {
  if (isGradient.value) return
  const next = !config.effect.capEchoEnabled
  setEffectProp('capEchoEnabled', next)
  if (!next) {
    resetEffectField('echoColor')
    resetEffectField('echoOpacity')
    resetEffectField('echoLength')
  }
}
</script>

<template>
  <section class="config-section">
    <h3 class="section-title">
      <svg width="14" height="14" viewBox="0 0 14 14" class="section-icon-svg">
        <rect x="2" y="2" width="10" height="5" rx="1" fill="none" stroke="currentColor" stroke-width="1.2" opacity="0.5"/>
        <rect x="2" y="7" width="10" height="5" rx="1" fill="none" stroke="currentColor" stroke-width="1.2"/>
      </svg>
      效果
    </h3>

    <!-- 边框 -->
    <div class="subsection">
      <div class="toggle-row">
        <label class="field-label toggle-label">边框</label>
        <div class="toggle-right">
          <button :class="['toggle', { on: config.body.borderEnabled }]" @click="() => { const next = !config.body.borderEnabled; setBodyProp('borderEnabled', next); resetBodyField('borderColor'); resetBodyField('borderOpacity'); resetBodyField('borderOpacityIndependent'); resetBodyField('borderWidth') }">
            <span class="toggle-knob"></span>
          </button>
        </div>
      </div>

      <div v-if="config.body.borderEnabled" class="sub-settings fade-in">
        <div class="sub-label-row">
          <span class="sub-label">颜色</span>
        </div>
        <div class="color-row">
          <input type="color" :value="rgbaToHex(config.body.borderColor)" class="color-picker" @input="applyBorderHex(($event.target as HTMLInputElement).value)" />
          <input v-model="bHex" class="hex-input" maxlength="7" @change="applyBorderHex(bHex)" @blur="applyBorderHex(bHex)" />
        </div>

        <div class="opacity-label-row">
          <span class="sub-label">透明度</span>
        </div>
        <div class="slider-row">
          <button :class="['opacity-independent-btn', { on: config.body.borderOpacityIndependent }]" @click="setBodyProp('borderOpacityIndependent', !config.body.borderOpacityIndependent)">独立</button>
          <input v-model.number="borderOpacityModel" type="range" min="0" max="255" class="slider" :disabled="!config.body.borderOpacityIndependent" />
          <span class="slider-val">{{ borderOpacityPct }}%</span>
        </div>

        <div class="other-label">粗细 <span class="unit">px</span></div>
        <div class="input-wrap">
          <input v-model.number="borderWidthModel" type="number" min="1" class="num-input narrow" />
        </div>
      </div>
    </div>

    <!-- 暗化重复 -->
    <div class="subsection">
      <div class="toggle-row">
        <label class="field-label toggle-label">暗化重复 <span v-if="isGradient" class="conflict-hint">与矩形渐变冲突</span></label>
        <div class="toggle-right">
          <button :class="['toggle', { on: config.effect.capEchoEnabled }]" :disabled="isGradient" @click="toggleEcho">
            <span class="toggle-knob"></span>
          </button>
        </div>
      </div>

      <div v-if="config.effect.capEchoEnabled" class="sub-settings fade-in">
        <div class="sub-label-row">
          <span class="sub-label">颜色</span>
        </div>
        <div class="color-row">
          <input type="color" :value="rgbaToHex(config.effect.echoColor)" class="color-picker" @input="applyEchoHex(($event.target as HTMLInputElement).value)" />
          <input v-model="echoHex" class="hex-input" maxlength="7" @change="applyEchoHex(echoHex)" @blur="applyEchoHex(echoHex)" />
        </div>

        <div class="opacity-label-row">
          <span class="sub-label">透明度</span>
        </div>
        <div class="slider-row">
          <input v-model.number="echoOpacityModel" type="range" min="0" max="255" class="slider" />
          <span class="slider-val">{{ echoOpacityPct }}%</span>
        </div>

        <div class="other-label">长度 <span class="unit">px</span></div>
        <div class="input-wrap" style="margin-top:4px">
          <input v-model.number="echoLengthModel" type="number" :min="0" :max="throwMax" class="num-input" />
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.section-icon-svg { color: var(--accent-purple); flex-shrink: 0; }
.subsection { margin-bottom: 14px; }
.subsection:last-child { margin-bottom: 0; }
.toggle-row { display: flex; align-items: center; justify-content: space-between; }
.toggle-label { margin-bottom: 0 !important; }
.toggle-right { display: flex; align-items: center; gap: 6px; }
.toggle:disabled { opacity: 0.4; cursor: not-allowed; }
.sub-settings { margin-top: 8px; padding: 10px; background: var(--bg-input); border-radius: var(--radius-md); border: 1px solid var(--border-color); border-left: 2px solid var(--accent-purple); }
.sub-label-row, .opacity-label-row { display: flex; align-items: center; justify-content: space-between; margin-bottom: 4px; }
.opacity-label-row { margin-top: 10px; }
.sub-label { font-size: 11px; color: var(--text-secondary); font-weight: 500; }
.other-label { font-size: 11px; color: var(--text-secondary); font-weight: 500; margin-top: 10px; margin-bottom: 4px; }
.color-row { display: flex; align-items: center; gap: 6px; }
.color-picker { width: 30px; height: 30px; border: 2px solid var(--border-color); border-radius: var(--radius-sm); cursor: pointer; background: transparent; padding: 2px; }
.color-picker::-webkit-color-swatch-wrapper { padding: 0; }
.color-picker::-webkit-color-swatch { border-radius: 2px; border: none; }
.hex-input { width: 72px; padding: 4px 6px; background: var(--bg-input); border: 1px solid var(--border-color); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 11px; font-family: 'JetBrains Mono', monospace; outline: none; letter-spacing: 0.5px; }
.hex-input:focus { border-color: var(--accent-purple); }
.slider-row { display: flex; align-items: center; gap: 8px; }
.slider { flex: 1; }
.slider:disabled { opacity: 0.35; cursor: not-allowed; }
.opacity-independent-btn { padding: 2px 6px; font-size: 10px; font-family: inherit; border: 1px solid var(--border-color); border-radius: var(--radius-sm); background: var(--bg-surface); color: var(--text-muted); cursor: pointer; white-space: nowrap; transition: all 0.15s; flex-shrink: 0; }
.opacity-independent-btn:hover { border-color: var(--text-muted); color: var(--text-secondary); }
.opacity-independent-btn.on { background: var(--accent-purple-bg); border-color: var(--accent-purple); color: var(--accent-purple); }
.opacity-independent-btn.on:hover { background: var(--accent-purple); color: #fff; }
.narrow { max-width: 80px; }
.input-wrap { position: relative; display: flex; align-items: center; gap: 6px; }
.input-wrap .num-input { flex: 1; }
.fade-in { animation: fadeSlideIn 0.25s ease-out; }
@keyframes fadeSlideIn { from { opacity: 0; transform: translateY(-4px); } to { opacity: 1; transform: translateY(0); } }
.conflict-hint { font-size: 10px; color: var(--text-muted); font-weight: 400; margin-left: 4px; }
</style>
