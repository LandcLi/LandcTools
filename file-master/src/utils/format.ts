export function formatSize(bytes: number): string {
  if (!bytes) return '0 B'
  if (bytes < 1024) return `${bytes} B`
  const units = ['KB', 'MB', 'GB', 'TB', 'PB']
  let v = bytes / 1024
  let i = 0
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024
    i++
  }
  return `${v < 10 ? v.toFixed(1) : Math.round(v)} ${units[i]}`
}

const pad = (n: number) => String(n).padStart(2, '0')

export function formatTime(ms: number | null | undefined, withSeconds = false): string {
  if (ms == null) return '—'
  const d = new Date(ms)
  if (Number.isNaN(d.getTime())) return '—'
  const base = `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
  return withSeconds ? `${base}:${pad(d.getSeconds())}` : base
}

/** 毫秒 → input[type=datetime-local] 需要的本地时间字符串 */
export function msToLocalInput(ms: number | null | undefined): string {
  if (ms == null) return ''
  const d = new Date(ms)
  if (Number.isNaN(d.getTime())) return ''
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

/** input[type=datetime-local] 字符串 → 毫秒 */
export function localInputToMs(s: string): number | null {
  if (!s) return null
  const t = new Date(s).getTime()
  return Number.isNaN(t) ? null : t
}

/** 对比新旧文件名，切出「相同前缀 / 变化部分 / 相同后缀」，用于差异高亮 */
export function diffParts(oldS: string, newS: string) {
  const maxPrefix = Math.min(oldS.length, newS.length)
  let p = 0
  while (p < maxPrefix && oldS[p] === newS[p]) p++

  let s = 0
  const maxSuffix = Math.min(oldS.length, newS.length) - p
  while (s < maxSuffix && oldS[oldS.length - 1 - s] === newS[newS.length - 1 - s]) s++

  return {
    prefix: newS.slice(0, p),
    changed: newS.slice(p, newS.length - s),
    suffix: s ? newS.slice(newS.length - s) : '',
  }
}

/** 毫秒 → 人类可读的时长差，用于「时间偏移」预览 */
export function formatDuration(ms: number): string {
  if (ms === 0) return '0'
  const abs = Math.abs(ms)
  const day = 86400000
  const hour = 3600000
  const minute = 60000
  const parts: string[] = []
  let rest = abs
  if (rest >= day) {
    const d = Math.floor(rest / day)
    parts.push(`${d} 天`)
    rest -= d * day
  }
  if (rest >= hour && parts.length < 2) {
    const h = Math.floor(rest / hour)
    parts.push(`${h} 小时`)
    rest -= h * hour
  }
  if (rest >= minute && parts.length < 2) {
    const m = Math.floor(rest / minute)
    parts.push(`${m} 分`)
  }
  if (!parts.length) parts.push(`${Math.round(abs / 1000)} 秒`)
  return (ms < 0 ? '-' : '+') + parts.join('')
}

export const KIND_LABEL: Record<string, string> = {
  image: '图片',
  office: 'Office',
  pdf: 'PDF',
  audio: '音频',
  video: '视频',
  archive: '压缩包',
  exec: '程序',
  text: '文本',
  folder: '文件夹',
  other: '其他',
}
