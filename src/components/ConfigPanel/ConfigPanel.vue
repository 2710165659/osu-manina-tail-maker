<script setup lang="ts">
import { onMounted, ref } from 'vue'
import ImageSection from './ImageSection.vue'
import MarginSection from './MarginSection.vue'
import CapSection from './CapSection.vue'
import BodySection from './BodySection.vue'
import GlobalSection from './GlobalSection.vue'
import PresetSection from './PresetSection.vue'

const loaded = ref(false)
onMounted(() => { loaded.value = true })
</script>

<template>
  <aside class="config-panel">
    <div class="panel-header">
      <div class="app-logo">
        <svg class="logo-svg" width="28" height="28" viewBox="0 0 28 28" fill="none">
          <defs>
            <linearGradient id="logoGrad" x1="0" y1="0" x2="28" y2="28">
              <stop offset="0%" stop-color="#00d4f0"/>
              <stop offset="100%" stop-color="#ff2d95"/>
            </linearGradient>
            <linearGradient id="logoBody" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#00d4f0" stop-opacity="0.6"/>
              <stop offset="100%" stop-color="#00d4f0" stop-opacity="0.25"/>
            </linearGradient>
          </defs>
          <!-- Body rect -->
          <rect x="8" y="10" width="12" height="16" rx="2" stroke="url(#logoGrad)" stroke-width="1.2" fill="url(#logoBody)"/>
          <!-- Cap: upper half-ellipse -->
          <path d="M8 12 A6 6 0 0 1 20 12" stroke="url(#logoGrad)" stroke-width="1.5" fill="url(#logoGrad)" fill-opacity="0.35"/>
          <!-- Note lanes hint -->
          <line x1="3" y1="4" x2="25" y2="4" stroke="url(#logoGrad)" stroke-width="0.4" opacity="0.4"/>
          <line x1="3" y1="6" x2="25" y2="6" stroke="url(#logoGrad)" stroke-width="0.4" opacity="0.25"/>
          <line x1="3" y1="24" x2="25" y2="24" stroke="url(#logoGrad)" stroke-width="0.4" opacity="0.25"/>
          <line x1="3" y1="26" x2="25" y2="26" stroke="url(#logoGrad)" stroke-width="0.4" opacity="0.4"/>
          <!-- Hit marker -->
          <circle cx="14" cy="5" r="1.5" fill="#ff2d95" opacity="0.7"/>
        </svg>
        <div class="logo-text-group">
          <span class="logo-title">Tail Maker</span>
          <span class="logo-subtitle">osu!mania 投皮生成器</span>
        </div>
      </div>
    </div>
    <div class="panel-scroll">
      <div :class="['section-wrapper', { visible: loaded }]" style="--i:0">
        <ImageSection />
      </div>
      <div :class="['section-wrapper', { visible: loaded }]" style="--i:1">
        <MarginSection />
      </div>
      <div :class="['section-wrapper', { visible: loaded }]" style="--i:2">
        <CapSection />
      </div>
      <div :class="['section-wrapper', { visible: loaded }]" style="--i:3">
        <BodySection />
      </div>
      <div :class="['section-wrapper', { visible: loaded }]" style="--i:4">
        <GlobalSection />
      </div>
      <div :class="['section-wrapper', { visible: loaded }]" style="--i:5">
        <PresetSection />
      </div>
    </div>
  </aside>
</template>

<style scoped>
.config-panel {
  width: 350px;
  min-width: 350px;
  height: 100vh;
  background: var(--bg-panel);
  border-left: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
}
/* Subtle side glow on the panel edge */
.config-panel::after {
  content: '';
  position: absolute;
  left: -1px;
  top: 0;
  bottom: 0;
  width: 1px;
  background: linear-gradient(
    180deg,
    transparent 0%,
    var(--accent-cyan) 15%,
    transparent 30%,
    var(--accent-rose) 60%,
    transparent 80%,
    var(--accent-cyan) 95%,
    transparent 100%
  );
  opacity: 0.3;
  pointer-events: none;
}

.panel-header {
  padding: 18px 20px 14px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
  background: linear-gradient(180deg, rgba(0,212,240,0.03) 0%, transparent 100%);
}
.app-logo {
  display: flex;
  align-items: center;
  gap: 12px;
}
.logo-svg {
  color: var(--accent-cyan);
  filter: drop-shadow(0 0 8px oklch(0.7 0.16 196 / 0.5));
  flex-shrink: 0;
}
.logo-text-group {
  display: flex;
  flex-direction: column;
  gap: 1px;
}
.logo-title {
  font-size: 18px;
  font-weight: 700;
  letter-spacing: 0.5px;
  background: linear-gradient(135deg, var(--accent-cyan), #8bf0ff);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  line-height: 1.2;
}
.logo-subtitle {
  font-size: 10px;
  color: var(--text-muted);
  letter-spacing: 0.8px;
  text-transform: uppercase;
  font-weight: 500;
}

.panel-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 10px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* Staggered entry animation */
.section-wrapper {
  opacity: 0;
  transform: translateX(-8px);
  transition: opacity 0.35s ease-out, transform 0.35s ease-out;
  transition-delay: calc(var(--i, 0) * 60ms + 50ms);
}
.section-wrapper.visible {
  opacity: 1;
  transform: translateX(0);
}

.panel-scroll::-webkit-scrollbar { width: 4px; }
.panel-scroll::-webkit-scrollbar-track { background: transparent; }
.panel-scroll::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 2px;
}
</style>
