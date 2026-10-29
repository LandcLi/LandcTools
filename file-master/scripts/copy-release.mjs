// 发布收尾脚本：把编译好的 exe 复制到 release/，并做三项硬校验。
//
// 存在的意义：`cargo build --release` 直接编译出来的 exe 是「开发版」——
// 里面嵌的是 devUrl(http://localhost:5183)，双击会报「localhost 拒绝连接」。
// 这个错误在编译期完全静默，只有在用户双击时才暴露，所以必须在这里拦住。
//
// 用法：npm run build:exe  （内部会先跑 tauri build --no-bundle）

import { existsSync, readdirSync, readFileSync, statSync, copyFileSync, mkdirSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = dirname(dirname(fileURLToPath(import.meta.url)))
const exePath = join(root, 'src-tauri', 'target', 'release', 'landc-file-master.exe')
const distIndex = join(root, 'dist', 'index.html')
const buildDir = join(root, 'src-tauri', 'target', 'release', 'build')
const outDir = join(root, 'release')

const fail = (msg) => {
  console.error(`\n[失败] ${msg}\n`)
  process.exit(1)
}

// ---- 校验 1：exe 存在 ----
if (!existsSync(exePath)) fail(`找不到编译产物：${exePath}\n请先执行 npm run build:exe`)

// ---- 校验 2：exe 不比 dist 旧（否则前端改动没嵌进去）----
if (!existsSync(distIndex)) fail(`找不到前端产物：${distIndex}`)
const exeTime = statSync(exePath).mtimeMs
const distTime = statSync(distIndex).mtimeMs
if (distTime > exeTime) {
  fail(
    `exe 比 dist 旧，说明前端改动没有被嵌入。\n` +
      `  dist/index.html : ${new Date(distTime).toLocaleString()}\n` +
      `  exe             : ${new Date(exeTime).toLocaleString()}\n` +
      `请用 npm run build:exe（它会先跑 tauri build 触发前端重建）`,
  )
}

// ---- 校验 3：确认不是「开发版」binary ----
// tauri-build 会为每个 crate 生成 build/*/output，里面记录 rustc-cfg。
// 生产构建应出现 custom_protocol，开发构建会出现 dev。
if (!existsSync(buildDir)) fail(`找不到构建缓存目录：${buildDir}`)

const outputs = readdirSync(buildDir)
  .filter((d) => d.startsWith('landc-file-master-'))
  .map((d) => join(buildDir, d, 'output'))
  .filter((p) => existsSync(p))

if (!outputs.length) fail('找不到主 crate 的构建记录，无法确认是否为生产构建')

// 取最新的一份
let newest = null
let newestTime = 0
for (const p of outputs) {
  const t = statSync(p).mtimeMs
  if (t > newestTime) {
    newestTime = t
    newest = p
  }
}

const cfgText = readFileSync(newest, 'utf8')
if (/cargo:rustc-cfg=dev\b/.test(cfgText)) {
  fail(
    `编译出来的 exe 是开发版（cfg=dev），双击会报「localhost 拒绝连接」。\n` +
      `  原因：直接用了 cargo build，没有加 --features custom-protocol。\n` +
      `  正确做法：npm run build:exe（内部调用 tauri build，会自动加该 feature）\n` +
      `  证据文件：${newest}`,
  )
}

// ---- 通过，复制到 release/ ----
if (!existsSync(outDir)) mkdirSync(outDir, { recursive: true })
const target = join(outDir, 'LandcFileMaster.exe')

try {
  copyFileSync(exePath, target)
} catch (e) {
  if (e.code === 'EBUSY' || e.code === 'EPERM') {
    fail(
      `目标文件被占用，无法覆盖：${target}\n` +
        `  原因：LandcFileMaster.exe 正在运行中。\n` +
        `  处理：先把正在运行的窗口关掉，再重跑 npm run build:exe\n` +
        `        （新 exe 已编译好，关掉旧进程后可直接再执行一次本脚本）`,
    )
  }
  throw e
}

const mb = (statSync(target).size / 1024 / 1024).toFixed(2)
console.log(`\n[通过] 三项校验全部通过`)
console.log(`  产物：${target}`)
console.log(`  体积：${mb} MB`)
console.log(`  模式：生产构建（前端已内嵌，无需连接任何服务器）\n`)
