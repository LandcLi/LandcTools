<script setup lang="ts">
defineProps<{ percent: number; active?: boolean; height?: number }>()
</script>

<template>
  <div class="track" :style="{ height: `${height ?? 3}px` }">
    <div class="bar" :class="{ active }" :style="{ width: `${Math.min(100, Math.max(0, percent))}%` }" />
  </div>
</template>

<style scoped>
.track {
  width: 100%;
  background: var(--c-border);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.bar {
  height: 100%;
  background: var(--c-primary);
  border-radius: var(--radius-full);
  transition: width 0.16s ease-out;
}

.bar.active {
  background: linear-gradient(90deg, var(--c-primary), #8b5cf6);
  background-size: 200% 100%;
  animation: shimmer 1.1s linear infinite;
}

@keyframes shimmer {
  to {
    background-position: -200% 0;
  }
}
</style>
