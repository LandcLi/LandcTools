<script setup lang="ts">
import { computed, ref } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { useAppStore } from '@/stores/app'
import KindBadge from './ui/KindBadge.vue'
import LcCheckbox from './ui/LcCheckbox.vue'
import { formatSize, formatTime, diffParts } from '@/utils/format'
import { ArrowRight, AlertCircle, AlertTriangle, FolderSearch } from 'lucide-vue-next'
import type { FileEntry, PlanItem } from '@/api/types'

const store = useAppStore()

interface Diff {
  prefix: string
  changed: string
  suffix: string
}

interface Row {
  f: FileEntry
  p: PlanItem | undefined
  changed: boolean
  d: Diff | null
}

const planMap = computed(() => {
  const m = new Map<string, PlanItem>()
  for (const it of store.plan?.items ?? []) m.set(it.id, it)
  return m
})

const rows = computed<Row[]>(() =>
  store.filtered.map((f) => {
    const p = planMap.value.get(f.id)
    const changed = !!p?.changed
    return { f, p, changed, d: changed && p ? diffParts(f.name, p.newName) : null }
  }),
)

const scrollRef = ref<HTMLElement | null>(null)

const virtualizer = useVirtualizer(
  computed(() => ({
    count: rows.value.length,
    getScrollElement: () => scrollRef.value,
    estimateSize: () => 34,
    overscan: 14,
  })),
)

/** 把虚拟项和真实行数据绑在一起，避免模板里反复做越界索引 */
const renderRows = computed(() => {
  const out: { key: string; start: number; row: Row }[] = []
  for (const vi of virtualizer.value.getVirtualItems()) {
    const row = rows.value[vi.index]
    if (row) out.push({ key: row.f.id, start: vi.start, row })
  }
  return out
})

const totalSize = computed(() => virtualizer.value.getTotalSize())

const allSelected = computed(
  () => store.filtered.length > 0 && store.selectedCount === store.filtered.length,
)

async function reveal(path: string) {
  try {
    await revealItemInDir(path)
  } catch {
    /* 资源管理器打不开就算了，不弹窗打扰 */
  }
}

function isSelected(id: string): boolean {
  return store.selected.has(id)
}
</script>

<template>
  <section class="table-wrap">
    <div class="thead">
      <div class="cell-check">
        <LcCheckbox
          :model-value="allSelected"
          :disabled="!store.filtered.length"
          @update:model-value="store.selectAll()"
        />
      </div>
      <div class="cell-kind">类型</div>
      <div class="cell-name">原文件名</div>
      <div class="cell-arrow" />
      <div class="cell-preview">预览结果</div>
      <div class="cell-size">大小</div>
      <div class="cell-time">修改时间</div>
      <div class="cell-flag" />
    </div>

    <div v-if="!store.files.length" class="empty">
      <div class="empty-art">
        <FolderSearch :size="30" />
      </div>
      <div class="empty-title">还没有载入文件</div>
      <div class="empty-desc">
        点击左上角「添加文件夹」，选择要批量处理的目录<br />
        所有操作都会先预览、再执行，改错了可以一键撤销
      </div>
    </div>

    <div v-else ref="scrollRef" class="tbody">
      <div class="phantom" :style="{ height: `${totalSize}px` }">
        <div
          v-for="it in renderRows"
          :key="it.key"
          class="tr"
          :class="{
            sel: isSelected(it.row.f.id),
            err: it.row.p?.level === 'error',
            warn: it.row.p?.level === 'warn',
          }"
          :style="{ transform: `translateY(${it.start}px)` }"
          @click="store.toggleSelect(it.row.f.id)"
          @dblclick="reveal(it.row.f.path)"
        >
          <div class="cell-check" @click.stop>
            <LcCheckbox
              :model-value="isSelected(it.row.f.id)"
              @update:model-value="store.toggleSelect(it.row.f.id)"
            />
          </div>

          <div class="cell-kind">
            <KindBadge :kind="it.row.f.kind" :ext="it.row.f.ext" />
          </div>

          <div class="cell-name" :title="it.row.f.path">
            <span class="nm">{{ it.row.f.name }}</span>
            <span v-if="it.row.f.isDir" class="tag-dir">目录</span>
          </div>

          <div class="cell-arrow">
            <ArrowRight v-if="it.row.changed" :size="11" />
          </div>

          <div class="cell-preview">
            <template v-if="it.row.changed && it.row.d">
              <span class="dim">{{ it.row.d.prefix }}</span>
              <span class="hit">{{ it.row.d.changed }}</span>
              <span class="dim">{{ it.row.d.suffix }}</span>
            </template>
            <span v-else-if="it.row.p?.level === 'warn'" class="dim">
              {{ it.row.p.issues[0] }}
            </span>
            <span v-else class="dash">—</span>
          </div>

          <div class="cell-size mono">
            {{ it.row.f.isDir ? '' : formatSize(it.row.f.size) }}
          </div>

          <div class="cell-time mono">{{ formatTime(it.row.f.modifiedMs) }}</div>

          <div class="cell-flag">
            <span
              v-if="it.row.p && it.row.p.level !== 'ok'"
              class="flag"
              :class="it.row.p.level"
              :title="it.row.p.issues.join('\n')"
            >
              <AlertCircle v-if="it.row.p.level === 'error'" :size="13" />
              <AlertTriangle v-else :size="13" />
            </span>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.table-wrap {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  background: var(--c-surface);
  overflow: hidden;
}

.thead {
  display: grid;
  grid-template-columns: 34px 44px minmax(120px, 1fr) 18px minmax(120px, 1fr) 76px 120px 24px;
  align-items: center;
  gap: 8px;
  height: 34px;
  padding: 0 12px;
  background: var(--c-surface-2);
  border-bottom: 1px solid var(--c-border);
  font-size: 11.5px;
  font-weight: 600;
  color: var(--c-text-muted);
  flex: none;
  user-select: none;
}

.tbody {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.phantom {
  position: relative;
  width: 100%;
}

.tr {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  display: grid;
  grid-template-columns: 34px 44px minmax(120px, 1fr) 18px minmax(120px, 1fr) 76px 120px 24px;
  align-items: center;
  gap: 8px;
  height: 34px;
  padding: 0 12px;
  border-bottom: 1px solid color-mix(in srgb, var(--c-border) 55%, transparent);
  cursor: default;
  font-size: 12.5px;
  will-change: transform;
}

.tr:hover {
  background: var(--c-surface-hover);
}

.tr.sel {
  background: var(--c-primary-soft);
}

.tr.sel:hover {
  background: color-mix(in srgb, var(--c-primary) 14%, var(--c-surface));
}

.tr.err {
  box-shadow: inset 3px 0 0 var(--c-danger);
}

.tr.warn {
  box-shadow: inset 3px 0 0 var(--c-warning);
}

.cell-check,
.cell-kind,
.cell-arrow,
.cell-flag {
  display: flex;
  align-items: center;
  justify-content: center;
}

.cell-name {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.nm {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--c-text);
}

.tag-dir {
  flex: none;
  font-size: 9.5px;
  padding: 0 4px;
  border-radius: 3px;
  background: var(--c-warning-soft);
  color: var(--c-warning);
  font-weight: 700;
}

.cell-arrow {
  color: var(--c-text-muted);
}

.cell-preview {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12.5px;
}

.dim {
  color: var(--c-text-muted);
}

.hit {
  color: var(--c-primary);
  font-weight: 600;
  background: var(--c-primary-soft);
  border-radius: 3px;
  padding: 0 1px;
}

.dash {
  color: var(--c-text-muted);
  opacity: 0.5;
}

.cell-size {
  text-align: right;
  font-size: 11.5px;
  color: var(--c-text-secondary);
}

.cell-time {
  font-size: 11.5px;
  color: var(--c-text-muted);
}

.flag {
  display: inline-flex;
  align-items: center;
}

.flag.error {
  color: var(--c-danger);
}

.flag.warn {
  color: var(--c-warning);
}

.empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding-bottom: 60px;
}

.empty-art {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 62px;
  height: 62px;
  border-radius: 50%;
  background: var(--c-surface-2);
  border: 1px solid var(--c-border);
  color: var(--c-text-muted);
  margin-bottom: 6px;
}

.empty-title {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--c-text-secondary);
}

.empty-desc {
  font-size: 12px;
  line-height: 1.75;
  color: var(--c-text-muted);
  text-align: center;
}
</style>
