import { defineStore } from 'pinia'
import { computed, reactive, ref, shallowRef, watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { api, errText, isMock, onProgress, onTaskDone } from '@/api'
import { emptyRules } from '@/api/types'
import type {
  AttrSpec,
  BatchReport,
  FileEntry,
  Journal,
  RenamePlan,
  RenameRules,
  ScanOptions,
  TimeSpec,
} from '@/api/types'

export type PanelKey = 'rename' | 'time' | 'attr' | 'ext' | 'meta' | 'history'
export type ToastType = 'ok' | 'err' | 'info'

export const useAppStore = defineStore('app', () => {
  // ---------- 数据 ----------
  const files = shallowRef<FileEntry[]>([])
  const selected = ref<Set<string>>(new Set())
  const plan = shallowRef<RenamePlan | null>(null)
  const rules = reactive<RenameRules>(emptyRules())
  const journals = ref<Journal[]>([])
  /** 用户添加过的文件夹（扫描根），用于显示来源、相对路径与重扫 */
  const roots = ref<string[]>([])

  // ---------- 界面 ----------
  const panel = ref<PanelKey>('rename')
  const theme = ref<'light' | 'dark'>('light')
  const query = ref('')
  const sidebarOpen = ref(true)

  // ---------- 任务 ----------
  const busy = ref(false)
  const phase = ref('')
  const progress = reactive({ done: 0, total: 0, current: '' })
  const toast = ref<{ type: ToastType; text: string } | null>(null)

  // ---------- 派生 ----------
  const filtered = computed(() => {
    const q = query.value.trim().toLowerCase()
    if (!q) return files.value
    return files.value.filter((f) => f.name.toLowerCase().includes(q))
  })

  /** 勾选的条目 id（扫描时生成的 UUID，每次扫描都会变） */
  const selectedIds = computed(() => Array.from(selected.value))
  /**
   * 勾选条目的**真实路径**。
   * 后端一律按 path 匹配目标，所以这里必须把 id 映射回 path。
   * 直接把 UUID 传下去会让后端一个都找不到 —— 表现为改名时弹
   * 「选中的文件已不在列表中」，时间/属性操作则静默失效。
   */
  const selectedPaths = computed(() => {
    if (!selected.value.size) return []
    const out: string[] = []
    for (const f of files.value) {
      if (selected.value.has(f.id)) out.push(f.path)
    }
    return out
  })
  const selectedCount = computed(() => selected.value.size)
  const visibleCount = computed(() => filtered.value.length)
  const hasSelection = computed(() => selected.value.size > 0)

  const changedCount = computed(() => plan.value?.changed ?? 0)
  const errorCount = computed(() => plan.value?.errors ?? 0)
  const warnCount = computed(() => plan.value?.warnings ?? 0)

  /** 操作目标：有勾选就用勾选项；没勾选返回空数组，后端语义 = 全部 */
  function targetPaths(): string[] {
    return selectedPaths.value
  }

  /** 本次操作实际覆盖的文件对象（供各面板做统计与文案） */
  const targetFiles = computed<FileEntry[]>(() =>
    selected.value.size > 0
      ? files.value.filter((f) => selected.value.has(f.id))
      : files.value,
  )

  /**
   * 本次操作实际覆盖的数量。
   * 勾选联动的核心：勾了 1 个就是 1，没勾才回落到全部。
   */
  const scopeCount = computed(() =>
    selected.value.size > 0 ? selected.value.size : files.value.length,
  )
  const scopeLabel = computed(() =>
    selected.value.size > 0
      ? `选中的 ${selected.value.size} 个文件`
      : `全部 ${files.value.length} 个文件`,
  )

  /** 改名能否执行 */
  const canExecuteRename = computed(
    () =>
      !!plan.value &&
      plan.value.changed > 0 &&
      plan.value.errors === 0 &&
      !plan.value.ruleError &&
      !busy.value,
  )

  /** 取路径最后一级目录名；盘根（如 C:\）原样返回 */
  function baseName(p: string): string {
    const s = p.replace(/[\\/]+$/, '')
    if (!s) return p
    const i = Math.max(s.lastIndexOf('\\'), s.lastIndexOf('/'))
    return i < 0 ? s : s.slice(i + 1)
  }

  /** 根目录按长度降序，保证嵌套根优先命中更具体的那个 */
  const sortedRoots = computed(() =>
    roots.value
      .map((r) => r.replace(/[\\/]+$/, ''))
      .filter(Boolean)
      .sort((a, b) => b.length - a.length),
  )

  /** 根目录的显示名：盘根显示 `D:\`，其余取最后一级目录名 */
  function rootLabel(r: string): string {
    if (/^[a-zA-Z]:$/.test(r)) return r + '\\'
    return baseName(r) || r
  }

  /**
   * 把绝对目录缩成带根名的短路径，供列表「所在位置」列展示。
   * 根目录本身返回根目录名（如 `Photos`），而不是无意义的 `.`；
   * 子目录返回 `Photos\2024\旅行`；不在任何根下的原样返回。
   */
  function relDir(dir: string): string {
    if (!dir) return ''
    const trimmed = dir.replace(/[\\/]+$/, '')
    const d = trimmed.toLowerCase().replace(/\//g, '\\')
    for (const r of sortedRoots.value) {
      const rl = r.toLowerCase().replace(/\//g, '\\')
      if (d === rl) return rootLabel(r)
      if (d.startsWith(rl + '\\')) {
        const rest = trimmed.slice(r.length + 1).replace(/\//g, '\\')
        const label = rootLabel(r)
        if (!rest) return label
        // 盘根（D:\）自带尾分隔符，不能再补一个
        return label.endsWith('\\') ? label + rest : `${label}\\${rest}`
      }
    }
    return dir
  }

  const progressPercent = computed(() => {
    if (!progress.total) return 0
    return Math.round((progress.done / progress.total) * 100)
  })

  // ---------- 提示 ----------
  let toastTimer: number | undefined
  function notify(type: ToastType, text: string) {
    toast.value = { type, text }
    window.clearTimeout(toastTimer)
    toastTimer = window.setTimeout(() => (toast.value = null), type === 'err' ? 5200 : 3000)
  }

  // ---------- 选择 ----------
  function toggleSelect(id: string) {
    const s = new Set(selected.value)
    if (s.has(id)) s.delete(id)
    else s.add(id)
    selected.value = s
  }

  function selectAll() {
    if (selected.value.size === filtered.value.length && filtered.value.length > 0) {
      selected.value = new Set()
    } else {
      selected.value = new Set(filtered.value.map((f) => f.id))
    }
  }

  function clearSelection() {
    selected.value = new Set()
  }

  /**
   * 勾选与列表不同步时（重扫后 id 全变）自动清理。
   * 否则会出现「有勾选、但一个都匹配不上」的中间态 —— 那种情况下如果
   * 把空目标当成「全部」传给后端，就会违背用户意图去动所有文件。
   */
  function ensureSelectionValid() {
    if (!selected.value.size) return
    const alive = new Set(files.value.map((f) => f.id))
    for (const id of selected.value) {
      if (!alive.has(id)) {
        selected.value = new Set()
        return
      }
    }
  }

  // ---------- 主题 ----------
  function setTheme(t: 'light' | 'dark') {
    theme.value = t
    document.documentElement.setAttribute('data-theme', t)
    try {
      localStorage.setItem('lcfm-theme', t)
    } catch {
      /* 隐私模式下忽略 */
    }
  }

  function initTheme() {
    let t: 'light' | 'dark' = 'light'
    try {
      const saved = localStorage.getItem('lcfm-theme')
      if (saved === 'dark' || saved === 'light') t = saved
      else if (window.matchMedia?.('(prefers-color-scheme: dark)').matches) t = 'dark'
    } catch {
      /* ignore */
    }
    setTheme(t)
  }

  // ---------- 事件监听（只注册一次） ----------
  let listenersReady = false
  async function ensureListeners() {
    if (listenersReady) return
    listenersReady = true
    if (isMock) return
    try {
      await onProgress((p) => {
        phase.value = p.phase
        progress.done = p.done
        progress.total = p.total
        progress.current = p.current
      })
      await onTaskDone(() => {
        /* 报告由调用方处理 */
      })
    } catch {
      /* 非 Tauri 环境（浏览器预览）忽略 */
    }
  }

  // ---------- 扫描 ----------
  async function pickDirs() {
    // 演示模式没有系统对话框，直接载入示例数据
    if (isMock) {
      await scan({ dirs: ['demo'] })
      return
    }
    const picked = await open({ directory: true, multiple: true, title: '选择文件夹' })
    if (!picked) return
    const dirs = (Array.isArray(picked) ? picked : [picked]).map((p) =>
      p.replace(/[\\/]+$/, ''),
    )
    // 追加而不是替换：用户可能分几次添加不同文件夹
    await scan({ dirs: Array.from(new Set([...roots.value, ...dirs])) })
  }

  async function scan(partial: Partial<ScanOptions>) {
    await ensureListeners()
    const opts: ScanOptions = {
      dirs: [],
      recursive: true,
      includeHidden: false,
      includeDirs: false,
      exts: [],
      nameFilter: '',
      useRegex: false,
      ...partial,
    }
    if (!opts.dirs.length) return

    busy.value = true
    phase.value = 'scan'
    progress.done = 0
    progress.total = 0
    try {
      const out = await api.scanFiles(opts)
      files.value = out.entries
      roots.value = out.roots
      selected.value = new Set()
      plan.value = null
      if (out.truncated) {
        notify('info', `文件数量超过上限，仅载入前 ${out.entries.length} 个`)
      } else if (!out.entries.length) {
        notify('info', '没有匹配到文件')
      } else {
        notify('ok', `已载入 ${out.entries.length} 个条目`)
      }
      if (out.errors.length) {
        console.warn('扫描告警：', out.errors.slice(0, 5))
      }
      await runPreview()
    } catch (e) {
      notify('err', errText(e))
    } finally {
      busy.value = false
      phase.value = ''
    }
  }

  async function rescan() {
    if (roots.value.length) {
      await scan({ dirs: [...roots.value] })
      return
    }
    // 兜底：没有记录根目录时按文件所在目录扫
    const dirs = Array.from(new Set(files.value.map((f) => f.dir))).filter(Boolean)
    if (!dirs.length) return
    await scan({ dirs })
  }

  /** 从列表里去掉某个已添加的文件夹，并只重扫剩下的 */
  async function removeRoot(dir: string) {
    if (busy.value) return
    const rest = roots.value.filter((r) => r !== dir)
    if (!rest.length) {
      await clearAll()
      return
    }
    await scan({ dirs: rest })
  }

  async function clearAll() {
    await api.clearFiles()
    files.value = []
    selected.value = new Set()
    plan.value = null
    roots.value = []
  }

  // ---------- 预览 ----------
  let previewTimer: number | undefined
  function schedulePreview(delay = 160) {
    window.clearTimeout(previewTimer)
    previewTimer = window.setTimeout(() => void runPreview(), delay)
  }

  /** 预览请求序号：勾选快速变化时可能有多个请求在飞，只认最新一份结果 */
  let previewSeq = 0

  async function runPreview() {
    if (!files.value.length) {
      plan.value = null
      return
    }
    ensureSelectionValid()
    const seq = ++previewSeq
    try {
      const snapshot = JSON.parse(JSON.stringify(rules)) as RenameRules
      // 关键：把勾选作为 targets 传给后端，Plan 只覆盖这些文件，
      // 于是 changedCount / errors / 执行范围 全部与勾选一致。
      const p = await api.previewRename(snapshot, selectedPaths.value)
      if (seq !== previewSeq) return
      plan.value = p
    } catch (e) {
      if (seq !== previewSeq) return
      plan.value = null
      notify('err', errText(e))
    }
  }

  // ---------- 执行 ----------
  async function executeRename() {
    if (!plan.value) return
    if (plan.value.ruleError) {
      notify('err', plan.value.ruleError)
      return
    }
    if (plan.value.errors > 0) {
      notify('err', `有 ${plan.value.errors} 个文件存在冲突，请先处理`)
      return
    }
    if (plan.value.changed === 0) {
      notify('info', '没有需要改名的文件')
      return
    }

    busy.value = true
    phase.value = 'rename'
    progress.done = 0
    progress.total = plan.value.changed
    try {
      const r: BatchReport = await api.executeRename()
      files.value = await api.getFiles()
      selected.value = new Set()
      const msg = r.failed.length
        ? `成功 ${r.ok} 个，失败 ${r.failed.length} 个`
        : `已改名 ${r.ok} 个文件`
      notify(r.failed.length ? 'err' : 'ok', msg)
      if (r.failed.length) {
        console.warn(
          '失败明细：',
          r.failed.map((f) => `${f.path} → ${f.reason}`),
        )
      }
      await runPreview()
      await loadJournals()
    } catch (e) {
      notify('err', errText(e))
    } finally {
      busy.value = false
      phase.value = ''
    }
  }

  async function applyTimes(times: TimeSpec | null, shift: TimeSpec | null) {
    const targets = targetPaths()
    const scope = targets.length ? `选中的 ${targets.length} 个文件` : '全部文件'
    busy.value = true
    phase.value = 'times'
    try {
      const r = await api.applyTimes({ targets, times, shift })
      notify(
        r.failed.length ? 'err' : 'ok',
        `已处理 ${scope}：成功 ${r.ok} 个${r.failed.length ? `，失败 ${r.failed.length} 个` : ''}`,
      )
      if (r.failed.length) console.warn('失败明细：', r.failed)
      await rescanQuiet()
      await loadJournals()
    } catch (e) {
      notify('err', errText(e))
    } finally {
      busy.value = false
      phase.value = ''
    }
  }

  async function applyAttrs(attrs: AttrSpec) {
    const targets = targetPaths()
    const scope = targets.length ? `选中的 ${targets.length} 个文件` : '全部文件'
    busy.value = true
    phase.value = 'attrs'
    try {
      const r = await api.applyAttrs({ targets, attrs })
      notify(
        r.failed.length ? 'err' : 'ok',
        `已处理 ${scope}：成功 ${r.ok} 个${r.failed.length ? `，失败 ${r.failed.length} 个` : ''}`,
      )
      if (r.failed.length) console.warn('失败明细：', r.failed)
      await rescanQuiet()
      await loadJournals()
    } catch (e) {
      notify('err', errText(e))
    } finally {
      busy.value = false
      phase.value = ''
    }
  }

  /** 静默重扫（保留勾选、不弹提示） */
  async function rescanQuiet() {
    const dirs = roots.value.length
      ? [...roots.value]
      : Array.from(new Set(files.value.map((f) => f.dir))).filter(Boolean)
    if (!dirs.length) return
    // 扫描后 id 会重新生成，所以用路径来保留勾选状态
    const keepPaths = new Set(
      files.value.filter((f) => selected.value.has(f.id)).map((f) => f.path),
    )
    try {
      const out = await api.scanFiles({
        dirs,
        recursive: true,
        includeHidden: false,
        includeDirs: false,
        exts: [],
        nameFilter: '',
        useRegex: false,
      })
      files.value = out.entries
      roots.value = out.roots
      selected.value = new Set(
        out.entries.filter((f) => keepPaths.has(f.path)).map((f) => f.id),
      )
      await runPreview()
    } catch {
      /* 静默失败，用户还有手动刷新 */
    }
  }

  // ---------- 撤销 ----------
  async function loadJournals() {
    try {
      journals.value = await api.listJournals()
    } catch {
      journals.value = []
    }
  }

  async function undo(id: string) {
    // 撤销改名后后端会清空内存列表，这里先记下目录用于重新扫描
    const dirs = roots.value.length
      ? [...roots.value]
      : Array.from(new Set(files.value.map((f) => f.dir))).filter(Boolean)
    busy.value = true
    phase.value = 'undo'
    try {
      const r = await api.undoJournal(id)
      notify(
        r.failed.length ? 'err' : 'ok',
        `已撤销 ${r.ok} 项${r.failed.length ? `，${r.failed.length} 项失败` : ''}`,
      )
      const fresh = await api.getFiles()
      if (fresh.length) {
        files.value = fresh
        selected.value = new Set()
        await runPreview()
      } else if (dirs.length) {
        // 后端已清空，按原目录重新载入
        await scan({ dirs })
      }
      await loadJournals()
    } catch (e) {
      notify('err', errText(e))
    } finally {
      busy.value = false
      phase.value = ''
    }
  }

  async function cancel() {
    try {
      await api.cancelTask()
      notify('info', '正在停止…')
    } catch {
      /* ignore */
    }
  }

  // ---------- 规则重置 ----------
  function resetRules() {
    Object.assign(rules, emptyRules())
    void runPreview()
  }

  // 勾选变化立刻重算预览：保证「将改动 N 个」与勾选实时一致
  watch(
    () => selected.value,
    () => {
      if (!busy.value && files.value.length) schedulePreview(32)
    },
  )

  return {
    // 数据
    files,
    selected,
    plan,
    rules,
    journals,
    roots,
    // 界面
    panel,
    theme,
    query,
    sidebarOpen,
    // 任务
    busy,
    phase,
    progress,
    progressPercent,
    toast,
    // 派生
    filtered,
    selectedPaths,
    selectedIds,
    selectedCount,
    visibleCount,
    hasSelection,
    changedCount,
    errorCount,
    warnCount,
    scopeCount,
    scopeLabel,
    targetFiles,
    canExecuteRename,
    // 方法
    notify,
    toggleSelect,
    selectAll,
    clearSelection,
    setTheme,
    initTheme,
    pickDirs,
    scan,
    rescan,
    removeRoot,
    clearAll,
    relDir,
    schedulePreview,
    runPreview,
    executeRename,
    applyTimes,
    applyAttrs,
    loadJournals,
    undo,
    cancel,
    resetRules,
    targetPaths,
  }
})
