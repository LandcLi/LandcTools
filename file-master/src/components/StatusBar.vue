<script setup lang="ts">
import { computed } from 'vue'
import { useAppStore } from '@/stores/app'
import LcProgress from './ui/LcProgress.vue'
import { AlertTriangle, CheckCircle2, CircleSlash, ListChecks, Layers } from 'lucide-vue-next'

const store = useAppStore()

const phaseText = computed(() => {
  switch (store.phase) {
    case 'scan':
      return '扫描中'
    case 'rename':
      return '改名中'
    case 'times':
      return '写入时间'
    case 'attrs':
      return '写入属性'
    case 'undo':
      return '撤销中'
    default:
      return '处理中'
  }
})

const readyText = computed(() => {
  if (!store.files.length) return '尚未载入文件'
  if (store.plan?.ruleError) return store.plan.ruleError
  if (store.errorCount > 0) return `${store.errorCount} 个文件存在冲突，无法执行`
  if (store.changedCount === 0) return '规则未产生任何改动'
  return `将改动 ${store.changedCount} 个文件`
})
</script>

<template>
  <footer class="statusbar">
    <div class="left">
      <span class="stat">
        <b>{{ store.files.length }}</b> 个条目
      </span>
      <span v-if="store.query && store.visibleCount !== store.files.length" class="stat">
        匹配 <b>{{ store.visibleCount }}</b>
      </span>
      <span v-if="store.selectedCount" class="stat sel">
        已选 <b>{{ store.selectedCount }}</b>
      </span>
      <span v-if="store.warnCount" class="stat warn">
        <AlertTriangle :size="12" />
        {{ store.warnCount }} 个提醒
      </span>
    </div>

    <div class="mid">
      <template v-if="store.busy">
        <span class="phase">{{ phaseText }}</span>
        <LcProgress :percent="store.progressPercent" active :height="4" />
        <span class="mono count">
          {{ store.progress.done }}<template v-if="store.progress.total">/{{ store.progress.total }}</template>
        </span>
        <button class="stop-btn" @click="store.cancel()">停止</button>
      </template>
      <template v-else-if="store.files.length">
        <span class="ready" :class="{ bad: store.errorCount > 0 }">
          <AlertTriangle v-if="store.errorCount > 0" :size="12" />
          <CheckCircle2 v-else-if="store.changedCount > 0" :size="12" />
          <CircleSlash v-else :size="12" />
          {{ readyText }}
        </span>
      </template>
    </div>

    <div class="right">
      <span v-if="store.files.length" class="scope" :class="{ on: store.hasSelection }">
        <ListChecks v-if="store.hasSelection" :size="12" />
        <Layers v-else :size="12" />
        {{ store.hasSelection ? `仅处理勾选的 ${store.selectedCount} 个` : `处理全部 ${store.files.length} 个` }}
      </span>
    </div>
  </footer>
</template>

<style scoped>
.statusbar {
  display: flex;
  align-items: center;
  gap: 16px;
  height: var(--status-h);
  padding: 0 14px;
  background: var(--c-surface);
  border-top: 1px solid var(--c-border);
  flex: none;
  font-size: 12px;
}

.left {
  display: flex;
  align-items: center;
  gap: 14px;
  flex: none;
  color: var(--c-text-secondary);
}

.stat {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
}

.stat b {
  color: var(--c-text);
  font-variant-numeric: tabular-nums;
}

.stat.warn {
  color: var(--c-warning);
}

.stat.sel {
  color: var(--c-primary);
  font-weight: 600;
}

.stat.sel b {
  color: var(--c-primary);
}

.mid {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 9px;
}

.phase {
  flex: none;
  color: var(--c-text-secondary);
  white-space: nowrap;
}

.mid :deep(.track) {
  max-width: 320px;
}

.count {
  font-size: 11px;
  color: var(--c-text-muted);
  flex: none;
}

.stop-btn {
  flex: none;
  border: 1px solid var(--c-border-strong);
  background: transparent;
  color: var(--c-text-secondary);
  border-radius: var(--radius-xs);
  padding: 1px 8px;
  font-size: 11.5px;
  cursor: pointer;
}

.stop-btn:hover {
  color: var(--c-danger);
  border-color: var(--c-danger);
}

.ready {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  color: var(--c-success);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.ready.bad {
  color: var(--c-danger);
}

.right {
  flex: none;
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 右下角常驻显示「本次操作范围」，替代原来放在这里的执行按钮 */
.scope {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 2px 10px;
  border: 1px solid var(--c-border);
  border-radius: var(--radius-full);
  font-size: 11.5px;
  color: var(--c-text-muted);
  white-space: nowrap;
  background: var(--c-surface-2);
}

.scope.on {
  border-color: color-mix(in srgb, var(--c-primary) 40%, transparent);
  background: var(--c-primary-soft);
  color: var(--c-primary);
  font-weight: 600;
}
</style>
