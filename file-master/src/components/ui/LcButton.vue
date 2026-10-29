<script setup lang="ts">
withDefaults(
  defineProps<{
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger' | 'soft'
    size?: 'sm' | 'md' | 'lg'
    disabled?: boolean
    loading?: boolean
    block?: boolean
    iconOnly?: boolean
    title?: string
  }>(),
  {
    variant: 'secondary',
    size: 'md',
    disabled: false,
    loading: false,
    block: false,
    iconOnly: false,
  },
)
</script>

<template>
  <button
    class="lc-btn"
    :class="[`v-${variant}`, `s-${size}`, { block, 'icon-only': iconOnly }]"
    :disabled="disabled || loading"
    :title="title"
    type="button"
  >
    <span v-if="loading" class="spinner" aria-hidden="true" />
    <slot v-else name="icon" />
    <span v-if="!iconOnly" class="label"><slot /></span>
  </button>
</template>

<style scoped>
.lc-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-weight: 500;
  white-space: nowrap;
  transition:
    background 0.14s ease,
    border-color 0.14s ease,
    color 0.14s ease,
    box-shadow 0.14s ease,
    transform 0.06s ease;
  flex: none;
}

.lc-btn:active:not(:disabled) {
  transform: translateY(0.5px);
}

.lc-btn:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.block {
  display: flex;
  width: 100%;
}

.icon-only {
  padding: 0;
  width: 32px;
}

.s-sm {
  height: 27px;
  padding: 0 9px;
  font-size: 12px;
}
.s-sm.icon-only {
  width: 27px;
}

.s-md {
  height: 32px;
  padding: 0 12px;
  font-size: 12.5px;
}

.s-lg {
  height: 38px;
  padding: 0 18px;
  font-size: 13px;
}

.v-primary {
  background: var(--c-primary);
  color: #fff;
  box-shadow: var(--shadow-xs);
}
.v-primary:hover:not(:disabled) {
  background: var(--c-primary-hover);
}
.v-primary:active:not(:disabled) {
  background: var(--c-primary-active);
}

.v-secondary {
  background: var(--c-surface);
  border-color: var(--c-border-strong);
  color: var(--c-text);
}
.v-secondary:hover:not(:disabled) {
  background: var(--c-surface-hover);
  border-color: var(--c-text-muted);
}

.v-ghost {
  background: transparent;
  color: var(--c-text-secondary);
}
.v-ghost:hover:not(:disabled) {
  background: var(--c-surface-hover);
  color: var(--c-text);
}

.v-soft {
  background: var(--c-primary-soft);
  color: var(--c-primary);
}
.v-soft:hover:not(:disabled) {
  background: var(--c-primary-soft);
  filter: brightness(0.96);
}

.v-danger {
  background: var(--c-danger);
  color: #fff;
}
.v-danger:hover:not(:disabled) {
  filter: brightness(1.06);
}

.spinner {
  width: 12px;
  height: 12px;
  border: 1.6px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  opacity: 0.85;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.label {
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
