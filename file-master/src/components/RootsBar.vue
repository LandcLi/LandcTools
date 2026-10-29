<script setup lang="ts">
import { computed } from 'vue'
import { useAppStore } from '@/stores/app'
import { FolderOpen, FolderPlus, X } from 'lucide-vue-next'

const store = useAppStore()

/** 每个根目录下当前载入了多少个文件，直接标在文件夹后面 */
const counts = computed(() => {
  const m = new Map<string, number>()
  for (const r of store.roots) {
    const rl = r.toLowerCase()
    let n = 0
    for (const f of store.files) {
      const d = f.dir.toLowerCase()
      if (d === rl || d.startsWith(rl + '\\') || d.startsWith(rl + '/')) n++
    }
    m.set(r, n)
  }
  return m
})
</script>

<template>
  <div v-if="store.roots.length" class="roots">
    <div class="label">
      <FolderOpen :size="13" />
      来源文件夹
    </div>

    <div class="chips">
      <div v-for="r in store.roots" :key="r" class="chip" :title="r">
        <span class="path">{{ r }}</span>
        <span class="cnt mono">{{ counts.get(r) ?? 0 }}</span>
        <button
          class="x"
          type="button"
          title="移除这个文件夹（只影响列表，不动磁盘文件）"
          :disabled="store.busy"
          @click="store.removeRoot(r)"
        >
          <X :size="11" />
        </button>
      </div>

      <button class="add" type="button" :disabled="store.busy" @click="store.pickDirs()">
        <FolderPlus :size="12" />
        添加
      </button>
    </div>

    <div class="hint">
      {{ store.hasSelection ? `已勾选 ${store.selectedCount} 个，本次只处理勾选项` : '未勾选时对所有文件生效' }}
    </div>
  </div>
</template>

<style scoped>
.roots {
  display: flex;
  align-items: center;
  gap: 10px;
  height: 36px;
  padding: 0 12px;
  background: var(--c-surface-2);
  border-bottom: 1px solid var(--c-border);
  flex: none;
  min-width: 0;
}

.label {
  display: flex;
  align-items: center;
  gap: 5px;
  flex: none;
  font-size: 11.5px;
  font-weight: 600;
  color: var(--c-text-muted);
  white-space: nowrap;
}

.chips {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
  overflow-x: auto;
  overflow-y: hidden;
  padding: 3px 0;
}

.chips::-webkit-scrollbar {
  height: 6px;
}

.chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex: none;
  max-width: 420px;
  height: 24px;
  padding: 0 4px 0 9px;
  border: 1px solid var(--c-border);
  border-radius: var(--radius-full);
  background: var(--c-surface);
  font-size: 11.5px;
  color: var(--c-text-secondary);
}

.chip:hover {
  border-color: color-mix(in srgb, var(--c-primary) 35%, var(--c-border));
}

.path {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--font-mono);
  font-size: 11px;
}

.cnt {
  flex: none;
  padding: 0 5px;
  border-radius: var(--radius-full);
  background: var(--c-surface-3, var(--c-surface-hover));
  color: var(--c-text-muted);
  font-size: 10px;
  font-weight: 700;
}

.x {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: none;
  width: 17px;
  height: 17px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--c-text-muted);
  cursor: pointer;
}

.x:hover {
  background: var(--c-danger-soft);
  color: var(--c-danger);
}

.x:disabled {
  opacity: 0.4;
  cursor: default;
}

.add {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  flex: none;
  height: 24px;
  padding: 0 9px;
  border: 1px dashed var(--c-border-strong);
  border-radius: var(--radius-full);
  background: transparent;
  color: var(--c-text-muted);
  font-size: 11.5px;
  cursor: pointer;
  white-space: nowrap;
}

.add:hover {
  border-color: var(--c-primary);
  color: var(--c-primary);
  background: var(--c-primary-soft);
}

.add:disabled {
  opacity: 0.5;
  cursor: default;
}

.hint {
  flex: none;
  font-size: 11px;
  color: var(--c-text-muted);
  white-space: nowrap;
}
</style>
