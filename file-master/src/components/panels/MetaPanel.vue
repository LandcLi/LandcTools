<script setup lang="ts">
import { computed } from 'vue'
import { useAppStore } from '@/stores/app'
import { Tags, Info } from 'lucide-vue-next'
import { formatSize, formatTime, KIND_LABEL } from '@/utils/format'

const store = useAppStore()

/** 没有勾选时，展示列表第一条；有勾选时展示最后点击的那条 */
const target = computed(() => {
  if (store.selectedCount > 0) {
    const path = store.selectedPaths[0]
    return store.files.find((f) => f.id === path || f.path === path) ?? null
  }
  return store.files[0] ?? null
})

const rows = computed(() => {
  const f = target.value
  if (!f) return []
  return [
    { k: '文件名', v: f.name },
    { k: '完整路径', v: f.path, wrap: true },
    { k: '类型', v: `${KIND_LABEL[f.kind] || '其他'}${f.ext ? ` · .${f.ext}` : ''}` },
    { k: '大小', v: f.isDir ? '—' : formatSize(f.size) },
    { k: '创建时间', v: formatTime(f.createdMs, true) },
    { k: '修改时间', v: formatTime(f.modifiedMs, true) },
    { k: '访问时间', v: formatTime(f.accessedMs, true) },
    {
      k: '属性',
      v:
        [
          f.readonly ? '只读' : '',
          f.hidden ? '隐藏' : '',
          f.system ? '系统' : '',
          f.archive ? '存档' : '',
        ]
          .filter(Boolean)
          .join(' · ') || '普通',
    },
  ]
})
</script>

<template>
  <div class="panel fade-in">
    <div class="panel-head">
      <span class="panel-name">元数据</span>
      <span class="scope">只读预览</span>
    </div>

    <section v-if="target" class="section">
      <div class="section-title"><Info :size="13" />文件信息</div>
      <div class="kv">
        <template v-for="r in rows" :key="r.k">
          <div class="kv-k">{{ r.k }}</div>
          <div class="kv-v" :class="{ wrap: r.wrap }">{{ r.v }}</div>
        </template>
      </div>
    </section>

    <section v-else class="section">
      <div class="field-hint">还没有载入文件，先添加一个文件夹。</div>
    </section>

    <section class="section">
      <div class="section-title"><Tags :size="13" />标签编辑</div>
      <div class="plan">
        <div class="plan-item">
          <span class="dot office" />
          <div>
            <div class="pi-title">Office 文档属性</div>
            <div class="pi-desc">标题 / 作者 / 主题 / 关键词 / 备注（docx、xlsx、pptx）</div>
          </div>
        </div>
        <div class="plan-item">
          <span class="dot pdf" />
          <div>
            <div class="pi-title">PDF 文档属性</div>
            <div class="pi-desc">标题 / 作者 / 主题 / 创建程序</div>
          </div>
        </div>
        <div class="plan-item">
          <span class="dot audio" />
          <div>
            <div class="pi-title">音乐标签</div>
            <div class="pi-desc">歌名 / 歌手 / 专辑 / 年份 / 封面（MP3、FLAC、M4A、OGG）</div>
          </div>
        </div>
        <div class="plan-item">
          <span class="dot image" />
          <div>
            <div class="pi-title">图片 EXIF</div>
            <div class="pi-desc">拍摄时间 / 相机型号 / 描述，可批量改写拍摄时间</div>
          </div>
        </div>
      </div>
      <div class="field-hint top-gap">
        标签写入需要按格式分别解析容器结构，为保证改名与时间属性的稳定性，这部分放在下个版本单独交付。
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

.plan {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.plan-item {
  display: flex;
  gap: 10px;
  align-items: flex-start;
}

.dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  margin-top: 6px;
  flex: none;
}

.dot.office {
  background: var(--kind-office);
}
.dot.pdf {
  background: var(--kind-pdf);
}
.dot.audio {
  background: var(--kind-audio);
}
.dot.image {
  background: var(--kind-image);
}

.pi-title {
  font-size: 12.5px;
  font-weight: 600;
}

.pi-desc {
  font-size: 11px;
  color: var(--c-text-muted);
  line-height: 1.5;
  margin-top: 1px;
}

.top-gap {
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px solid var(--c-border);
}
</style>
