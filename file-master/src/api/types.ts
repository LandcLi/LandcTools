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
