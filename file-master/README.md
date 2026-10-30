# Landc 文件大师 / FileMaster

Windows 上的批量文件处理工具。海量文件一键统一更名、批量改时间与属性、批量换后缀、编辑文档与媒体元数据。

面向普通用户设计：**所有操作先预览、再执行，改错了一键撤销**。

## 功能

| 模块 | 能力 | 状态 |
| --- | --- | --- |
| 批量改名 | 查找替换（支持正则与捕获组）、剔除指定字符、批量加前缀/后缀、自动编号（起始/步长/补零/自定义模板）、大小写转换、去空格、全角转半角 | ✅ v0.1 |
| 后缀转换 | 批量替换后缀、统一转大小写、追加多级后缀（`.tar` → `.tar.gz`） | ✅ v0.1 |
| 时间属性 | 创建时间 / 修改时间 / 访问时间，支持「设为指定时间」与「按原值加减偏移」 | ✅ v0.1 |
| 文件属性 | 只读 / 隐藏 / 系统 / 存档，四属性独立三态控制 | ✅ v0.1 |
| 撤销历史 | 每次批量操作自动记录，任意一条可整体回滚 | ✅ v0.1 |
| 标签编辑 | Office 属性（docx/xlsx/pptx）、PDF Info、音乐标签（MP3/FLAC/M4A/WAV/OGG）、图片 EXIF（JPG/PNG/TIFF/WebP/HEIC），支持批量与撤销 | ✅ v0.2 |

## 技术栈

| 层 | 选型 | 说明 |
| --- | --- | --- |
| 外壳 | Tauri 2 | 单个 exe，约 8–12MB，无需安装运行时 |
| 前端 | Vue 3.5 + TypeScript + Vite 6 + Pinia | 产物 177KB JS / 27KB CSS |
| 后端 | Rust 2021 (MSVC) | 批量 IO 与 Win32 调用 |
| UI | 自研设计系统 | 双主题、虚拟滚动，不依赖重型组件库 |

## 目录结构

```
file-master/
├─ src/                        前端
│  ├─ api/                     invoke 封装与类型定义（与 Rust serde 模型一一对应）
│  ├─ stores/app.ts            单一状态源：文件、规则、预览计划、任务进度
│  ├─ components/
│  │  ├─ TopBar / Sidebar / FileTable / StatusBar
│  │  ├─ ui/                   LcButton / LcInput / LcSelect / LcSwitch / ...
│  │  └─ panels/               六个功能面板
│  ├─ styles/                  tokens.css（设计令牌） + base.css
│  └─ utils/format.ts          体积/时间格式化、文件名差异切分
├─ src-tauri/src/
│  ├─ commands/                Tauri 命令层，只做参数整理与事件推送
│  ├─ core/
│  │  ├─ scan.rs               目录扫描（walkdir + 自然序排序）
│  │  ├─ rename.rs             改名规则引擎 + 预览计划构建
│  │  ├─ conflicts.rs          非法字符 / 保留设备名 / 路径长度 / 重名检测
│  │  ├─ executor.rs           两阶段安全改名执行器
│  │  ├─ journal.rs            撤销日志（JSON 落盘，保留最近 100 条）
│  │  ├─ times.rs              Win32 CreateFileW + Get/SetFileTime
│  │  └─ attrs.rs              Win32 Get/SetFileAttributesW
│  ├─ model.rs                 FileEntry 等公共模型
│  ├─ state.rs                 跨命令共享状态
│  └─ error.rs                 统一错误类型
└─ scripts/gen-icon.mjs        零依赖图标生成器（手写 PNG 编码）
```

## 三个关键设计

### 1. 三段式：预览 → 冲突检查 → 执行

用户改规则时，前端把规则发给后端算出完整 `RenamePlan`，逐条标注 `ok / warn / error`。
**只要存在一条 error，整批不允许执行**。这是批量改名工具不把用户文件搞废的前提。

### 2. 两阶段改名（解决连环改名）

当一批文件里存在 `a→b`、`b→c` 这种首尾相接的改名时，直接顺序执行会把 `b` 覆盖掉。
执行器检测到「某条的目标路径是本批次的另一个源路径」时，整批切换到：

```
阶段一：所有源文件 → 唯一临时名 .lcfm_<uuid>_<i>
阶段二：临时名 → 目标名（失败则回滚为原名）
```

代价是多一次 rename 系统调用（几乎无感），换来的是绝对安全。

### 3. 撤销日志

每次批量操作成功后落一个 JSON 到 `%APPDATA%/landc.file.master/journals/`，
撤销时逆序回放。改名记路径对，时间与属性记原值快照。

## 开发

```bash
# Rust 工具链不在 Git Bash PATH 时先补上
export PATH="$HOME/.cargo/bin:$PATH"

npm install
npm run tauri:dev      # 开发（会启动 vite + tauri）
npm run typecheck      # 前端类型检查
cargo check            # 后端类型检查（在 src-tauri 目录）
```

## 打包

```bash
npm run build:exe
```

一条命令搞定：`tauri build --no-bundle`（自动跑前端构建 + 带 `custom-protocol` 编译）
→ `scripts/copy-release.mjs` 做三项校验 → 复制到 `release/LandcFileMaster.exe`。

### ⚠️ 不要直接 `cargo build --release`

Tauri 2 用 `custom-protocol` 这个 Cargo feature 区分两种模式：

| 编译方式 | 结果 | 双击表现 |
| --- | --- | --- |
| `tauri build`（带 `--features custom-protocol`） | 前端 `dist/` 内嵌进 exe | 正常打开 |
| `cargo build --release`（不带该 feature） | 只嵌入 `devUrl` | **报「localhost 拒绝连接」** |

第二种编译产物**编译期零报错**，只有用户双击时才暴露，是最难排查的一类问题。
`scripts/copy-release.mjs` 会读取 `target/release/build/*/output` 里的 `rustc-cfg`，
一旦发现是 `dev` 构建就直接 `exit 1`，从源头拦住。

同理，`custom-protocol` **不能写进 `[features] default`** —— 否则 `tauri dev` 也会去加载
内嵌资源，热更新失效。让它只由 `tauri build` 显式带上。

> `dist/` 变化后 `tauri build` 会正确重编主 crate；早期用 `cargo build` 时遇到的
> 「必须 touch src/lib.rs 才会重编」问题，用 `tauri build` 后不再存在。

### 关于 WebView2 依赖

绿色 exe 没有安装器，Tauri 的 `webviewInstallMode` 对便携版无效。缺 WebView2 时
Tauri 会在创建窗口阶段失败，表现为**双击毫无反应**——这是最伤售后的失败模式。

因此 `src-tauri/src/preflight.rs` 在 `main()` 最前面调用 `tauri::webview_version()`
做前置探测：缺失时弹出中文对话框说明原因，并可用 `ShellExecuteW` 一键打开微软官方下载页。

实际覆盖率：Windows 11 全部预装，Windows 10 绝大多数通过 Edge 更新已内置；
缺失集中在 LTSC / Server / 精简镜像 / 企业策略禁用的机器上。

## 数据目录

| 内容 | 路径 |
| --- | --- |
| 撤销日志 | `%APPDATA%/landc.file.master/journals/*.json` |
| 主题偏好 | WebView localStorage |

## 路线图

- [x] 标签编辑：docx/xlsx/pptx（OPC `docProps/core.xml`）、PDF Info 字典、音频标签（ID3/FLAC/MP4）、图片 EXIF
- [ ] 按文件名解析日期并写入时间属性
- [ ] 改名规则方案保存与复用
- [ ] 拖拽文件/文件夹到窗口即载入
- [ ] 右键菜单集成（资源管理器「用文件大师打开」）
