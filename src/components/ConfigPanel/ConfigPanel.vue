<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import ImageSection from './ImageSection.vue'
import GlobalAppearanceSection from './GlobalAppearanceSection.vue'
import CapSection from './CapSection.vue'
import BodySection from './BodySection.vue'
import EffectSection from './EffectSection.vue'

const loaded = ref(false)
onMounted(() => { loaded.value = true })

function openGitHub() {
  invoke('open_url', { url: 'https://github.com/2710165659/osu-manina-tail-maker' })
}
</script>

<template>
  <aside class="config-panel">
    <div class="panel-header">
      <div class="app-logo">
        <svg class="logo-svg" width="28" height="28" viewBox="0 0 28 28" fill="none">
          <defs>
            <linearGradient id="logoGrad" x1="0" y1="0" x2="28" y2="28">
              <stop offset="0%" stop-color="#b76cf1" />
              <stop offset="100%" stop-color="#ff66aa" />
            </linearGradient>
            <linearGradient id="logoBody" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#b76cf1" stop-opacity="0.6" />
              <stop offset="100%" stop-color="#b76cf1" stop-opacity="0.25" />
            </linearGradient>
          </defs>
          <!-- Body rect -->
          <rect x="8" y="10" width="12" height="16" rx="2" stroke="url(#logoGrad)" stroke-width="1.2"
            fill="url(#logoBody)" />
          <!-- Cap: upper half-ellipse -->
          <path d="M8 12 A6 6 0 0 1 20 12" stroke="url(#logoGrad)" stroke-width="1.5" fill="url(#logoGrad)"
            fill-opacity="0.35" />
          <!-- Note lanes hint -->
          <line x1="3" y1="4" x2="25" y2="4" stroke="url(#logoGrad)" stroke-width="0.4" opacity="0.4" />
          <line x1="3" y1="6" x2="25" y2="6" stroke="url(#logoGrad)" stroke-width="0.4" opacity="0.25" />
          <line x1="3" y1="24" x2="25" y2="24" stroke="url(#logoGrad)" stroke-width="0.4" opacity="0.25" />
          <line x1="3" y1="26" x2="25" y2="26" stroke="url(#logoGrad)" stroke-width="0.4" opacity="0.4" />
          <!-- Hit marker -->
          <circle cx="14" cy="5" r="1.5" fill="#ff66aa" opacity="0.7" />
        </svg>
        <div class="logo-text-group">
          <span class="logo-title">Tail Maker</span>
          <span class="logo-subtitle">osu!mania 投皮生成器</span>
        </div>
        <span class="github-link" @click="openGitHub" title="GitHub">
          <svg width="24" height="24" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38
              0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52
              -.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2
              -3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82
              .64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08
              2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01
              1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" />
          </svg>
        </span>
      </div>
    </div>
    <div class="panel-scroll">
      <div :class="['section-wrapper', { visible: loaded }]" style="--i:0">
        <ImageSection />
      </div>
      <div :class="['section-wrapper', { visible: loaded }]" style="--i:1">
        <GlobalAppearanceSection />
      </div>
      <div :class="['section-wrapper', { visible: loaded }]" style="--i:2">
        <CapSection />
      </div>
      <div :class="['section-wrapper', { visible: loaded }]" style="--i:3">
        <BodySection />
      </div>
      <div :class="['section-wrapper', { visible: loaded }]" style="--i:4">
        <EffectSection />
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
  overflow: visible;
  position: relative;
  z-index: 10001;
  /* 高于 body::before 的 9999，确保 tooltip 不被遮挡 */
}

/* Subtle side glow on the panel edge */
.config-panel::after {
  content: '';
  position: absolute;
  left: -1px;
  top: 0;
  bottom: 0;
  width: 1px;
  background: linear-gradient(180deg,
      transparent 0%,
      var(--accent-purple) 15%,
      transparent 30%,
      var(--accent-pink) 60%,
      transparent 80%,
      var(--accent-purple) 95%,
      transparent 100%);
  opacity: 0.3;
  pointer-events: none;
}

.panel-header {
  padding: 18px 20px 14px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
  background: linear-gradient(180deg, rgba(183, 108, 241, 0.03) 0%, transparent 100%);
}

.app-logo {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-svg {
  color: var(--accent-purple);
  filter: drop-shadow(0 0 8px rgba(183, 108, 241, 0.5));
  flex-shrink: 0;
}

.logo-text-group {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.github-link {
  color: var(--text-muted);
  display: flex;
  align-items: center;
  cursor: pointer;
  transition: color 0.15s;
  margin-left: auto;
  flex-shrink: 0;
}

.github-link:hover {
  color: var(--text-primary);
}

.logo-title {
  font-size: 18px;
  font-weight: 700;
  letter-spacing: 0.5px;
  background: linear-gradient(135deg, var(--accent-purple), var(--accent-purple-light));
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

.panel-scroll::-webkit-scrollbar {
  width: 4px;
}

.panel-scroll::-webkit-scrollbar-track {
  background: transparent;
}

.panel-scroll::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 2px;
}
</style>
