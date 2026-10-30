//! Landc 文件大师 —— 后端入口。
//!
//! 分层：
//! - `commands/`：Tauri 命令，只做参数整理与事件推送
//! - `core/`：纯业务逻辑，可单测、不依赖 Tauri
//! - `state.rs`：跨命令共享状态
//! - `model.rs` / `error.rs`：公共模型与错误

mod commands;
pub mod core;
mod error;
pub mod model;
pub mod preflight;
mod state;

use state::{AppState, SharedState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shared: SharedState = std::sync::Arc::new(AppState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(shared)
        .invoke_handler(tauri::generate_handler![
            commands::scan_files,
            commands::get_files,
            commands::clear_files,
            commands::cancel_task,
            commands::preview_rename,
            commands::execute_rename,
            commands::apply_times,
            commands::apply_attrs,
            commands::list_journals,
            commands::undo_journal,
            commands::get_file_detail,
            commands::read_meta,
            commands::apply_meta,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Landc 文件大师失败");
}
