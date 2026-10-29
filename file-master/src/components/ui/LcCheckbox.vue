<script setup lang="ts">
defineProps<{ disabled?: boolean; label?: string; indeterminate?: boolean }>()
const model = defineModel<boolean>({ default: false })
</script>

<template>
  <label class="lc-check" :class="{ disabled }">
    <input v-model="model" type="checkbox" :disabled="disabled" />
    <span class="box" :class="{ checked: model }">
      <svg viewBox="0 0 12 12" width="10" height="10" aria-hidden="true">
        <path
          v-if="model"
          d="M2 6.2l2.6 2.6L10 3.4"
          fill="none"
          stroke="#fff"
          stroke-width="1.9"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </span>
    <span v-if="label" class="text">{{ label }}</span>
  </label>
</template>

<style scoped>
.lc-check {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  cursor: pointer;
  min-width: 0;
}

.lc-check.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}

.box {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 15px;
  height: 15px;
  flex: none;
  border: 1.4px solid var(--c-border-strong);
  border-radius: 4px;
  background: var(--c-surface);
  transition:
    background 0.13s ease,
    border-color 0.13s ease;
}

.lc-check:hover .box {
  border-color: var(--c-primary);
}

.box.checked {
  background: var(--c-primary);
  border-color: var(--c-primary);
}

input:focus-visible + .box {
  outline: 2px solid var(--c-primary);
  outline-offset: 2px;
}

.text {
  font-size: 12.5px;
  color: var(--c-text);
  user-select: none;
}
</style>
