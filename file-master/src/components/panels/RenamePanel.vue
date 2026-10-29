<script setup lang="ts">
import { computed, onMounted, watch } from 'vue'
import { useAppStore } from '@/stores/app'
import LcInput from '../ui/LcInput.vue'
import LcSelect from '../ui/LcSelect.vue'
import LcSwitch from '../ui/LcSwitch.vue'
import LcSegmented from '../ui/LcSegmented.vue'
import LcButton from '../ui/LcButton.vue'
import { Replace, Scissors, TextCursorInput, Hash, CaseSensitive, FileType2, Sparkles, RotateCcw } from 'lucide-vue-next'

const store = useAppStore()

const caseOptions = [
  { label: '保持原样', value: 'none' },
  { label: '全部小写', value: 'lower' },
  { label: '全部大写', value: 'upper' },
  { label: '每词首字母大写', value: 'title' },
  { label: '仅句首大写', value: 'sentence' },
]

const extOptions = [
  { label: '不修改', value: 'keep' },
  { label: '替换为', value: 'replace' },
  { label: '全部转小写', value: 'lower' },
  { label: '全部转大写', value: 'upper' },
  { label: '追加后缀', value: 'add' },
]

const posOptions = [
  { label: '前缀', value: 'prefix' },
  { label: '后缀', value: 'suffix' },
  { label: '替换原名', value: 'replace' },
]

/** 用第一条文件做实时示例，让用户不用看列表也知道规则效果 */
const sample = computed(() => {
  const first = store.plan?.items.find((i) => i.changed) ?? store.plan?.items[0]
  return first ?? null
})

onMounted(() => {
  watch(
    () => JSON.stringify(store.rules),
    () => store.schedulePreview(),
  )
})
</script>

<template>
  <div class="panel fade-in">
    <div class="panel-head">
      <span class="panel-name">批量改名</span>
      <LcButton variant="ghost" size="sm" title="重置所有规则" @click="store.resetRules()">
        <template #icon><RotateCcw :size="13" /></template>
        重置
      </LcButton>
    </div>

    <!-- 查找替换 -->
    <section class="section">
      <div class="section-title"><Replace :size="13" />查找替换</div>
      <div class="stack">
        <LcInput v-model="store.rules.find" placeholder="要查找的文字，如 副本" clearable />
        <LcInput v-model="store.rules.replaceWith" placeholder="替换成（留空表示删除）" clearable />
      </div>
      <div class="opts">
        <LcSwitch v-model="store.rules.useRegex" label="正则表达式" />
        <LcSwitch v-model="store.rules.caseSensitive" label="区分大小写" />
      </div>
      <div v-if="store.rules.useRegex" class="field-hint">
        支持分组捕获，替换处可用 $1 $2 引用；例如查找 <code>(\d+)</code> 替换为 <code>第$1集</code>
      </div>
    </section>

    <!-- 剔除字符 -->
    <section class="section">
      <div class="section-title"><Scissors :size="13" />剔除字符</div>
      <LcInput v-model="store.rules.removeChars" placeholder="这些字符会从文件名中删除" clearable />
      <div class="field-hint">
        直接输入要剔除的字符，例如输入 <code>【】()</code> 就会把所有文件名的方括号和圆括号去掉
      </div>
    </section>

    <!-- 前后缀 -->
    <section class="section">
      <div class="section-title"><TextCursorInput :size="13" />添加前缀 / 后缀</div>
      <div class="row2">
        <LcInput v-model="store.rules.prefix" placeholder="前缀" clearable />
        <LcInput v-model="store.rules.suffix" placeholder="后缀" clearable />
      </div>
      <div class="field-hint">后缀会加在扩展名之前：<code>照片.jpg</code> → <code>2024照片_精修.jpg</code></div>
    </section>

    <!-- 自动编号 -->
    <section class="section">
      <div class="section-title">
        <Hash :size="13" />自动编号
        <LcSwitch v-model="store.rules.numbering.enabled" class="ml-auto" />
      </div>

      <template v-if="store.rules.numbering.enabled">
        <LcSegmented v-model="store.rules.numbering.position" :options="posOptions" class="mb8" />
        <div class="grid3">
          <div>
            <label class="field-label">起始</label>
            <LcInput v-model="store.rules.numbering.start" number type="number" mono />
          </div>
          <div>
            <label class="field-label">步长</label>
            <LcInput v-model="store.rules.numbering.step" number type="number" mono />
          </div>
          <div>
            <label class="field-label">补零</label>
            <LcInput v-model="store.rules.numbering.pad" number type="number" mono />
          </div>
        </div>
        <div class="mt8">
          <label class="field-label">自定义格式（用 {n} 代表序号）</label>
          <LcInput v-model="store.rules.numbering.template" placeholder="留空 = {n}，可写 第{n}集 或 ({n})" />
        </div>
        <div class="opts">
          <LcSwitch v-model="store.rules.numbering.perDir" label="每个文件夹单独计数" />
        </div>
      </template>
      <div v-else class="field-hint">开启后可以按顺序给文件加 001、002、003 这样的编号</div>
    </section>

    <!-- 大小写与空格 -->
    <section class="section">
      <div class="section-title"><CaseSensitive :size="13" />大小写与空格</div>
      <div class="stack">
        <LcSelect v-model="store.rules.caseMode" :options="caseOptions" />
        <div class="opts">
          <LcSwitch v-model="store.rules.trim" label="去掉首尾空格" />
          <LcSwitch v-model="store.rules.removeSpaces" label="去掉所有空格" />
        </div>
      </div>
    </section>

    <!-- 扩展名 -->
    <section class="section">
      <div class="section-title"><FileType2 :size="13" />扩展名</div>
      <div class="stack">
        <LcSelect v-model="store.rules.extMode" :options="extOptions" />
        <LcInput
          v-if="store.rules.extMode === 'replace' || store.rules.extMode === 'add'"
          v-model="store.rules.extValue"
          placeholder="扩展名，如 jpg（不用输入点）"
          mono
          clearable
        />
      </div>
      <div v-if="store.rules.extMode === 'add'" class="field-hint">
        追加会用点连接：<code>a.tar</code> + <code>gz</code> → <code>a.tar.gz</code>
      </div>
    </section>

    <!-- 高级 -->
    <section class="section">
      <div class="section-title"><Sparkles :size="13" />高级</div>
      <div class="opts col">
        <LcSwitch v-model="store.rules.normalizeWidth" label="全角字符转半角" />
        <LcSwitch v-model="store.rules.sanitize" label="自动替换 Windows 非法字符为下划线" />
      </div>
    </section>

    <!-- 示例 -->
    <section v-if="sample" class="section sample">
      <div class="sample-label">示例</div>
      <div class="sample-body">
        <div class="sample-line">{{ sample.oldName }}</div>
        <div class="sample-line new">{{ sample.newName }}</div>
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

.stack {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.row2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.grid3 {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}

.opts {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-top: 9px;
  flex-wrap: wrap;
}

.opts.col {
  flex-direction: column;
  align-items: flex-start;
  gap: 10px;
}

.ml-auto {
  margin-left: auto;
}

.mb8 {
  margin-bottom: 8px;
}

.mt8 {
  margin-top: 8px;
}

code {
  font-family: var(--font-mono);
  font-size: 11px;
  padding: 1px 4px;
  border-radius: 3px;
  background: var(--c-surface-2);
  border: 1px solid var(--c-border);
  color: var(--c-text-secondary);
}

.sample {
  background: var(--c-surface-2);
}

.sample-label {
  font-size: 11px;
  font-weight: 700;
  color: var(--c-text-muted);
  letter-spacing: 0.05em;
  margin-bottom: 7px;
}

.sample-body {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sample-line {
  font-family: var(--font-mono);
  font-size: 11.5px;
  color: var(--c-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sample-line.new {
  color: var(--c-primary);
  font-weight: 600;
}
</style>
