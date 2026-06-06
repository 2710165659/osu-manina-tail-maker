<script setup lang="ts">
import { onMounted } from 'vue'
import { useConfig } from './composables/useConfig'
import ConfigPanel from './components/ConfigPanel/ConfigPanel.vue'
import PreviewPanel from './components/PreviewPanel/PreviewPanel.vue'
import NotificationToast from './components/NotificationToast.vue'

const { init, loadUserPresets } = useConfig()

onMounted(async () => {
  await init()
  loadUserPresets()
})
</script>

<template>
  <div class="app-shell">
    <PreviewPanel />
    <ConfigPanel />
    <NotificationToast />
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background: var(--bg-base);
}
</style>
