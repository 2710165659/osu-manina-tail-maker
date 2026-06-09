<template>
  <Transition name="revert">
    <button v-if="visible" class="revert-btn" title="恢复默认值" @click="$emit('revert')">
      <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"
        stroke-linejoin="round">
        <path d="M2 7a5 5 0 0 1 9-3M12 7a5 5 0 0 1-9 3" />
        <path d="M2 2v3.5h3.5M12 12V8.5H8.5" />
      </svg>
    </button>
  </Transition>
</template>

<script setup lang="ts">
defineProps<{
  visible: boolean
}>()

defineEmits<{
  revert: []
}>()
</script>

<style scoped>
.revert-enter-active {
  animation: revertIn 0.2s ease-out;
}

.revert-leave-active {
  animation: revertIn 0.12s ease-in reverse;
}

@keyframes revertIn {
  from {
    opacity: 0;
    transform: translateX(8px) scale(0.8);
  }

  to {
    opacity: 1;
    transform: translateX(0) scale(1);
  }
}
</style>
