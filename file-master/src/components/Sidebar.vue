<script setup lang="ts">
import { useAppStore, type PanelKey } from '@/stores/app'
import RenamePanel from './panels/RenamePanel.vue'
import TimePanel from './panels/TimePanel.vue'
import AttrPanel from './panels/AttrPanel.vue'
import ExtPanel from './panels/ExtPanel.vue'
import MetaPanel from './panels/MetaPanel.vue'
import HistoryPanel from './panels/HistoryPanel.vue'
import LcButton from './ui/LcButton.vue'
import {
  Type,
  Clock,
  SlidersHorizontal,
  FileType2,
  Tags,
  History,
  Rocket,
  AlertTriangle,
} from 'lucide-vue-next'
import type { Component } from 'vue'

const store = useAppStore()

const tabs: { key: PanelKey; label: string; icon: Component }[] = [
  { key: 'rename', label: '批量改名', icon: Type },
  { key: 'time', label: '时间属性', icon: Clock },
  { key: 'attr', label: '文件属性', icon: SlidersHorizontal },
  { key: 'ext', label: '后缀转换', icon: FileType2 },
  { key: 'meta', label: '元数据', icon: Tags },
  { key: 'history', label: '撤销历史', icon: History },
]
</script>

<template>
  <aside class="sidebar">
    <nav class="tabs">
      <button
        v-for="t in tabs"
        :key="t.key"
        type="button"
        :class="{ on: store.panel === t.key }"
        @click="store.panel = t.key"
      >
        <component :is="t.icon" :size="15" />
        <span>{{ t.label }}</span>
      </button>
    </nav>

    <div class="body">
      <RenamePanel v-show="store.panel === 'rename'" />
      <TimePanel v-show="store.panel === 'time'" />
      <AttrPanel v-show="store.panel === 'attr'" />
      <ExtPanel v-show="store.panel === 'ext'" />
      <MetaPanel v-show="store.panel === 'meta'" />
      <HistoryPanel v-show="store.panel === 'history'" />
    </div>

    <!-- 执行区固定在底部：改规则和点执行在同一个视觉流里，不用再往右下角找 -->
    <footer v-if="store.panel === 'rename'" class="exec">
      <div class="exec-head">
        <span class="exec-scope" :class="{ on: store.hasSelection }" :title="store.scopeLabel">
          {{ store.scopeLabel }}
        </span>
        <span v-if="store.errorCount > 0" class="exec-bad">
          <AlertTriangle :size="11" />
          {{ store.errorCount }} 个冲突
        </span>
        <span v-else-if="store.changedCount > 0" class="exec-ok">
          将改动 {{ store.changedCount }} 个
        </span>
        <span v-else class="exec-dim">无改动</span>
      </div>

      <LcButton
        variant="primary"
        size="lg"
        block
        :disabled="!store.canExecuteRename"
        title="按预览结果执行改名（可撤销）"
        @click="store.executeRename()"
      >
        <template #icon><Rocket :size="15" /></template>
        立即改名
        <span v-if="store.changedCount > 0" class="pill">{{ store.changedCount }}</span>
      </LcButton>

      <div v-if="store.plan?.ruleError" class="exec-err">{{ store.plan.ruleError }}</div>
    </footer>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-w);
  flex: none;
  display: flex;
  flex-direction: column;
  background: var(--c-surface);
  border-right: 1px solid var(--c-border);
  overflow: hidden;
}

.tabs {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 4px;
  padding: 8px;
  border-bottom: 1px solid var(--c-border);
  flex: none;
}

.tabs button {
  display: flex;
  align-items: center;
  gap: 7px;
  height: 32px;
  padding: 0 9px;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--c-text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition:
    background 0.13s ease,
    color 0.13s ease,
    border-color 0.13s ease;
  white-space: nowrap;
  overflow: hidden;
}

.tabs button:hover {
  background: var(--c-surface-hover);
  color: var(--c-text);
}

.tabs button.on {
  background: var(--c-primary-soft);
  border-color: color-mix(in srgb, var(--c-primary) 22%, transparent);
  color: var(--c-primary);
  font-weight: 600;
}

.body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

/* ---------- 底部执行区 ---------- */
.exec {
  flex: none;
  display: flex;
  flex-direction: column;
  gap: 7px;
  padding: 10px 12px 12px;
  border-top: 1px solid var(--c-border);
  background: var(--c-surface);
  box-shadow: 0 -6px 14px -10px rgba(0, 0, 0, 0.18);
}

.exec-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-size: 11.5px;
  min-width: 0;
}

.exec-scope {
  color: var(--c-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 有勾选时高亮：明确告诉用户「这次不是全部」 */
.exec-scope.on {
  color: var(--c-primary);
  font-weight: 600;
}

.exec-ok {
  flex: none;
  color: var(--c-success);
  font-weight: 600;
}

.exec-bad {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  flex: none;
  color: var(--c-danger);
  font-weight: 600;
}

.exec-dim {
  flex: none;
  color: var(--c-text-muted);
}

.pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 17px;
  padding: 0 5px;
  border-radius: var(--radius-full);
  background: rgba(255, 255, 255, 0.24);
  font-size: 10.5px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.exec-err {
  font-size: 11px;
  color: var(--c-danger);
  line-height: 1.45;
}
</style>
