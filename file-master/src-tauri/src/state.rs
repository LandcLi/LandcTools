//! 全局状态：当前文件列表、最近一次预览计划、撤销历史。
//! 所有跨 command 共享的数据都放这里，避免前端来回搬运大数组。

use crate::core::journal::Journal;
use crate::core::rename::RenamePlan;
use crate::model::FileEntry;
use parking_lot::RwLock;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct AppState {
    pub files: RwLock<Vec<FileEntry>>,
    pub plan: RwLock<Option<RenamePlan>>,
    pub journals: RwLock<Vec<Journal>>,
    /// 当前长任务的取消标志，点「停止」时置 true
    pub cancel: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            files: RwLock::new(Vec::new()),
            plan: RwLock::new(None),
            journals: RwLock::new(Vec::new()),
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn reset_cancel(&self) {
        self.cancel.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(std::sync::atomic::Ordering::SeqCst)
    }
}

pub type SharedState = Arc<AppState>;
