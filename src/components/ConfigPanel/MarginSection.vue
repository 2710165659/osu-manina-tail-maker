<script setup lang="ts">
import { computed } from 'vue'
import { useConfig } from '../../composables/useConfig'

const { config } = useConfig()

const marginMax = computed(() => Math.floor((config.image.width - 1) / 2))
const contentWidth = computed(() => config.image.width - config.margin * 2)
const model = computed({
  get: () => config.margin,
  set: (v: number) => (config.margin = Math.max(0, Math.min(marginMax.value, v))),
})
</script>

<template>
  <section class="config-section">
    <h3 class="section-title">
      <svg width="14" height="14" viewBox="0 0 14 14" class="section-icon-svg">
        <rect x="2" y="1" width="3" height="12" rx="0.5" fill="currentColor" opacity="0.3"/>
        <rect x="9" y="1" width="3" height="12" rx="0.5" fill="currentColor" opacity="0.3"/>
        <rect x="5" y="3" width="4" height="8" rx="1" fill="currentColor" opacity="0.5"/>
      </svg>
      留白
    </h3>
    <div class="field">
      <label class="field-label">
        留白 <span class="unit">px</span>
        <span class="field-hint">（左右对称）</span>
      </label>
      <div class="input-wrap">
        <input
          v-model.number="model"
          type="number"
          :min="0"
          :max="marginMax"
          class="num-input"
        />
      </div>
      <div class="field-info">
        内容区宽度: <strong>{{ contentWidth }}px</strong>
      </div>
    </div>
  </section>
</template>

<style scoped>
.section-icon-svg {
  color: var(--accent-cyan);
  flex-shrink: 0;
}
</style>
