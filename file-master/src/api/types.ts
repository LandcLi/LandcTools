// 与 Rust 端 serde 模型一一对应（Rust 侧统一 camelCase 序列化）

export interface FileEntry {
  id: string
  path: string
  dir: string
  name: string
  stem: string
  ext: string
  size: number
  isDir: boolean
  createdMs: number | null
  modifiedMs: number | null
  accessedMs: number | null
  readonly: boolean
  hidden: boolean
  system: boolean
  archive: boolean
  kind: FileKind
}

export type FileKind =
  | 'image'
  | 'office'
  | 'pdf'
  | 'audio'
  | 'video'
  | 'archive'
  | 'exec'
  | 'text'
  | 'folder'
  | 'other'

export interface ScanOptions {
  dirs: string[]
  recursive?: boolean
  maxDepth?: number | null
  includeHidden?: boolean
  includeDirs?: boolean
  exts?: string[]
  nameFilter?: string
  useRegex?: boolean
  minSize?: number | null
  maxSize?: number | null
  limit?: number | null
}

export interface ScanOutcome {
  entries: FileEntry[]
  scanned: number
  matched: number
  truncated: boolean
  errors: string[]
  /** 实际扫描的根目录，用于展示「已添加的文件夹」与计算相对路径 */
  roots: string[]
}

export type CaseMode = 'none' | 'lower' | 'upper' | 'title' | 'sentence'
export type NumPos = 'prefix' | 'suffix' | 'replace'
export type ExtMode = 'keep' | 'replace' | 'lower' | 'upper' | 'add'

export interface Numbering {
  enabled: boolean
  start: number
  step: number
  pad: number
  template: string
  position: NumPos
  perDir: boolean
}

export interface RenameRules {
  normalizeWidth: boolean
  removeChars: string
  find: string
  replaceWith: string
  useRegex: boolean
  caseSensitive: boolean
  caseMode: CaseMode
  trim: boolean
  removeSpaces: boolean
  insertText: string
  insertPosition: number
  prefix: string
  suffix: string
  numbering: Numbering
  extMode: ExtMode
  extValue: string
  sanitize: boolean
}

export type Level = 'ok' | 'warn' | 'error'

export interface PlanItem {
  id: string
  dir: string
  oldName: string
  newName: string
  oldPath: string
  newPath: string
  changed: boolean
  level: Level
  issues: string[]
  needsTwoPhase: boolean
}

export interface RenamePlan {
  items: PlanItem[]
  total: number
  changed: number
  unchanged: number
  errors: number
  warnings: number
  ruleError: string | null
}

export interface TimeSpec {
  createdMs: number | null
  modifiedMs: number | null
  accessedMs: number | null
}

export interface AttrSpec {
  readonly: boolean | null
  hidden: boolean | null
  system: boolean | null
  archive: boolean | null
}

export interface AttrFlags {
  readonly: boolean
  hidden: boolean
  system: boolean
  archive: boolean
}

export interface BatchOptions {
  targets: string[]
  times?: TimeSpec | null
  shift?: TimeSpec | null
  attrs?: AttrSpec | null
}

export interface FailureItem {
  path: string
  reason: string
}

export interface BatchReport {
  ok: number
  failed: FailureItem[]
  cancelled: boolean
  journalId: string | null
  elapsedMs: number
}

export interface JournalEntry {
  after: string
  before: string
}

export interface Journal {
  id: string
  title: string
  kind: string
  createdAt: number
  entries: JournalEntry[]
  undone: boolean
}

export interface FileDetail {
  path: string
  times: TimeSpec
  attrs: AttrFlags
}

export interface ProgressPayload {
  phase: string
  done: number
  total: number
  current: string
}

// ---------- 标签 / 元数据 ----------

export type MetaFormat = 'office' | 'pdf' | 'audio' | 'image' | 'none'

/**
 * 归一化标签字段。
 * `null` 表示该格式**不提供**这个字段（界面不显示该输入框），
 * `''` 表示有这个字段但当前为空——两者语义完全不同，不要混用。
 */
export interface MetaFields {
  title: string | null
  artist: string | null
  album: string | null
  subject: string | null
  keywords: string | null
  comment: string | null
  genre: string | null
  year: number | null
  software: string | null
  copyright: string | null
}

export type MetaFieldKey = keyof MetaFields

export interface MetaInfo {
  path: string
  format: MetaFormat
  formatLabel: string
  writable: boolean
  reason: string | null
  fields: MetaFields
}

export interface MetaOptions {
  targets: string[]
  /** 只放用户勾选的字段；空串表示清空该字段 */
  patch: Record<string, string>
  /** 写完还原修改时间 */
  keepMtime: boolean
}

/**
 * 各格式支持的字段。
 * ⚠️ 必须与 Rust 端 `core/meta/mod.rs` 的 `MetaFields::supported` 保持一致，
 * 改动时两边一起改。这里重复一份是为了在「勾选了多个不同格式的文件」时
 * 能算出字段并集，而不必为每个文件各跑一次 IPC。
 */
export const FORMAT_FIELDS: Record<MetaFormat, MetaFieldKey[]> = {
  office: ['title', 'artist', 'subject', 'keywords', 'comment'],
  pdf: ['title', 'artist', 'subject', 'keywords', 'software'],
  audio: ['title', 'artist', 'album', 'genre', 'year', 'comment'],
  image: ['comment', 'artist', 'copyright', 'software'],
  none: [],
}

/** 扩展名 → 格式（与 Rust `MetaFormat::of_ext` 对齐） */
export function metaFormatOfExt(ext: string): MetaFormat {
  const e = ext.toLowerCase()
  if (['docx', 'xlsx', 'pptx', 'docm', 'xlsm', 'pptm'].includes(e)) return 'office'
  if (e === 'pdf') return 'pdf'
  if (['mp3', 'flac', 'm4a', 'ogg', 'oga', 'opus', 'wav', 'wma', 'aac', 'aiff', 'ape', 'wv'].includes(e))
    return 'audio'
  if (['jpg', 'jpeg', 'png', 'tif', 'tiff', 'webp', 'heic', 'heif', 'avif', 'jxl'].includes(e))
    return 'image'
  return 'none'
}

/** 新建一个空规则（默认值必须与 Rust 的 Default 对齐） */
export function emptyRules(): RenameRules {
  return {
    normalizeWidth: false,
    removeChars: '',
    find: '',
    replaceWith: '',
    useRegex: false,
    caseSensitive: false,
    caseMode: 'none',
    trim: false,
    removeSpaces: false,
    insertText: '',
    insertPosition: 0,
    prefix: '',
    suffix: '',
    numbering: {
      enabled: false,
      start: 1,
      step: 1,
      pad: 3,
      template: '',
      position: 'prefix',
      perDir: false,
    },
    extMode: 'keep',
    extValue: '',
    sanitize: true,
  }
}
