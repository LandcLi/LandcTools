<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    placeholder?: string
    disabled?: boolean
    mono?: boolean
    clearable?: boolean
    type?: string
    /** 把输入值转成 number（等价于 v-model.number，但组件能正确处理） */
    number?: boolean
    min?: number
    max?: number
  }>(),
  {
    placeholder: '',
    disabled: false,
    mono: false,
    clearable: false,
    type: 'text',
    number: false,
  },
)

const model = defineModel<string | number>({ default: '' })
const emit = defineEmits<{ enter: [] }>()

const text = computed({
  get: () => (model.value == null ? '' : String(model.value)),
  set: (v: string) => {
    if (!props.number) {
      model.value = v
      return
    }
    if (v.trim() === '' || v === '-') {
      model.value = 0
      return
    }
    const n = Number(v)
    if (Number.isNaN(n)) {
      model.value = 0
      return
    }
    let out = n
    if (props.min != null && out < props.min) out = props.min
    if (props.max != null && out > props.max) out = props.max
    model.value = out
  },
})
</script>

<template>
  <div class="lc-input" :class="{ disabled, mono }">
    <span v-if="$slots.prefix" class="prefix"><slot name="prefix" /></span>
    <input
      v-model="text"
      :type="type"
      :placeholder="placeholder"
      :disabled="disabled"
      spellcheck="false"
      @keydown.enter="emit('enter')"
    />
    <button
      v-if="clearable && text"
      class="clear"
      type="button"
      title="清空"
      @click="model = ''"
    >
      ×
    </button>
    <span v-if="$slots.suffix" class="suffix"><slot name="suffix" /></span>
  </div>
</template>

<style scoped>
.lc-input {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 10px;
  background: var(--c-surface);
  border: 1px solid var(--c-border-strong);
  border-radius: var(--radius-sm);
  transition:
    border-color 0.14s ease,
    box-shadow 0.14s ease;
  min-width: 0;
}

.lc-input:focus-within {
  border-color: var(--c-primary);
  box-shadow: 0 0 0 3px var(--c-primary-ring);
}

.lc-input.disabled {
  opacity: 0.55;
}

input {
  flex: 1;
  min-width: 0;
  border: none;
  outline: none;
  background: transparent;
  font-size: 12.5px;
  color: var(--c-text);
}

input::placeholder {
  color: var(--c-text-muted);
}

.mono input {
  font-family: var(--font-mono);
}

/* 数字输入去掉上下箭头，界面更干净 */
input[type='number']::-webkit-inner-spin-button,
input[type='number']::-webkit-outer-spin-button {
  appearance: none;
  margin: 0;
}

.prefix,
.suffix {
  display: inline-flex;
  align-items: center;
  color: var(--c-text-muted);
  flex: none;
  font-size: 12px;
}

.clear {
  border: none;
  background: transparent;
  color: var(--c-text-muted);
  cursor: pointer;
  font-size: 15px;
  line-height: 1;
  padding: 0 2px;
  border-radius: 3px;
}

.clear:hover {
  color: var(--c-text);
  background: var(--c-surface-hover);
}
</style>
