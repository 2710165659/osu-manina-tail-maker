<template>
  <Teleport to="body">
    <div class="toast-container">
      <TransitionGroup name="toast">
        <div v-for="n in notifications" :key="n.id" class="toast" :class="`toast-${n.type}`">
          {{ n.message }}
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { useNotification } from '../composables/useNotification'

const { notifications } = useNotification()
</script>

<style scoped>
.toast-container {
  position: fixed;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 10000;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  pointer-events: none;
}

.toast {
  padding: 7px 18px;
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  pointer-events: auto;
  backdrop-filter: blur(8px);
}

.toast-info {
  background: rgba(39, 29, 53, 0.8);
  color: var(--text-secondary);
  border: 1px solid rgba(58, 45, 80, 0.5);
}

.toast-success {
  background: rgba(30, 40, 22, 0.8);
  color: rgba(136, 179, 0, 0.85);
  border: 1px solid rgba(136, 179, 0, 0.15);
}

.toast-error {
  background: rgba(45, 22, 30, 0.8);
  color: rgba(255, 102, 102, 0.85);
  border: 1px solid rgba(255, 102, 102, 0.15);
}

/* Transitions */
.toast-enter-active {
  transition: all 0.25s ease-out;
}

.toast-leave-active {
  transition: all 0.25s ease-in;
  position: absolute;
}

.toast-move {
  transition: transform 0.25s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateY(-10px);
}

.toast-leave-to {
  opacity: 0;
}
</style>
