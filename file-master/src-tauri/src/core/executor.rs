//! 批量执行引擎。
//!
//! 核心是**两阶段改名**：当一批文件里存在 `a→b`、`b→c` 这种连环改名时，
//! 直接按顺序改会把 b 覆盖掉。做法是先把所有源文件改成唯一临时名，
//! 再统一改成目标名。代价是 2 倍 rename 系统调用（几乎无感），换来的是绝对安全。

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ExecItem {
    pub old_path: String,
    pub new_path: String,
    pub needs_two_phase: bool,
}

#[derive(Debug, Clone)]
pub struct ExecFailure {
    pub old_path: String,
    pub new_path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct ExecResult {
    pub ok: Vec<(String, String)>,
    pub failed: Vec<ExecFailure>,
    pub cancelled: bool,
}

/// 执行改名。`on_progress(done, total, current_path)`，`is_cancelled()` 返回 true 时尽快停下。
pub fn execute_rename<F, C>(
    items: &[ExecItem],
    mut on_progress: F,
    is_cancelled: C,
) -> ExecResult
where
    F: FnMut(usize, usize, &str),
    C: Fn() -> bool,
{
    let total = items.len();
    let mut result = ExecResult::default();
    if total == 0 {
        return result;
    }

    let two_phase = items.iter().any(|i| i.needs_two_phase);

    if !two_phase {
        // 简单路径：逐个直接改，出错即记
        for (i, item) in items.iter().enumerate() {
            if is_cancelled() {
                result.cancelled = true;
                break;
            }
            match fs::rename(&item.old_path, &item.new_path) {
                Ok(_) => result.ok.push((item.old_path.clone(), item.new_path.clone())),
                Err(e) => result.failed.push(ExecFailure {
                    old_path: item.old_path.clone(),
                    new_path: item.new_path.clone(),
                    reason: friendly_io_error(&e),
                }),
            }
            on_progress(i + 1, total, &item.new_path);
        }
        return result;
    }

    // ---- 阶段一：全部搬到临时名 ----
    let mut staged: Vec<Option<PathBuf>> = vec![None; total];
    for (i, item) in items.iter().enumerate() {
        if is_cancelled() {
            result.cancelled = true;
            break;
        }
        let src = Path::new(&item.old_path);
        let tmp = src.with_file_name(format!(
            ".lcfm_{}_{}",
            uuid::Uuid::new_v4().simple(),
            i
        ));
        match fs::rename(src, &tmp) {
            Ok(_) => staged[i] = Some(tmp),
            Err(e) => result.failed.push(ExecFailure {
                old_path: item.old_path.clone(),
                new_path: item.new_path.clone(),
                reason: format!("暂存阶段失败：{}", friendly_io_error(&e)),
            }),
        }
        // 阶段一占进度的 0~50%
        on_progress((i + 1) / 2, total, &item.old_path);
    }

    if result.cancelled {
        // 已暂存的先搬回去，避免留下一堆 .lcfm_ 临时文件
        for (i, slot) in staged.iter().enumerate() {
            if let Some(tmp) = slot {
                let _ = fs::rename(tmp, &items[i].old_path);
            }
        }
        result.failed.clear();
        return result;
    }

    // ---- 阶段二：临时名 → 目标名 ----
    for (i, item) in items.iter().enumerate() {
        let Some(tmp) = staged[i].as_ref() else {
            continue;
        };
        match fs::rename(tmp, &item.new_path) {
            Ok(_) => result.ok.push((item.old_path.clone(), item.new_path.clone())),
            Err(e) => {
                // 尽力回滚这一条
                let rolled_back = fs::rename(tmp, &item.old_path).is_ok();
                let mut reason = friendly_io_error(&e);
                if !rolled_back {
                    reason.push_str("（且回滚失败，原文件可能停留在临时文件名，请手动恢复）");
                }
                result.failed.push(ExecFailure {
                    old_path: item.old_path.clone(),
                    new_path: item.new_path.clone(),
                    reason,
                });
            }
        }
        // 阶段二占 50~100%
        on_progress(total / 2 + (i + 1) / 2, total, &item.new_path);
    }

    on_progress(total, total, "");
    result
}

pub fn friendly_io_error(e: &std::io::Error) -> String {
    use std::io::ErrorKind::*;
    match e.kind() {
        PermissionDenied => "没有权限或被其他程序占用".to_string(),
        NotFound => "文件不存在".to_string(),
        AlreadyExists => "目标文件已存在".to_string(),
        _ => {
            let raw = e.to_string();
            // Windows 常见错误码翻译成人话
            if raw.contains("os error 32") {
                "文件正被其他程序占用".to_string()
            } else if raw.contains("os error 5") {
                "拒绝访问（权限不足）".to_string()
            } else if raw.contains("os error 3") {
                "路径不存在".to_string()
            } else if raw.contains("os error 206") {
                "路径过长".to_string()
            } else {
                raw
            }
        }
    }
}

/// 简单重命名，用于单个操作
pub fn rename_one(old: &Path, new: &Path) -> Result<(), String> {
    fs::rename(old, new).map_err(|e| friendly_io_error(&e))
}
