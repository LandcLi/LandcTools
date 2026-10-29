<script setup lang="ts">
import { computed, reactive } from 'vue'
import { useAppStore } from '@/stores/app'
import LcSelect from '../ui/LcSelect.vue'
import LcButton from '../ui/LcButton.vue'
import { SlidersHorizontal, Zap } from 'lucide-vue-next'

const store = useAppStore()

type Tri = 'keep' | 'on' | 'off'

const spec = reactive<Record<'readonly' | 'hidden' | 'system' | 'archive', Tri>>({
  readonly: 'keep',
  hidden: 'keep',
  system: 'keep',
  archive: 'keep',
})

const options = [
  { label: '不修改', value: 'keep' },
  { label: '设为 是', value: 'on' },
  { label: '设为 否', value: 'off' },
]

const toBool = (v: Tri): boolean | null => (v === 'on' ? true : v === 'off' ? false : null)

const scopeText = computed(() =>
  store.selectedCount ? `选中的 ${store.selectedCount} 个文件` : `全部 ${store.files.length} 个文件`,
)

const canApply = computed(
  () => !store.busy && store.files.length > 0 && Object.values(spec).some((v) => v !== 'keep'),
)

function apply() {
  void store.applyAttrs({
    readonly: toBool(spec.readonly),
    hidden: toBool(spec.hidden),
    system: toBool(spec.system),
    archive: toBool(spec.archive),
  })
}

function reset() {
  spec.readonly = 'keep'
  spec.hidden = 'keep'
  spec.system = 'keep'
  spec.archive = 'keep'
}
</script>

<template>
  <div class="panel fade-in">
    <div class="panel-head">
      <span class="panel-name">文件属性</span>
      <span class="scope">{{ scopeText }}</span>
    </div>

    <section class="section">
      <div class="section-title"><SlidersHorizontal :size="13" />Windows 属性位</div>

      <div class="attr-row">
        <div class="attr-meta">
          <div class="attr-name">只读</div>
          <div class="attr-desc">防止被误改，程序写入前需要先取消</div>
        </div>
        <LcSelect v-model="spec.readonly" :options="options" class="w96" />
      </div>

      <div class="attr-row">
        <div class="attr-meta">
          <div class="attr-name">隐藏</div>
          <div class="attr-desc">资源管理器默认不显示</div>
        </div>
        <LcSelect v-model="spec.hidden" :options="options" class="w96" />
      </div>

      <div class="attr-row">
        <div class="attr-meta">
          <div class="attr-name">系统</div>
          <div class="attr-desc">标记为系统文件，谨慎使用</div>
        </div>
        <LcSelect v-model="spec.system" :options="options" class="w96" />
      </div>

      <div class="attr-row">
        <div class="attr-meta">
          <div class="attr-name">存档</div>
          <div class="attr-desc">备份软件判断「是否需要备份」的标记</div>
        </div>
        <LcSelect v-model="spec.archive" :options="options" class="w96" />
      </div>
    </section>

    <section class="section">
      <div class="actions">
        <LcButton variant="primary" :disabled="!canApply" @click="apply">
          <template #icon><Zap :size="14" /></template>
          应用属性修改
        </LcButton>
        <LcButton variant="ghost" @click="reset">清空</LcButton>
      </div>
      <div class="field-hint center">「不修改」表示保持该属性当前状态不变</div>
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

.attr-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 0;
  border-bottom: 1px solid color-mix(in srgb, var(--c-border) 60%, transparent);
}

.attr-row:last-child {
  border-bottom: none;
  padding-bottom: 2px;
}

.attr-meta {
  flex: 1;
  min-width: 0;
}

.attr-name {
  font-size: 12.5px;
  font-weight: 600;
}

.attr-desc {
  font-size: 11px;
  color: var(--c-text-muted);
  margin-top: 1px;
  line-height: 1.45;
}

.w96 {
  width: 92px;
  flex: none;
}

.actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.center {
  text-align: center;
  margin-top: 10px;
}
</style>
