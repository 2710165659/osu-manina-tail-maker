<script setup lang="ts">
import { computed, ref } from 'vue'
import { useConfig } from '../../composables/useConfig'

const { config, setImageProp } = useConfig()
const widthWarn = ref('')

const widthModel = computed({
  get: () => config.image.width,
  set: (v: number) => {
    if (v > 800) {
      setImageProp('width', 40)
      widthWarn.value = '宽度不能超过800，已重置为40'
      setTimeout(() => { widthWarn.value = '' }, 3000)
      return
    }
    widthWarn.value = ''
    setImageProp('width', Math.max(1, v))
  },
})
const heightModel = computed({
  get: () => config.image.height,
  set: (v: number) => setImageProp('height', Math.max(1, Math.min(65535, v))),
})
const filenameModel = computed({
  get: () => config.image.filename,
  set: (v: string) => setImageProp('filename', v),
})
</script>

<template>
  <section class="config-section">
    <h3 class="section-title">
      <svg width="14" height="14" viewBox="0 0 14 14" class="section-icon-svg">
        <rect x="1" y="1" width="12" height="12" rx="2" fill="none" stroke="currentColor" stroke-width="1.2"/>
        <line x1="1" y1="5" x2="13" y2="5" stroke="currentColor" stroke-width="0.6" opacity="0.5"/>
        <line x1="1" y1="9" x2="13" y2="9" stroke="currentColor" stroke-width="0.6" opacity="0.5"/>
      </svg>
      图片尺寸
    </h3>
    <div class="field-grid">
      <div class="field">
        <label class="field-label">宽度 <span class="unit">px</span></label>
        <div class="input-wrap">
          <input
            v-model.number="widthModel"
            type="number"
            min="1"
            max="800"
            class="num-input"
          />
        </div>
        <p v-if="widthWarn" class="warn">{{ widthWarn }}</p>
      </div>
      <div class="field">
        <label class="field-label">高度 <span class="unit">px</span></label>
        <div class="input-wrap">
          <input
            v-model.number="heightModel"
            type="number"
            min="1"
            max="65535"
            class="num-input"
          />
        </div>
      </div>
    </div>
    <div class="field">
      <label class="field-label">图片名称</label>
      <div class="input-wrap input-with-suffix">
        <input
          v-model="filenameModel"
          type="text"
          class="text-input"
          placeholder="mania-note1H"
          spellcheck="false"
        />
        <span class="input-suffix">.png</span>
      </div>
    </div>
  </section>
</template>

<style scoped>
.section-icon-svg {
  color: var(--accent-cyan);
  flex-shrink: 0;
}
.field-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}
.input-with-suffix {
  position: relative;
}
.input-with-suffix .text-input {
  padding-right: 42px;
}
.warn { color: #ff4466; font-size: 11px; margin-top: 4px }
.input-suffix {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-muted);
  font-size: 11px;
  font-family: 'JetBrains Mono', monospace;
  pointer-events: none;
  background: var(--bg-input);
  padding-left: 4px;
}
</style>
