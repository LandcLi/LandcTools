//! 操作日志（undo journal）。
//!
//! 每次批量操作成功后落一个 JSON 文件到 `%APPDATA%/landc.file.master/journals/`。
//! 撤销时逆序回放。保留最近 100 条，超出的自动清理。
//!
//! 这是「敢卖」的底气 —— 用户改错了能一键还原，售后少一半。

use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
    /// 操作后的路径（撤销时的起点）
    pub after: String,
    /// 操作前的路径或原值
    pub before: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Journal {
    pub id: String,
    /// 展示用标题，如「批量改名 128 个文件」
    pub title: String,
    /// rename / times / attrs / ext
    pub kind: String,
    pub created_at: i64,
    pub entries: Vec<JournalEntry>,
    pub undone: bool,
}

pub fn base_dir() -> PathBuf {
    let root = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    root.join("landc.file.master")
}

pub fn journal_dir() -> PathBuf {
    base_dir().join("journals")
}

pub fn new_journal(title: impl Into<String>, kind: impl Into<String>) -> Journal {
    Journal {
        id: uuid::Uuid::new_v4().to_string(),
        title: title.into(),
        kind: kind.into(),
        created_at: chrono::Utc::now().timestamp_millis(),
        entries: Vec::new(),
        undone: false,
    }
}

pub fn save(j: &Journal) -> AppResult<()> {
    let dir = journal_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.json", j.id));
    let data = serde_json::to_string_pretty(j)
        .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn load_all() -> Vec<Journal> {
    let dir = journal_dir();
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return out;
    };
    for e in rd.flatten() {
        if e.path().extension().map(|x| x == "json").unwrap_or(false) {
            if let Ok(text) = std::fs::read_to_string(e.path()) {
                if let Ok(j) = serde_json::from_str::<Journal>(&text) {
                    out.push(j);
                }
            }
        }
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    out
}

pub fn mark_undone(id: &str) -> AppResult<()> {
    let path = journal_dir().join(format!("{id}.json"));
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(mut j) = serde_json::from_str::<Journal>(&text) {
            j.undone = true;
            return save(&j);
        }
    }
    Ok(())
}

/// 只保留最近 keep 条
pub fn prune(keep: usize) {
    let all = load_all();
    if all.len() <= keep {
        return;
    }
    for j in all.into_iter().skip(keep) {
        let _ = std::fs::remove_file(journal_dir().join(format!("{}.json", j.id)));
    }
}

/// 回滚一条 rename 记录：after → before
pub fn undo_rename_entry(e: &JournalEntry) -> Result<(), String> {
    let from = std::path::Path::new(&e.after);
    let to = std::path::Path::new(&e.before);
    if !from.exists() {
        return Err(format!("文件已不在原位置：{}", e.after));
    }
    if to.exists() {
        return Err(format!("原位置已被占用：{}", e.before));
    }
    if let Some(parent) = to.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
    }
    std::fs::rename(from, to).map_err(|err| crate::core::executor::friendly_io_error(&err))
}
