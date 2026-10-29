<script setup lang="ts">
defineProps<{
  options: { label: string; value: string }[]
  disabled?: boolean
  compact?: boolean
}>()

const model = defineModel<string>({ default: '' })
</script>

<template>
  <div class="seg" :class="{ compact }">
    <button
      v-for="o in options"
      :key="o.value"
      type="button"
      :class="{ on: model === o.value }"
      :disabled="disabled"
      @click="model = o.value"
    >
      {{ o.label }}
    </button>
  </div>
</template>

<style scoped>
.seg {
  display: inline-flex;
  padding: 2px;
  gap: 2px;
  background: var(--c-surface-2);
  border: 1px solid var(--c-border);
  border-radius: var(--radius-sm);
  height: 32px;
  min-width: 0;
}

.seg.compact {
  height: 27px;
}

button {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  color: var(--c-text-secondary);
  font-size: 12px;
  border-radius: 5px;
  cursor: pointer;
  padding: 0 9px;
  white-space: nowrap;
  transition:
    background 0.13s ease,
    color 0.13s ease;
}

button:hover:not(:disabled):not(.on) {
  color: var(--c-text);
  background: var(--c-surface-hover);
}

button.on {
  background: var(--c-surface);
  color: var(--c-primary);
  font-weight: 600;
  box-shadow: var(--shadow-xs);
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
