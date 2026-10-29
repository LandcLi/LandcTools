<script setup lang="ts">
import { onMounted } from 'vue'
import { useAppStore } from '@/stores/app'
import LcButton from '../ui/LcButton.vue'
import { History, Undo2, RefreshCw } from 'lucide-vue-next'
import { formatTime } from '@/utils/format'

const store = useAppStore()

const kindLabel: Record<string, string> = {
  rename: '改名',
  times: '时间',
  attrs: '属性',
  ext: '后缀',
}

onMounted(() => {
  if (!store.journals.length) void store.loadJournals()
})
</script>

<template>
  <div class="panel fade-in">
    <div class="panel-head">
      <span class="panel-name">撤销历史</span>
      <LcButton variant="ghost" size="sm" title="刷新" @click="store.loadJournals()">
        <template #icon><RefreshCw :size="13" /></template>
      </LcButton>
    </div>

    <section v-if="!store.journals.length" class="section">
      <div class="empty">
        <History :size="22" />
        <div class="t">还没有操作记录</div>
        <div class="d">每次批量执行后都会自动记录，可以在这里一键还原</div>
      </div>
    </section>

    <section v-else class="list">
      <div
        v-for="j in store.journals"
        :key="j.id"
        class="item"
        :class="{ undone: j.undone }"
      >
        <div class="main">
          <div class="title-row">
            <span class="kind">{{ kindLabel[j.kind] || j.kind }}</span>
            <span class="title">{{ j.title }}</span>
          </div>
          <div class="meta">{{ formatTime(j.createdAt, true) }}</div>
        </div>
        <LcButton
          v-if="!j.undone"
          variant="secondary"
          size="sm"
          :disabled="store.busy"
          @click="store.undo(j.id)"
        >
          <template #icon><Undo2 :size="12" /></template>
          撤销
        </LcButton>
        <span v-else class="done-tag">已撤销</span>
      </div>
    </section>

    <section class="section">
      <div class="field-hint">
        撤销会把文件恢复到操作前的状态。如果操作后文件被移动或删除，对应条目会跳过并提示失败。
      </div>
    </section>
  </div>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
}

.panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px 4px;
}

.panel-name {
  font-size: 13px;
  font-weight: 700;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 28px 0;
  color: var(--c-text-muted);
}

.empty .t {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--c-text-secondary);
  margin-top: 4px;
}

.empty .d {
  font-size: 11.5px;
  text-align: center;
  line-height: 1.6;
  max-width: 240px;
}

.list {
  display: flex;
  flex-direction: column;
}

.item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 16px;
  border-bottom: 1px solid var(--c-border);
}

.item.undone {
  opacity: 0.55;
}

.main {
  flex: 1;
  min-width: 0;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.kind {
  flex: none;
  font-size: 9.5px;
  font-weight: 700;
  padding: 1px 5px;
  border-radius: 3px;
  background: var(--c-primary-soft);
  color: var(--c-primary);
}

.title {
  font-size: 12.5px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.meta {
  font-size: 10.5px;
  color: var(--c-text-muted);
  margin-top: 2px;
  font-family: var(--font-mono);
}

.done-tag {
  font-size: 11px;
  color: var(--c-text-muted);
  flex: none;
}
</style>
