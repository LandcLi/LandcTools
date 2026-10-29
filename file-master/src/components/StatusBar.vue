<script setup lang="ts">
import { computed } from 'vue'
import { useAppStore } from '@/stores/app'
import LcButton from './ui/LcButton.vue'
import LcProgress from './ui/LcProgress.vue'
import { AlertTriangle, CheckCircle2, CircleSlash } from 'lucide-vue-next'

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

const canExecute = computed(
  () =>
    store.panel === 'rename' &&
    !!store.plan &&
    store.plan.changed > 0 &&
    store.plan.errors === 0 &&
    !store.plan.ruleError &&
    !store.busy,
)

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
      <span v-if="store.selectedCount" class="stat">
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
      <LcButton
        v-if="store.panel === 'rename'"
        variant="primary"
        size="lg"
        :disabled="!canExecute"
        @click="store.executeRename()"
      >
        立即改名
        <template v-if="store.changedCount > 0" #icon>
          <span class="count-pill">{{ store.changedCount }}</span>
        </template>
      </LcButton>
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

.count-pill {
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
</style>
