# LandcTools

Landc 系列桌面工具集合。每个工具都是一个独立项目，可独立开发、独立打包、独立售卖。

## 工具清单

| 目录 | 产品名 | 说明 | 状态 |
| --- | --- | --- | --- |
| `file-master/` | Landc 文件大师 / FileMaster | 批量改名、批量改时间与属性、批量改后缀、Office/PDF/音频/图片元数据编辑 | 开发中 |

## 目录约定

- 每个子目录 = 一个完整可交付项目，自带 `README.md`、构建脚本、发版说明
- 工具之间不共享源码；需要复用的通用代码以独立 crate / npm 包形式各自引用，避免耦合
- 每个项目分开 `git init`，方便单独开源或单独出售

## 技术栈基线

| 层 | 选型 |
| --- | --- |
| 壳 | Tauri 2（Windows 绿色单 exe，无安装器） |
| 前端 | Vue 3.5 + TypeScript + Vite 6 + Pinia |
| 后端 | Rust（edition 2021, MSVC） |
| UI | 自研轻量设计系统（CSS 变量 + 双主题，不依赖重型组件库） |

## 本地开发

```bash
# Rust 工具链不在 Git Bash PATH 里时，先补上
export PATH="$HOME/.cargo/bin:$PATH"

cd file-master
npm install
npm run tauri:dev     # 开发
npm run tauri:build   # 打包
```
