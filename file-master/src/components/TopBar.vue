<script setup lang="ts">
import { useAppStore } from '@/stores/app'
import LcButton from './ui/LcButton.vue'
import LcInput from './ui/LcInput.vue'
import { FolderPlus, RefreshCw, Trash2, Search, Sun, Moon, PanelLeft, Loader2 } from 'lucide-vue-next'
import logo from '@/assets/logo.png'

const store = useAppStore()
</script>

<template>
  <header class="topbar">
    <div class="brand">
      <img :src="logo" alt="" class="logo" draggable="false" />
      <div class="titles">
        <div class="name">文件大师</div>
        <div class="sub">FileMaster</div>
      </div>
    </div>

    <div class="divider-v" />

    <div class="actions">
      <LcButton variant="primary" :disabled="store.busy" @click="store.pickDirs()">
        <template #icon><FolderPlus :size="15" /></template>
        添加文件夹
      </LcButton>

      <LcButton
        variant="secondary"
        :disabled="!store.files.length || store.busy"
        title="重新扫描当前目录"
        @click="store.rescan()"
      >
        <template #icon>
          <Loader2 v-if="store.busy && store.phase === 'scan'" :size="15" class="spin" />
          <RefreshCw v-else :size="15" />
        </template>
        刷新
      </LcButton>

      <LcButton
        variant="ghost"
        :disabled="!store.files.length || store.busy"
        title="清空文件列表"
        @click="store.clearAll()"
      >
        <template #icon><Trash2 :size="15" /></template>
      </LcButton>
    </div>

    <div class="grow" />

    <div class="right">
      <LcInput v-model="store.query" placeholder="搜索文件名…" class="search" clearable>
        <template #prefix><Search :size="13" /></template>
      </LcInput>

      <button
        class="icon-btn"
        :title="store.theme === 'dark' ? '切换到浅色' : '切换到深色'"
        @click="store.setTheme(store.theme === 'dark' ? 'light' : 'dark')"
      >
        <Sun v-if="store.theme === 'dark'" :size="16" />
        <Moon v-else :size="16" />
      </button>

      <button
        class="icon-btn"
        :class="{ active: store.sidebarOpen }"
        title="显示/隐藏工具栏"
        @click="store.sidebarOpen = !store.sidebarOpen"
      >
        <PanelLeft :size="16" />
      </button>
    </div>
  </header>
</template>

<style scoped>
.topbar {
  display: flex;
  align-items: center;
  gap: 12px;
  height: var(--header-h);
  padding: 0 14px;
  background: var(--c-surface);
  border-bottom: 1px solid var(--c-border);
  flex: none;
  -webkit-app-region: drag;
}

/* 可交互元素不能吃掉窗口拖拽区域 */
.topbar button,
.topbar input,
.topbar .search {
  -webkit-app-region: no-drag;
}

.brand {
  display: flex;
  align-items: center;
  gap: 9px;
  flex: none;
}

.logo {
  width: 26px;
  height: 26px;
  border-radius: 6px;
  pointer-events: none;
}

.titles {
  display: flex;
  flex-direction: column;
  line-height: 1.15;
}

.name {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.01em;
}

.sub {
  font-size: 9px;
  font-weight: 600;
  letter-spacing: 0.09em;
  color: var(--c-text-muted);
  text-transform: uppercase;
}

.actions {
  display: flex;
  align-items: center;
  gap: 7px;
}

.right {
  display: flex;
  align-items: center;
  gap: 7px;
}

.search {
  width: 200px;
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: 1px solid transparent;
  background: transparent;
  border-radius: var(--radius-sm);
  color: var(--c-text-secondary);
  cursor: pointer;
  transition:
    background 0.14s ease,
    color 0.14s ease;
}

.icon-btn:hover {
  background: var(--c-surface-hover);
  color: var(--c-text);
}

.icon-btn.active {
  background: var(--c-primary-soft);
  color: var(--c-primary);
}

.spin {
  animation: spin 0.9s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
