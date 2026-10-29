<script setup lang="ts">
defineProps<{ disabled?: boolean; label?: string }>()
const model = defineModel<boolean>({ default: false })
</script>

<template>
  <label class="lc-switch" :class="{ on: model, disabled }">
    <button
      type="button"
      class="track"
      role="switch"
      :aria-checked="model"
      :disabled="disabled"
      @click="model = !model"
    >
      <span class="knob" />
    </button>
    <span v-if="label" class="text">{{ label }}</span>
  </label>
</template>

<style scoped>
.lc-switch {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.lc-switch.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.track {
  position: relative;
  width: 32px;
  height: 18px;
  flex: none;
  border: none;
  border-radius: var(--radius-full);
  background: var(--c-border-strong);
  cursor: inherit;
  padding: 0;
  transition: background 0.16s ease;
}

.track:disabled {
  cursor: inherit;
}

.lc-switch.on .track {
  background: var(--c-primary);
}

.knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.25);
  transition: transform 0.16s cubic-bezier(0.4, 0, 0.2, 1);
}

.lc-switch.on .knob {
  transform: translateX(14px);
}

.text {
  font-size: 12.5px;
  color: var(--c-text);
  user-select: none;
}
</style>
