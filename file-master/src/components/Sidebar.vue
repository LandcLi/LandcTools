<script setup lang="ts">
import { useAppStore, type PanelKey } from '@/stores/app'
import RenamePanel from './panels/RenamePanel.vue'
import TimePanel from './panels/TimePanel.vue'
import AttrPanel from './panels/AttrPanel.vue'
import ExtPanel from './panels/ExtPanel.vue'
import MetaPanel from './panels/MetaPanel.vue'
import HistoryPanel from './panels/HistoryPanel.vue'
import { Type, Clock, SlidersHorizontal, FileType2, Tags, History } from 'lucide-vue-next'
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
</style>
