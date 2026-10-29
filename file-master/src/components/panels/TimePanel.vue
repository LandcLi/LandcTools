<script setup lang="ts">
import { computed, ref } from 'vue'
import { useAppStore } from '@/stores/app'
import LcInput from '../ui/LcInput.vue'
import LcSwitch from '../ui/LcSwitch.vue'
import LcSegmented from '../ui/LcSegmented.vue'
import LcButton from '../ui/LcButton.vue'
import { localInputToMs, msToLocalInput, formatDuration } from '@/utils/format'
import { Clock, Hourglass, Zap } from 'lucide-vue-next'

const store = useAppStore()

const mode = ref<'absolute' | 'shift'>('absolute')
const useCreated = ref(true)
const useModified = ref(false)
const useAccessed = ref(false)

const absolute = ref(msToLocalInput(Date.now()))
const days = ref(0)
const hours = ref(0)
const minutes = ref(0)

const modeOptions = [
  { label: '设为指定时间', value: 'absolute' },
  { label: '按原值偏移', value: 'shift' },
]

const shiftMs = computed(() => ((days.value * 24 + hours.value) * 60 + minutes.value) * 60000)

const scopeText = computed(() =>
  store.selectedCount ? `选中的 ${store.selectedCount} 个文件` : `全部 ${store.files.length} 个文件`,
)

const canApply = computed(() => {
  if (store.busy || !store.files.length) return false
  if (!useCreated.value && !useModified.value && !useAccessed.value) return false
  if (mode.value === 'absolute') return localInputToMs(absolute.value) != null
  return shiftMs.value !== 0
})

function setNow() {
  absolute.value = msToLocalInput(Date.now())
}

function setTodayZero() {
  const d = new Date()
  d.setHours(0, 0, 0, 0)
  absolute.value = msToLocalInput(d.getTime())
}

function apply() {
  if (!canApply.value) {
    if (!useCreated.value && !useModified.value && !useAccessed.value) {
      store.notify('err', '请至少勾选一个要修改的时间')
    } else if (mode.value === 'shift' && shiftMs.value === 0) {
      store.notify('err', '偏移量为 0，不会产生改动')
    } else {
      store.notify('err', '请填写有效的时间')
    }
    return
  }

  if (mode.value === 'absolute') {
    const ms = localInputToMs(absolute.value)
    void store.applyTimes(
      {
        createdMs: useCreated.value ? ms : null,
        modifiedMs: useModified.value ? ms : null,
        accessedMs: useAccessed.value ? ms : null,
      },
      null,
    )
  } else {
    void store.applyTimes(null, {
      createdMs: useCreated.value ? shiftMs.value : null,
      modifiedMs: useModified.value ? shiftMs.value : null,
      accessedMs: useAccessed.value ? shiftMs.value : null,
    })
  }
}
</script>

<template>
  <div class="panel fade-in">
    <div class="panel-head">
      <span class="panel-name">时间属性</span>
      <span class="scope" :class="{ on: store.hasSelection }">{{ scopeText }}</span>
    </div>

    <section class="section">
      <div class="section-title"><Clock :size="13" />要修改哪个时间</div>
      <div class="opts col">
        <LcSwitch v-model="useCreated" label="创建时间" />
        <LcSwitch v-model="useModified" label="修改时间" />
        <LcSwitch v-model="useAccessed" label="访问时间" />
      </div>
      <div class="field-hint">
        创建时间是 Windows 独有属性，资源管理器「属性」里能看到，很多场景（如相册归档、文件排序）都靠它
      </div>
    </section>

    <section class="section">
      <div class="section-title"><Hourglass :size="13" />修改方式</div>
      <LcSegmented v-model="mode" :options="modeOptions" class="mb10" />

      <template v-if="mode === 'absolute'">
        <label class="field-label">目标时间</label>
        <input v-model="absolute" type="datetime-local" step="1" class="dt-input" />
        <div class="quick">
          <button type="button" @click="setNow">设为现在</button>
          <button type="button" @click="setTodayZero">设为今天 00:00</button>
        </div>
      </template>

      <template v-else>
        <div class="grid3">
          <div>
            <label class="field-label">天</label>
            <LcInput v-model="days" number type="number" mono />
          </div>
          <div>
            <label class="field-label">小时</label>
            <LcInput v-model="hours" number type="number" mono />
          </div>
          <div>
            <label class="field-label">分钟</label>
            <LcInput v-model="minutes" number type="number" mono />
          </div>
        </div>
        <div class="field-hint">
          可以填负数表示往前推。当前偏移：<b>{{ formatDuration(shiftMs) }}</b>
        </div>
      </template>
    </section>

    <section class="section">
      <LcButton variant="primary" block :disabled="!canApply" @click="apply">
        <template #icon><Zap :size="14" /></template>
        应用时间修改
      </LcButton>
      <div class="field-hint center">执行前不会改动任何文件，执行后可在「撤销历史」还原</div>
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
  white-space: nowrap;
}

.scope.on {
  color: var(--c-primary);
  font-weight: 600;
}

.opts.col {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 10px;
}

.grid3 {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}

.mb10 {
  margin-bottom: 12px;
}

.dt-input {
  width: 100%;
  height: 32px;
  padding: 0 10px;
  background: var(--c-surface);
  border: 1px solid var(--c-border-strong);
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
  font-size: 12.5px;
  color: var(--c-text);
  outline: none;
}

.dt-input:focus {
  border-color: var(--c-primary);
  box-shadow: 0 0 0 3px var(--c-primary-ring);
}

/* 让原生日期选择器图标在深色下也可见 */
.dt-input::-webkit-calendar-picker-indicator {
  filter: invert(0.5);
  cursor: pointer;
}

[data-theme='dark'] .dt-input::-webkit-calendar-picker-indicator {
  filter: invert(0.85);
}

.quick {
  display: flex;
  gap: 6px;
  margin-top: 8px;
}

.quick button {
  border: 1px solid var(--c-border-strong);
  background: transparent;
  color: var(--c-text-secondary);
  border-radius: var(--radius-xs);
  padding: 3px 9px;
  font-size: 11.5px;
  cursor: pointer;
}

.quick button:hover {
  color: var(--c-primary);
  border-color: var(--c-primary);
}

.center {
  text-align: center;
  margin-top: 10px;
}
</style>
