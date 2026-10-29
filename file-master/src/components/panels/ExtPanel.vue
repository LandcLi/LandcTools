<script setup lang="ts">
import { computed } from 'vue'
import { useAppStore } from '@/stores/app'
import LcInput from '../ui/LcInput.vue'
import LcSegmented from '../ui/LcSegmented.vue'
import LcButton from '../ui/LcButton.vue'
import { FileType2, Zap } from 'lucide-vue-next'

const store = useAppStore()

const modeOptions = [
  { label: '替换', value: 'replace' },
  { label: '转小写', value: 'lower' },
  { label: '转大写', value: 'upper' },
  { label: '追加', value: 'add' },
]

/** 当前后缀分布，让用户在转换前心里有数（统计范围跟随勾选） */
const dist = computed(() => {
  const m = new Map<string, number>()
  for (const f of store.targetFiles) {
    if (f.isDir) continue
    const k = f.ext || '(无)'
    m.set(k, (m.get(k) ?? 0) + 1)
  }
  return Array.from(m.entries())
    .sort((a, b) => b[1] - a[1])
    .slice(0, 10)
})

const total = computed(() => store.targetFiles.filter((f) => !f.isDir).length)

const canRun = computed(
  () =>
    !store.busy &&
    total.value > 0 &&
    store.changedCount > 0 &&
    store.errorCount === 0 &&
    !store.plan?.ruleError,
)

function syncMode(v: string) {
  store.rules.extMode = v as typeof store.rules.extMode
  store.schedulePreview()
}

function run() {
  void store.executeRename()
}
</script>

<template>
  <div class="panel fade-in">
    <div class="panel-head">
      <span class="panel-name">后缀转换</span>
      <span class="scope" :class="{ on: store.hasSelection }">
        {{ store.hasSelection ? '勾选' : '全部' }} {{ total }} 个文件
      </span>
    </div>

    <section class="section">
      <div class="section-title"><FileType2 :size="13" />当前后缀分布</div>
      <div v-if="dist.length" class="chips">
        <span v-for="[ext, n] in dist" :key="ext" class="chip">
          <b>{{ ext }}</b>
          <i>{{ n }}</i>
        </span>
      </div>
      <div v-else class="field-hint">还没有载入文件</div>
    </section>

    <section class="section">
      <div class="section-title">转换方式</div>
      <LcSegmented
        :model-value="store.rules.extMode === 'keep' ? 'replace' : store.rules.extMode"
        :options="modeOptions"
        class="mb10"
        @update:model-value="syncMode"
      />

      <template v-if="store.rules.extMode === 'replace' || store.rules.extMode === 'add'">
        <label class="field-label">
          {{ store.rules.extMode === 'replace' ? '改成什么后缀' : '追加什么后缀' }}
        </label>
        <LcInput
          v-model="store.rules.extValue"
          placeholder="如 jpg、png、mp4（不用输入点）"
          mono
          clearable
          @update:model-value="store.schedulePreview()"
        />
      </template>

      <div class="field-hint">
        <template v-if="store.rules.extMode === 'replace'">
          只改后缀文字，不会真正转换文件内容。视频、图片的真实格式转换需要专门工具。
        </template>
        <template v-else-if="store.rules.extMode === 'lower' || store.rules.extMode === 'upper'">
          把所有文件后缀统一成{{ store.rules.extMode === 'lower' ? '小写' : '大写' }}，文件名部分不动。
        </template>
        <template v-else>
          追加会用点连接：<code>a.tar</code> + <code>gz</code> → <code>a.tar.gz</code>
        </template>
      </div>
    </section>

    <section class="section">
      <LcButton variant="primary" block :disabled="!canRun" @click="run">
        <template #icon><Zap :size="14" /></template>
        立即转换 {{ store.changedCount }} 个文件
      </LcButton>
      <div class="field-hint center">建议先在列表里确认预览结果再执行</div>
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
  align-items: baseline;
  justify-content: space-between;
  padding: 10px 16px 4px;
  gap: 8px;
}

.panel-name {
  font-size: 13px;
  font-weight: 700;
}

.scope {
  font-size: 11px;
  color: var(--c-text-muted);
}

.scope.on {
  color: var(--c-primary);
  font-weight: 600;
}

.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 2px 8px;
  border-radius: var(--radius-full);
  background: var(--c-surface-2);
  border: 1px solid var(--c-border);
  font-size: 11px;
}

.chip b {
  font-family: var(--font-mono);
  font-weight: 600;
  color: var(--c-text);
}

.chip i {
  font-style: normal;
  color: var(--c-text-muted);
  font-variant-numeric: tabular-nums;
}

.mb10 {
  margin-bottom: 12px;
}

.center {
  text-align: center;
  margin-top: 10px;
}

code {
  font-family: var(--font-mono);
  font-size: 11px;
  padding: 1px 4px;
  border-radius: 3px;
  background: var(--c-surface-2);
  border: 1px solid var(--c-border);
}
</style>
