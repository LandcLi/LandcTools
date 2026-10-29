import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { mockApi } from './mock'
import type {
  BatchOptions,
  BatchReport,
  FileDetail,
  FileEntry,
  Journal,
  ProgressPayload,
  RenamePlan,
  RenameRules,
  ScanOptions,
  ScanOutcome,
} from './types'

export const realApi = {
  scanFiles: (opts: ScanOptions) => invoke<ScanOutcome>('scan_files', { opts }),
  getFiles: () => invoke<FileEntry[]>('get_files'),
  clearFiles: () => invoke<void>('clear_files'),
  cancelTask: () => invoke<void>('cancel_task'),
  previewRename: (rules: RenameRules) => invoke<RenamePlan>('preview_rename', { rules }),
  executeRename: () => invoke<BatchReport>('execute_rename'),
  applyTimes: (opts: BatchOptions) => invoke<BatchReport>('apply_times', { opts }),
  applyAttrs: (opts: BatchOptions) => invoke<BatchReport>('apply_attrs', { opts }),
  listJournals: () => invoke<Journal[]>('list_journals'),
  undoJournal: (id: string) => invoke<BatchReport>('undo_journal', { id }),
  getFileDetail: (path: string) => invoke<FileDetail>('get_file_detail', { path }),
}

export type Api = typeof realApi

/**
 * 演示模式：`VITE_MOCK=1 npm run build` 时用假数据，
 * 用于浏览器里调 UI 与上架截图，不触碰真实文件。
 */
export const isMock = import.meta.env.VITE_MOCK === '1'

export const api: Api = isMock ? (mockApi as unknown as Api) : realApi

export function onProgress(cb: (p: ProgressPayload) => void): Promise<UnlistenFn> {
  return listen<ProgressPayload>('task:progress', (e) => cb(e.payload))
}

export function onTaskDone(cb: (r: BatchReport) => void): Promise<UnlistenFn> {
  return listen<BatchReport>('task:done', (e) => cb(e.payload))
}

/** 把后端返回的错误统一成可展示文案 */
export function errText(e: unknown): string {
  if (typeof e === 'string') return e
  if (e instanceof Error) return e.message
  return JSON.stringify(e)
}
