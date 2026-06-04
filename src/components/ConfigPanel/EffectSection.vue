<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useConfig } from '../../composables/useConfig'
import { rgbaToHex, hexToRgba, isEffectFieldDefault } from '../../types/config'
import RevertButton from './RevertButton.vue'

const { config, setEffectProp, resetEffectField } = useConfig()

// echo 颜色 hex 可编辑
const echoHex = ref(rgbaToHex(config.effect.echoColor))
watch(() => config.effect.echoColor, (c) => { echoHex.value = rgbaToHex(c) })
function applyEchoHex(v: string) {
  const clean = v.replace('#', '').trim()
  if (/^[0-9a-fA-F]{6}$/.test(clean)) {
    config.effect.echoColor = hexToRgba('#' + clean, config.effect.echoColor.a)
  }
}

const echoOpacityModel = computed({
  get: () => config.effect.echoOpacity,
  set: (v: number) => setEffectProp('echoOpacity', Math.max(0, Math.min(255, v))),
})
const echoOpacityPct = computed(() => Math.round((config.effect.echoOpacity / 255) * 100))

// 矩形渐变不允许暗化重复：切换到渐变时自动关闭并重置
const isGradient = computed(() => config.cap.shape === 'gradient')
watch(isGradient, (v) => {
  if (v && config.effect.capEchoEnabled) {
    setEffectProp('capEchoEnabled', false)
    resetEffectField('echoColor')
    resetEffectField('echoOpacity')
    resetEffectField('echoLength')
  }
})

const echoLengthModel = computed({
  get: () => config.effect.echoLength,
  set: (v: number) => setEffectProp('echoLength', Math.max(0, v)),
})

const throwMax = computed(() => Math.max(0, config.image.height - 1))
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

    <div class="subsection">
      <div class="toggle-row">
        <div class="label-row">
          <label class="field-label toggle-label">暗化重复</label>
          <RevertButton :visible="!isEffectFieldDefault(config, 'capEchoEnabled')" @revert="resetEffectField('capEchoEnabled')" />
        </div>
        <button :class="['toggle', { on: config.effect.capEchoEnabled }]" :disabled="isGradient" @click="() => { if (isGradient) return; const next = !config.effect.capEchoEnabled; setEffectProp('capEchoEnabled', next); if (!next) { resetEffectField('echoColor'); resetEffectField('echoOpacity'); resetEffectField('echoLength') } }">
          <span class="toggle-knob"></span>
        </button>
      </div>

      <div v-if="config.effect.capEchoEnabled" class="echo-settings fade-in">
        <div class="field">
          <div class="label-row">
            <label class="field-label">暗化重复颜色</label>
            <RevertButton :visible="!isEffectFieldDefault(config, 'echoColor')" @revert="resetEffectField('echoColor')" />
          </div>
          <div class="color-row" style="margin-top:6px">
            <input type="color" :value="rgbaToHex(config.effect.echoColor)" class="color-picker" @input="applyEchoHex(($event.target as HTMLInputElement).value)" />
            <input v-model="echoHex" class="hex-input" maxlength="7" @change="applyEchoHex(echoHex)" @blur="applyEchoHex(echoHex)" />
          </div>
        </div>

        <div class="field">
          <div class="label-row">
            <label class="field-label">透明度</label>
            <RevertButton :visible="!isEffectFieldDefault(config, 'echoOpacity')" @revert="resetEffectField('echoOpacity')" />
          </div>
          <div class="slider-row" style="margin-top:6px">
            <input v-model.number="echoOpacityModel" type="range" min="0" max="255" class="slider" />
            <span class="slider-val">{{ echoOpacityPct }}%</span>
          </div>
        </div>

        <div class="field">
          <div class="label-row">
            <label class="field-label">暗化重复长度 <span class="unit">px</span></label>
            <RevertButton :visible="!isEffectFieldDefault(config, 'echoLength')" @revert="resetEffectField('echoLength')" />
          </div>
          <div class="input-wrap">
            <input v-model.number="echoLengthModel" type="number" :min="0" :max="throwMax" class="num-input" />
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.section-icon-svg { color: var(--accent-purple); flex-shrink: 0; }
.subsection { margin-bottom: 14px; }
.subsection:last-child { margin-bottom: 0; }
.label-row { display: flex; align-items: center; justify-content: space-between; gap: 6px; }
.toggle-row { display: flex; align-items: center; justify-content: space-between; }
.toggle-label { margin-bottom: 0 !important; }
.toggle:disabled { opacity: 0.4; cursor: not-allowed; }
.echo-settings {
  margin-top: 8px;
  padding: 10px;
  background: var(--bg-input);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
  border-left: 2px solid var(--accent-purple);
}
.field { margin-bottom: 10px; }
.field:last-child { margin-bottom: 0; }
.color-row { display: flex; align-items: center; gap: 6px; }
.color-picker { width: 30px; height: 30px; border: 2px solid var(--border-color); border-radius: var(--radius-sm); cursor: pointer; background: transparent; padding: 2px; }
.color-picker::-webkit-color-swatch-wrapper { padding: 0; }
.color-picker::-webkit-color-swatch { border-radius: 2px; border: none; }
.hex-input { width: 72px; padding: 4px 6px; background: var(--bg-input); border: 1px solid var(--border-color); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 11px; font-family: 'JetBrains Mono', monospace; outline: none; letter-spacing: 0.5px; }
.hex-input:focus { border-color: var(--accent-purple); }
.fade-in { animation: fadeSlideIn 0.25s ease-out; }
@keyframes fadeSlideIn { from { opacity: 0; transform: translateY(-4px); } to { opacity: 1; transform: translateY(0); } }
</style>
