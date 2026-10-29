<script setup lang="ts">
import { computed } from 'vue'
import { KIND_LABEL } from '@/utils/format'

const props = defineProps<{ kind: string; ext?: string }>()

const color = computed(() => `var(--kind-${props.kind || 'other'})`)
const text = computed(() => {
  if (props.kind === 'folder') return 'DIR'
  return (props.ext || '').slice(0, 4).toUpperCase() || '—'
})
const title = computed(() => KIND_LABEL[props.kind] || '其他')
</script>

<template>
  <span class="badge" :style="{ color, backgroundColor: `color-mix(in srgb, ${color} 12%, transparent)` }" :title="title">
    {{ text }}
  </span>
</template>

<style scoped>
.badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 34px;
  height: 17px;
  padding: 0 4px;
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: 9.5px;
  font-weight: 700;
  letter-spacing: 0.03em;
  flex: none;
}
</style>
