// 演示模式数据源：仅在 `VITE_MOCK=1` 构建时启用。
// 用途：① 浏览器里调 UI 不用每次起 Tauri；② 上架时截图/录屏；
// 不参与真实文件操作，正式 exe 里不会被触发。

import type {
  AttrSpec,
  BatchOptions,
  BatchReport,
  FileEntry,
  FileKind,
  Journal,
  PlanItem,
  RenamePlan,
  RenameRules,
  ScanOptions,
  ScanOutcome,
  TimeSpec,
} from './types'

const NOW = Date.now()

interface Seed {
  name: string
  size: number
  /** 相对今天的天数偏移 */
  days: number
  dir: string
}

const D1 = 'C:\\Users\\Landc\\Desktop\\三月项目资料'
const D2 = 'C:\\Users\\Landc\\Desktop\\三月项目资料\\图片素材'

const SEEDS: Seed[] = [
  { name: '会议纪要-20240312.docx', size: 48_213, days: 2, dir: D1 },
  { name: '会议纪要-20240315(终版).docx', size: 51_902, days: 1, dir: D1 },
  { name: '项目报价单 (1).xlsx', size: 122_880, days: 5, dir: D1 },
  { name: '项目报价单 (2).xlsx', size: 124_416, days: 5, dir: D1 },
  { name: '需求说明 V2.pdf', size: 2_481_920, days: 8, dir: D1 },
  { name: '交付清单.txt', size: 1_842, days: 3, dir: D1 },
  { name: '产品宣传片-1080p.mp4', size: 186_432_512, days: 12, dir: D1 },
  { name: '背景音乐.mp3', size: 8_912_384, days: 30, dir: D1 },
  { name: '【重要】合同扫描件.pdf', size: 5_402_112, days: 20, dir: D1 },
  { name: '未命名文档 3.docx', size: 24_576, days: 0, dir: D1 },
  { name: '客服录音 2024-03-01.m4a', size: 12_582_912, days: 25, dir: D1 },
  { name: '数据备份.zip', size: 45_201_408, days: 40, dir: D1 },
  { name: 'IMG_2041.JPG', size: 4_182_016, days: 3, dir: D2 },
  { name: 'IMG_2042.JPG', size: 4_320_512, days: 3, dir: D2 },
  { name: 'IMG_2043.JPG', size: 3_981_312, days: 3, dir: D2 },
  { name: 'DSC_0087.NEF', size: 28_311_552, days: 6, dir: D2 },
  { name: 'banner_首页 (final).png', size: 892_416, days: 9, dir: D2 },
  { name: 'banner_首页 (final-1).png', size: 894_208, days: 9, dir: D2 },
  { name: 'logo 备份.svg', size: 12_288, days: 15, dir: D2 },
  { name: '微信图片_20240310142233.jpg', size: 1_204_224, days: 18, dir: D2 },
  { name: '微信图片_20240310142244.jpg', size: 1_180_672, days: 18, dir: D2 },
  { name: '屏幕截图 2024-03-11 093015.png', size: 421_888, days: 17, dir: D2 },
  { name: '产品图-主图.tiff', size: 15_728_640, days: 22, dir: D2 },
  { name: '产品图-细节.jpg', size: 3_145_728, days: 22, dir: D2 },
]

function kindOf(ext: string): FileKind {
  const e = ext.toLowerCase()
  if (['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'tif', 'tiff', 'nef', 'svg'].includes(e))
    return 'image'
  if (['doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'csv'].includes(e)) return 'office'
  if (e === 'pdf') return 'pdf'
  if (['mp3', 'flac', 'wav', 'aac', 'm4a', 'ogg'].includes(e)) return 'audio'
  if (['mp4', 'mkv', 'avi', 'mov', 'wmv'].includes(e)) return 'video'
  if (['zip', 'rar', '7z', 'tar', 'gz'].includes(e)) return 'archive'
  if (['txt', 'md', 'log', 'json'].includes(e)) return 'text'
  return 'other'
}

function buildEntries(): FileEntry[] {
  return SEEDS.map((s, i) => {
    const dot = s.name.lastIndexOf('.')
    const stem = dot > 0 ? s.name.slice(0, dot) : s.name
    const ext = dot > 0 ? s.name.slice(dot + 1).toLowerCase() : ''
    const modified = NOW - s.days * 86_400_000 - i * 137_000
    return {
      id: `mock-${i}`,
      path: `${s.dir}\\${s.name}`,
      dir: s.dir,
      name: s.name,
      stem,
      ext,
      size: s.size,
      isDir: false,
      createdMs: modified - 3_600_000,
      modifiedMs: modified,
      accessedMs: modified,
      readonly: false,
      hidden: false,
      system: false,
      archive: true,
      kind: kindOf(ext),
    }
  })
}

let entries = buildEntries()

// ---------- 规则引擎的浏览器端简化实现（仅演示用） ----------
function escapeRe(s: string) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

function toHalf(s: string) {
  return s.replace(/[\uFF01-\uFF5E]/g, (c) => String.fromCharCode(c.charCodeAt(0) - 0xfee0)).replace(/\u3000/g, ' ')
}

function sanitizeName(s: string) {
  let out = s.replace(/[<>:"/\\|?*\x00-\x1f]/g, '_')
  while (out.endsWith(' ') || out.endsWith('.')) out = out.slice(0, -1)
  return out || '_'
}

function applyRules(stem: string, ext: string, seq: number, r: RenameRules) {
  let s = stem

  if (r.normalizeWidth) s = toHalf(s)
  if (r.removeChars) {
    const set = new Set(Array.from(r.removeChars))
    s = Array.from(s).filter((c) => !set.has(c)).join('')
  }
  if (r.find) {
    const flags = r.caseSensitive ? 'g' : 'gi'
    const pattern = r.useRegex ? r.find : escapeRe(r.find)
    try {
      s = s.replace(new RegExp(pattern, flags), r.replaceWith)
    } catch {
      /* 正则非法时原样保留，与后端行为一致 */
    }
  }

  switch (r.caseMode) {
    case 'lower':
      s = s.toLowerCase()
      break
    case 'upper':
      s = s.toUpperCase()
      break
    case 'title':
      s = s.replace(/(^|[\s_\-.])(\p{L})/gu, (_, p, c) => p + c.toUpperCase())
      break
    case 'sentence':
      s = s.charAt(0).toUpperCase() + s.slice(1).toLowerCase()
      break
    default:
      break
  }

  if (r.trim) s = s.trim()
  if (r.removeSpaces) s = s.replace(/\s+/g, '')

  if (r.insertText) {
    const arr = Array.from(s)
    let idx = r.insertPosition < 0 ? Math.max(0, arr.length + r.insertPosition) : Math.min(r.insertPosition, arr.length)
    s = arr.slice(0, idx).join('') + r.insertText + arr.slice(idx).join('')
  }

  if (r.prefix) s = r.prefix + s
  if (r.suffix) s = s + r.suffix

  if (r.numbering.enabled) {
    const n = r.numbering.start + seq * r.numbering.step
    const pad = r.numbering.pad > 0 ? String(Math.abs(n)).padStart(r.numbering.pad, '0') : String(n)
    const num = n < 0 ? `-${pad}` : pad
    const tpl = r.numbering.template.trim() || '{n}'
    const token = tpl.replace(/\{n\}/g, num)
    if (r.numbering.position === 'prefix') s = token + s
    else if (r.numbering.position === 'suffix') s = s + token
    else s = token
  }

  if (r.sanitize) s = sanitizeName(s)

  let e = ext
  if (r.extMode === 'lower') e = ext.toLowerCase()
  else if (r.extMode === 'upper') e = ext.toUpperCase()
  else if (r.extMode === 'replace') e = r.extValue.replace(/^\./, '').toLowerCase()
  else if (r.extMode === 'add') {
    const add = r.extValue.replace(/^\./, '').toLowerCase()
    if (add) e = ext ? `${ext}.${add}` : add
  }

  return { stem: s, ext: e }
}

// ---------- mock api ----------
const delay = (ms: number) => new Promise((r) => setTimeout(r, ms))

let journals: Journal[] = []
/** 上一次预览的计划，供 executeRename 演示「真的改了名」 */
let lastPlan: PlanItem[] = []

export const mockApi = {
  async scanFiles(_opts: ScanOptions): Promise<ScanOutcome> {
    await delay(420)
    entries = buildEntries()
    return {
      entries,
      scanned: entries.length,
      matched: entries.length,
      truncated: false,
      errors: [],
      roots: [D1, D2],
    }
  },

  async getFiles(): Promise<FileEntry[]> {
    return entries
  },

  async clearFiles(): Promise<void> {
    entries = []
  },

  async cancelTask(): Promise<void> {},

  async previewRename(rules: RenameRules, targets: string[]): Promise<RenamePlan> {
    // 与后端语义一致：targets 非空时只对这些文件建计划
    const scope = targets.length ? entries.filter((e) => targets.includes(e.path)) : entries
    const counters = new Map<string, number>()
    const items: PlanItem[] = scope.map((e) => {
      const key = rules.numbering.enabled && rules.numbering.perDir ? e.dir : ''
      const seq = counters.get(key) ?? 0
      counters.set(key, seq + 1)

      const { stem, ext } = applyRules(e.stem, e.ext, seq, rules)
      const newName = ext ? `${stem}.${ext}` : stem
      const newPath = `${e.dir}\\${newName}`
      return {
        id: e.id,
        dir: e.dir,
        oldName: e.name,
        newName,
        oldPath: e.path,
        newPath,
        changed: newName !== e.name,
        level: 'ok',
        issues: [],
        needsTwoPhase: false,
      }
    })

    // 简单重名检测
    const seen = new Map<string, number>()
    for (const it of items) {
      if (!it.changed) continue
      const k = `${it.dir}\\${it.newName}`.toLowerCase()
      seen.set(k, (seen.get(k) ?? 0) + 1)
    }
    for (const it of items) {
      if (!it.changed) continue
      const k = `${it.dir}\\${it.newName}`.toLowerCase()
      if ((seen.get(k) ?? 0) > 1) {
        it.level = 'error'
        it.issues.push('批次内有多个文件会变成同一个名字')
      }
    }

    lastPlan = items
    const changed = items.filter((i) => i.changed).length
    return {
      items,
      total: items.length,
      changed,
      unchanged: items.length - changed,
      errors: items.filter((i) => i.level === 'error').length,
      warnings: 0,
      ruleError: null,
    }
  },

  async executeRename(): Promise<BatchReport> {
    await delay(600)
    // 演示模式真的把新名字写回内存，这样「立即改名」后列表会变，能看到效果
    const done = lastPlan.filter((i) => i.changed && i.level !== 'error')
    const byPath = new Map(entries.map((e) => [e.path, e]))
    for (const it of done) {
      const e = byPath.get(it.oldPath)
      if (!e) continue
      const dot = it.newName.lastIndexOf('.')
      e.name = it.newName
      e.path = it.newPath
      e.stem = dot > 0 ? it.newName.slice(0, dot) : it.newName
      e.ext = dot > 0 ? it.newName.slice(dot + 1).toLowerCase() : ''
    }
    lastPlan = []
    const j: Journal = {
      id: `mock-j-${journals.length + 1}`,
      title: `批量改名 ${done.length} 个文件`,
      kind: 'rename',
      createdAt: Date.now(),
      entries: [],
      undone: false,
    }
    journals = [j, ...journals]
    return { ok: done.length, failed: [], cancelled: false, journalId: j.id, elapsedMs: 600 }
  },

  async applyTimes(_opts: BatchOptions): Promise<BatchReport> {
    await delay(500)
    const j: Journal = {
      id: `mock-j-${journals.length + 1}`,
      title: `修改时间属性 ${entries.length} 个文件`,
      kind: 'times',
      createdAt: Date.now(),
      entries: [],
      undone: false,
    }
    journals = [j, ...journals]
    return { ok: entries.length, failed: [], cancelled: false, journalId: j.id, elapsedMs: 500 }
  },

  async applyAttrs(_opts: BatchOptions): Promise<BatchReport> {
    await delay(400)
    const j: Journal = {
      id: `mock-j-${journals.length + 1}`,
      title: `修改文件属性 ${entries.length} 个文件`,
      kind: 'attrs',
      createdAt: Date.now(),
      entries: [],
      undone: false,
    }
    journals = [j, ...journals]
    return { ok: entries.length, failed: [], cancelled: false, journalId: j.id, elapsedMs: 400 }
  },

  async listJournals(): Promise<Journal[]> {
    return journals
  },

  async undoJournal(id: string): Promise<BatchReport> {
    await delay(400)
    journals = journals.map((j) => (j.id === id ? { ...j, undone: true } : j))
    return { ok: 1, failed: [], cancelled: false, journalId: null, elapsedMs: 400 }
  },

  async getFileDetail(path: string) {
    const f = entries.find((x) => x.path === path)
    return {
      path,
      times: {
        createdMs: f?.createdMs ?? null,
        modifiedMs: f?.modifiedMs ?? null,
        accessedMs: f?.accessedMs ?? null,
      } as TimeSpec,
      attrs: {
        readonly: f?.readonly ?? false,
        hidden: f?.hidden ?? false,
        system: f?.system ?? false,
        archive: f?.archive ?? true,
      } as AttrSpec,
    }
  },
}
