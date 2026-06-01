<script setup lang="ts">
import { computed } from 'vue'
import { useConfig } from '../../composables/useConfig'

const { config } = useConfig()

const opacityModel = computed({
  get: () => config.globalOpacity,
  set: (v: number) => (config.globalOpacity = Math.max(0, Math.min(255, v))),
})
const pct = computed(() => Math.round((config.globalOpacity / 255) * 100))
</script>

<template>
  <section class="config-section">
    <h3 class="section-title">
      <svg width="14" height="14" viewBox="0 0 14 14" class="section-icon-svg">
        <circle cx="7" cy="7" r="5" fill="none" stroke="currentColor" stroke-width="1.2"/>
        <circle cx="7" cy="7" r="2" fill="currentColor" opacity="0.5"/>
      </svg>
      整体透明度
    </h3>
    <div class="opacity-row">
      <input
        v-model.number="opacityModel"
        type="range"
        min="0"
        max="255"
        class="slider"
      />
      <div class="opacity-value">
        <span class="slider-val">{{ opacityModel }}</span>
        <span class="opacity-pct">/ {{ pct }}%</span>
      </div>
    </div>
    <div class="field-info">
      作用于所有非透明区域，与各区域透明度相乘
    </div>
  </section>
</template>

<style scoped>
.section-icon-svg {
  color: var(--accent-cyan);
  flex-shrink: 0;
}
.opacity-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.slider { flex: 1; }
.opacity-value {
  display: flex;
  align-items: baseline;
  gap: 2px;
  min-width: 56px;
  justify-content: flex-end;
}
.opacity-pct {
  font-size: 10px;
  color: var(--text-muted);
  font-family: 'JetBrains Mono', monospace;
}
</style>
