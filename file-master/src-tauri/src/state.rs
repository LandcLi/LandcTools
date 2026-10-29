//! 全局状态：当前文件列表、最近一次预览计划、任务取消标志。
//! 所有跨 command 共享的数据都放这里，避免前端来回搬运大数组。
//! 撤销日志写入磁盘（见 `core/journal.rs`），不放内存。

use crate::core::rename::RenamePlan;
use crate::model::FileEntry;
use parking_lot::RwLock;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct AppState {
    pub files: RwLock<Vec<FileEntry>>,
    pub plan: RwLock<Option<RenamePlan>>,
    /// 当前长任务的取消标志，点「停止」时置 true
    pub cancel: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            files: RwLock::new(Vec::new()),
            plan: RwLock::new(None),
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn reset_cancel(&self) {
        self.cancel.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

pub type SharedState = Arc<AppState>;
