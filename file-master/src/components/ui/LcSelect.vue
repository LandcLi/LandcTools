<script setup lang="ts">
defineProps<{
  options: { label: string; value: string }[]
  disabled?: boolean
}>()

const model = defineModel<string>({ default: '' })
</script>

<template>
  <div class="lc-select">
    <select v-model="model" :disabled="disabled">
      <option v-for="o in options" :key="o.value" :value="o.value">{{ o.label }}</option>
    </select>
    <svg class="chev" viewBox="0 0 12 12" width="12" height="12" aria-hidden="true">
      <path
        d="M2.5 4.5L6 8l3.5-3.5"
        fill="none"
        stroke="currentColor"
        stroke-width="1.6"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </div>
</template>

<style scoped>
.lc-select {
  position: relative;
  display: flex;
  align-items: center;
  height: 32px;
  min-width: 0;
}

select {
  appearance: none;
  width: 100%;
  height: 100%;
  padding: 0 26px 0 10px;
  background: var(--c-surface);
  border: 1px solid var(--c-border-strong);
  border-radius: var(--radius-sm);
  font-size: 12.5px;
  color: var(--c-text);
  cursor: pointer;
  outline: none;
  transition:
    border-color 0.14s ease,
    box-shadow 0.14s ease;
  text-overflow: ellipsis;
}

select:hover:not(:disabled) {
  border-color: var(--c-text-muted);
}

select:focus {
  border-color: var(--c-primary);
  box-shadow: 0 0 0 3px var(--c-primary-ring);
}

select:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

/* 下拉展开项在不同浏览器/WebView 下样式有限，这里保证深色下不出现白底黑字割裂 */
option {
  background: var(--c-surface);
  color: var(--c-text);
}

.chev {
  position: absolute;
  right: 9px;
  pointer-events: none;
  color: var(--c-text-muted);
}
</style>
