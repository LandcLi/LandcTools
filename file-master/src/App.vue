<script setup lang="ts">
import { onMounted } from 'vue'
import { useAppStore } from '@/stores/app'
import { isMock } from '@/api'
import TopBar from '@/components/TopBar.vue'
import Sidebar from '@/components/Sidebar.vue'
import RootsBar from '@/components/RootsBar.vue'
import FileTable from '@/components/FileTable.vue'
import StatusBar from '@/components/StatusBar.vue'

const store = useAppStore()

onMounted(() => {
  store.initTheme()
  void store.loadJournals()
  // 演示模式直接铺好示例数据，省掉「先点添加文件夹」这一步
  if (isMock) void store.pickDirs()
})
</script>

<template>
  <div class="app">
    <TopBar />
    <main class="main">
      <Sidebar v-show="store.sidebarOpen" />
      <div class="content">
        <RootsBar />
        <FileTable />
      </div>
    </main>
    <StatusBar />

    <Transition name="toast">
      <div v-if="store.toast" class="toast" :class="store.toast.type">
        {{ store.toast.text }}
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--c-bg);
}

.main {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.content {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.toast {
  position: fixed;
  left: 50%;
  bottom: calc(var(--status-h) + 18px);
  transform: translateX(-50%);
  max-width: 620px;
  padding: 9px 16px;
  border-radius: var(--radius-md);
  font-size: 12.5px;
  line-height: 1.5;
  box-shadow: var(--shadow-lg);
  z-index: 90;
  border: 1px solid var(--c-border);
  background: var(--c-surface);
  color: var(--c-text);
  pointer-events: none;
}

.toast.ok {
  border-color: color-mix(in srgb, var(--c-success) 38%, transparent);
  background: var(--c-success-soft);
  color: var(--c-success);
}

.toast.err {
  border-color: color-mix(in srgb, var(--c-danger) 38%, transparent);
  background: var(--c-danger-soft);
  color: var(--c-danger);
}

.toast.info {
  border-color: color-mix(in srgb, var(--c-info) 35%, transparent);
  background: var(--c-info-soft);
  color: var(--c-info);
}

.toast-enter-active,
.toast-leave-active {
  transition:
    opacity 0.18s ease,
    transform 0.18s ease;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(8px);
}
</style>
