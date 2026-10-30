<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useAppStore } from '@/stores/app'
import { api, errText } from '@/api'
import {
  FORMAT_FIELDS,
  metaFormatOfExt,
  type MetaFieldKey,
  type MetaFormat,
  type MetaInfo,
} from '@/api/types'
import LcButton from '../ui/LcButton.vue'
import LcCheckbox from '../ui/LcCheckbox.vue'
import LcInput from '../ui/LcInput.vue'
import LcSwitch from '../ui/LcSwitch.vue'
import { AlertTriangle, Info, Tags, Zap } from 'lucide-vue-next'
import { formatSize, formatTime, KIND_LABEL } from '@/utils/format'

const store = useAppStore()

/** 字段定义。顺序即界面顺序，`hint` 兼顾「这个字段在不同格式里叫什么」 */
const FIELDS: { key: MetaFieldKey; label: string; hint: string }[] = [
  { key: 'title', label: '标题', hint: '文档标题 / 歌曲名' },
  { key: 'artist', label: '作者', hint: '作者 / 歌手 / 摄影师' },
  { key: 'album', label: '专辑', hint: '音频专辑名' },
  { key: 'subject', label: '主题', hint: 'Office / PDF 文档主题' },
  { key: 'keywords', label: '关键词', hint: '多个用逗号分隔' },
  { key: 'genre', label: '流派', hint: '如 流行 / 摇滚 / 纯音乐' },
  { key: 'year', label: '年份', hint: '四位数字，留空表示清空' },
  { key: 'comment', label: '备注', hint: 'Office 的备注 / 图片的描述' },
  { key: 'software', label: '创建程序', hint: '标记是用什么软件做的' },
  { key: 'copyright', label: '版权', hint: '版权声明' },
]

const editing = reactive<Record<string, boolean>>({})
const values = reactive<Record<string, string>>({})
// 预置成 false，否则 LcCheckbox 收到 undefined 会闪烁告警
for (const f of FIELDS) {
  editing[f.key] = false
  values[f.key] = ''
}
const keepMtime = ref(true)
const current = ref<MetaInfo | null>(null)
const loading = ref(false)
const loadError = ref('')

/** 当前正在查看/取现有值的文件：优先第一个勾选项，否则第一条 */
const focus = computed(() => store.targetFiles[0] ?? null)

/** 本次操作覆盖到的格式集合（可能混格式） */
const formats = computed(() => {
  const s = new Set<MetaFormat>()
  for (const f of store.targetFiles) s.add(metaFormatOfExt(f.ext))
  return s
})

/** 可用字段 = 所有目标格式支持字段的并集 */
const supportedKeys = computed(() => {
  const s = new Set<string>()
  for (const fmt of formats.value) for (const k of FORMAT_FIELDS[fmt]) s.add(k)
  return s
})

const hasAnySupport = computed(() => supportedKeys.value.size > 0)

/** 界面上要显示的字段：至少有一种目标格式支持 */
const visibleFields = computed(() => FIELDS.filter((f) => supportedKeys.value.has(f.key)))

const formatLabel = computed(() => {
  const list = [...formats.value].filter((f) => f !== 'none')
  if (!list.length) return '不支持'
  if (list.length === 1) return labelOf(list[0])
  return `${list.length} 种格式`
})

function labelOf(f: MetaFormat) {
  return (
    { office: 'Office 文档', pdf: 'PDF 文档', audio: '音频文件', image: '图片', none: '不支持' }[f] ??
    '不支持'
  )
}

const patch = computed(() => {
  const p: Record<string, string> = {}
  for (const f of visibleFields.value) {
    if (!editing[f.key]) continue
    p[f.key] = (values[f.key] ?? '').trim()
  }
  return p
})

const patchCount = computed(() => Object.keys(patch.value).length)

const canApply = computed(() => !store.busy && hasAnySupport.value && patchCount.value > 0)

const scopeText = computed(() =>
  store.targetFiles.length ? `选中的 ${store.targetFiles.length} 个文件` : `全部 ${store.files.length} 个文件`,
)

/** 当前文件里该字段的现有值，用来做 placeholder */
function currentOf(key: MetaFieldKey): string {
  const f = current.value?.fields
  if (!f) return ''
  const v = f[key]
  if (v === null || v === undefined) return ''
  return String(v)
}

const infoRows = computed(() => {
  const f = focus.value
  if (!f) return []
  return [
    { k: '文件名', v: f.name },
    { k: '完整路径', v: f.path, wrap: true },
    { k: '类型', v: `${KIND_LABEL[f.kind] || '其他'}${f.ext ? ` · .${f.ext}` : ''}` },
    { k: '大小', v: f.isDir ? '—' : formatSize(f.size) },
    { k: '修改时间', v: formatTime(f.modifiedMs, true) },
  ]
})

async function load(path: string) {
  loading.value = true
  loadError.value = ''
  try {
    current.value = await api.readMeta(path)
  } catch (e) {
    current.value = null
    loadError.value = errText(e)
  } finally {
    loading.value = false
  }
}

// 切换文件就重新读一次标签，否则输入框 placeholder 会一直显示上一个文件的值
watch(
  () => focus.value?.path,
  (p) => {
    if (p) void load(p)
    else current.value = null
  },
  { immediate: true },
)

// 目标格式变了（比如从 docx 换成 mp3），已勾选但不适用的字段要清掉
watch(supportedKeys, (s) => {
  for (const f of FIELDS) {
    if (!s.has(f.key)) {
      editing[f.key] = false
      values[f.key] = ''
    }
  }
})

function reset() {
  for (const f of FIELDS) {
    editing[f.key] = false
    values[f.key] = ''
  }
}

async function apply() {
  if (!canApply.value) return
  await store.applyMeta({ ...patch.value }, keepMtime.value)
  // 写完值变了，把 placeholder 刷新成最新
  if (focus.value) await load(focus.value.path)
}
</script>

<template>
  <div class="panel fade-in">
    <div class="panel-head">
      <span class="panel-name">元数据</span>
      <span class="scope" :class="{ on: store.hasSelection }">{{ scopeText }}</span>
    </div>

    <section v-if="focus" class="section">
      <div class="section-title"><Info :size="13" />文件信息</div>
      <div class="kv">
        <template v-for="r in infoRows" :key="r.k">
          <div class="kv-k">{{ r.k }}</div>
          <div class="kv-v" :class="{ wrap: r.wrap }">{{ r.v }}</div>
        </template>
      </div>
    </section>

    <section class="section">
      <div class="section-title">
        <Tags :size="13" />标签编辑
        <span class="tag-format">{{ formatLabel }}</span>
      </div>

      <div v-if="loadError" class="warn-box">
        <AlertTriangle :size="13" />
        <span>{{ loadError }}</span>
      </div>

      <template v-if="hasAnySupport">
        <div class="field-hint top-gap-s">
          勾选要修改的字段 → 填新值 → 应用。不勾的字段保持原样，
          <b>勾选但留空表示清空该字段</b>。
        </div>

        <div class="fields">
          <div v-for="f in visibleFields" :key="f.key" class="field-row">
            <LcCheckbox v-model="editing[f.key]" />
            <div class="field-meta">
              <div class="field-name">{{ f.label }}</div>
              <div class="field-hint-inline">{{ f.hint }}</div>
            </div>
            <LcInput
              v-model="values[f.key]"
              class="field-input"
              :disabled="!editing[f.key]"
              :placeholder="currentOf(f.key) || (f.key === 'year' ? '如 2024' : '未设置')"
            />
          </div>
        </div>

        <div class="switch-row">
          <div class="field-meta">
            <div class="field-name">保留修改时间</div>
            <div class="field-hint-inline">写完把文件修改时间还原，避免整批文件时间变成今天</div>
          </div>
          <LcSwitch v-model="keepMtime" />
        </div>

        <div class="actions">
          <LcButton variant="primary" :disabled="!canApply" @click="apply">
            <template #icon><Zap :size="14" /></template>
            应用{{ patchCount ? ` ${patchCount} 项` : '' }}到 {{ store.targetFiles.length || store.files.length }} 个文件
          </LcButton>
          <LcButton variant="ghost" @click="reset">清空</LcButton>
        </div>
      </template>

      <div v-else-if="!store.files.length" class="warn-box plain">
        <Info :size="13" />
        <span>还没有载入文件，先在左侧添加一个文件夹。</span>
      </div>

      <div v-else class="warn-box">
        <AlertTriangle :size="13" />
        <span>
          当前{{ store.targetFiles.length > 1 ? '选中的文件' : '文件' }}不支持标签编辑。
          支持 docx / xlsx / pptx、PDF、MP3 / FLAC / M4A / WAV 等音频、JPG / PNG / TIFF / WebP 图片。
        </span>
      </div>

      <div v-if="loading" class="field-hint top-gap-s">正在读取标签…</div>
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

.kv {
  display: grid;
  grid-template-columns: 62px 1fr;
  gap: 6px 10px;
  font-size: 12px;
}

.kv-k {
  color: var(--c-text-muted);
}

.kv-v {
  color: var(--c-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--font-mono);
  font-size: 11.5px;
}

.kv-v.wrap {
  white-space: normal;
  word-break: break-all;
  line-height: 1.5;
}

.tag-format {
  margin-left: auto;
  font-size: 10.5px;
  color: var(--c-text-muted);
  background: var(--c-surface-2, rgba(127, 127, 127, 0.12));
  padding: 1px 6px;
  border-radius: 999px;
}

.top-gap-s {
  margin-top: 8px;
}

.fields {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-top: 10px;
}

.field-row {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 5px 0;
}

.field-meta {
  flex: 1;
  min-width: 0;
}

.field-name {
  font-size: 12.5px;
  font-weight: 600;
  line-height: 1.3;
}

.field-hint-inline {
  font-size: 10.5px;
  color: var(--c-text-muted);
  line-height: 1.35;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.field-input {
  width: 46%;
  flex: none;
}

.switch-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px solid var(--c-border);
}

.actions {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
}

.warn-box {
  display: flex;
  gap: 8px;
  align-items: flex-start;
  margin-top: 10px;
  padding: 9px 10px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--c-warn, #d97706) 12%, transparent);
  color: var(--c-text);
  font-size: 11.5px;
  line-height: 1.5;
}

.warn-box svg {
  flex: none;
  margin-top: 2px;
  color: var(--c-warn, #d97706);
}

.warn-box.plain {
  background: var(--c-surface-2, rgba(127, 127, 127, 0.1));
}

.warn-box.plain svg {
  color: var(--c-text-muted);
}
</style>
