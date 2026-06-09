<template>
  <div class="tool-overlay" @mousedown.self="emit('close')">
    <div class="tool-panel">
      <!-- 顶部装饰条 -->
      <div class="panel-stripe"></div>

      <div class="panel-inner">
        <!-- 头部 -->
        <header class="panel-header">
          <div class="header-left">
            <div class="tool-icon-wrap">
              <svg width="18" height="18" viewBox="0 0 18 18" fill="none">
                <path d="M7.5 1.5L3 6l4.5 4.5L12 6 7.5 1.5z" stroke="currentColor" stroke-width="1.2"
                  stroke-linejoin="round" />
                <path d="M3 12l4.5 4.5L12 12" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
                <circle cx="14" cy="4" r="2.5" stroke="currentColor" stroke-width="1.2" />
                <path d="M14 2.5v3M12.5 4h3" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
              </svg>
            </div>
            <div class="header-text">
              <h2 class="panel-title">工具箱</h2>
              <span class="panel-subtitle">Toolbox</span>
            </div>
          </div>
          <button class="close-btn" @click="emit('close')">
            <svg width="14" height="14" viewBox="0 0 14 14">
              <path d="M3 3l8 8M11 3l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
          </button>
        </header>

        <!-- 工具选择器 -->
        <nav class="tool-nav">
          <button v-for="tab in tabs" :key="tab.id" :class="['nav-item', { active: activeTab === tab.id }]"
            @click="activeTab = tab.id">
            <span class="nav-label">{{ tab.label }}</span>
            <span class="nav-indicator"></span>
          </button>
        </nav>

        <!-- 内容区域 - 固定高度 -->
        <main class="tool-body">
          <Transition name="tool-switch" mode="out-in">
            <TailRepair v-if="activeTab === 'tailRepair'" key="tailRepair" />
            <BatchGenerate v-else-if="activeTab === 'batchGenerate'" key="batchGenerate" />
            <AddScript v-else-if="activeTab === 'addScript'" key="addScript" />
          </Transition>
        </main>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import TailRepair from '../ToolPanel/TailRepair.vue'
import BatchGenerate from '../ToolPanel/BatchGenerate.vue'
import AddScript from '../ToolPanel/AddScript.vue'

const emit = defineEmits<{ close: [] }>()

const activeTab = ref('tailRepair')

const tabs = [
  {
    id: 'tailRepair',
    label: 'lazer面尾适配',
  },
  {
    id: 'batchGenerate',
    label: '批量生成图片',
  },
  {
    id: 'addScript',
    label: '为皮肤添加脚本',
  }
]
</script>

<style scoped>
.tool-overlay {
  position: fixed;
  inset: 0;
  z-index: 20000;
  background: rgba(4, 5, 10, 0.7);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  animation: overlayIn 0.2s ease-out;
}

@keyframes overlayIn {
  from {
    opacity: 0;
  }

  to {
    opacity: 1;
  }
}

.tool-panel {
  width: 680px;
  max-width: 92vw;
  height: 620px;
  max-height: 85vh;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: 14px;
  overflow: hidden;
  box-shadow:
    0 32px 80px rgba(0, 0, 0, 0.6),
    0 0 0 1px rgba(255, 255, 255, 0.03),
    inset 0 1px 0 rgba(255, 255, 255, 0.04);
  animation: panelIn 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  display: flex;
  flex-direction: column;
}

@keyframes panelIn {
  from {
    opacity: 0;
    transform: translateY(16px) scale(0.96);
  }

  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.panel-stripe {
  height: 3px;
  background: linear-gradient(90deg,
      var(--accent-purple) 0%,
      var(--accent-pink) 50%,
      var(--accent-purple) 100%);
  flex-shrink: 0;
}

.panel-inner {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* Header */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px 12px;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.tool-icon-wrap {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: linear-gradient(135deg, rgba(183, 108, 241, 0.15), rgba(255, 102, 170, 0.1));
  border: 1px solid rgba(183, 108, 241, 0.2);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent-purple);
}

.header-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.panel-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: 0.5px;
  margin: 0;
}

.panel-subtitle {
  font-size: 10px;
  color: var(--text-muted);
  letter-spacing: 1.5px;
  text-transform: uppercase;
  font-weight: 500;
}

.close-btn {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.close-btn:hover {
  background: var(--bg-surface);
  border-color: var(--border-color);
  color: var(--text-primary);
}

/* Tool Navigation */
.tool-nav {
  display: flex;
  padding: 0 16px;
  gap: 4px;
  flex-shrink: 0;
}

.nav-item {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px 16px;
  background: transparent;
  border: none;
  border-radius: 8px 8px 0 0;
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 500;
  font-family: inherit;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
}

.nav-item:hover {
  color: var(--text-secondary);
  background: rgba(183, 108, 241, 0.05);
}

.nav-item.active {
  color: var(--accent-purple);
  background: var(--bg-surface);
}

.nav-item.active .nav-indicator {
  position: absolute;
  bottom: 0;
  left: 20%;
  right: 20%;
  height: 2px;
  background: var(--accent-purple);
  border-radius: 1px 1px 0 0;
}

/* Tool Body - Fixed height container */
.tool-body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 20px;
  background: var(--bg-surface);
  border-top: 1px solid var(--border-color);
  border-bottom: 1px solid var(--border-color);
  min-height: 0;
}

.tool-body::-webkit-scrollbar {
  width: 6px;
}

.tool-body::-webkit-scrollbar-track {
  background: transparent;
}

.tool-body::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.08);
  border-radius: 3px;
}

.tool-body::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.15);
}

/* Tool switch transition */
.tool-switch-enter-active {
  animation: toolIn 0.25s ease-out;
}

.tool-switch-leave-active {
  animation: toolOut 0.15s ease-in;
}

@keyframes toolIn {
  from {
    opacity: 0;
    transform: translateX(12px);
  }

  to {
    opacity: 1;
    transform: translateX(0);
  }
}

@keyframes toolOut {
  from {
    opacity: 1;
    transform: translateX(0);
  }

  to {
    opacity: 0;
    transform: translateX(-12px);
  }
}
</style>
