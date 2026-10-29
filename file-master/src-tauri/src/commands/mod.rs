//! Tauri command 层：只做参数整理、状态读写、事件推送，业务逻辑都在 `core`。

use crate::core::attrs::{self, AttrSpec};
use crate::core::executor::{self, ExecItem};
use crate::core::journal::{self, Journal, JournalEntry};
use crate::core::rename::{self, RenamePlan, RenameRules};
use crate::core::scan::{self, ScanOptions, ScanOutcome};
use crate::core::times::{self, TimeSpec};
use crate::error::{AppError, AppResult};
use crate::model::FileEntry;
use crate::state::SharedState;
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressPayload {
    phase: String,
    done: usize,
    total: usize,
    current: String,
}

fn emit_progress(app: &AppHandle, phase: &str, done: usize, total: usize, current: &str) {
    let _ = app.emit(
        "task:progress",
        ProgressPayload {
            phase: phase.to_string(),
            done,
            total,
            current: current.to_string(),
        },
    );
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailureItem {
    pub path: String,
    pub reason: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchReport {
    pub ok: usize,
    pub failed: Vec<FailureItem>,
    pub cancelled: bool,
    pub journal_id: Option<String>,
    pub elapsed_ms: u64,
}

// ---------------------------------------------------------------------------
// 扫描
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn scan_files(
    app: AppHandle,
    state: State<'_, SharedState>,
    opts: ScanOptions,
) -> AppResult<ScanOutcome> {
    state.reset_cancel();
    let cancel = state.cancel.clone();
    let app2 = app.clone();

    let outcome = tokio::task::spawn_blocking(move || {
        scan::scan(
            &opts,
            |n, p| emit_progress(&app2, "scan", n, 0, p),
            || cancel.load(Ordering::Relaxed),
        )
    })
    .await
    .map_err(|e| AppError::Internal(format!("扫描任务异常：{e}")))??;

    *state.files.write() = outcome.entries.clone();
    *state.plan.write() = None;
    emit_progress(&app, "scan", outcome.scanned, outcome.scanned, "");
    Ok(outcome)
}

#[tauri::command]
pub fn get_files(state: State<'_, SharedState>) -> Vec<FileEntry> {
    state.files.read().clone()
}

#[tauri::command]
pub fn clear_files(state: State<'_, SharedState>) {
    state.files.write().clear();
    *state.plan.write() = None;
}

#[tauri::command]
pub fn cancel_task(state: State<'_, SharedState>) {
    state.cancel.store(true, Ordering::SeqCst);
}

// ---------------------------------------------------------------------------
// 改名预览 / 执行
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn preview_rename(
    state: State<'_, SharedState>,
    rules: RenameRules,
    targets: Option<Vec<String>>,
) -> AppResult<RenamePlan> {
    let files = state.files.read();
    if files.is_empty() {
        return Err(AppError::Invalid("请先添加文件或文件夹".into()));
    }

    // targets 为空表示「对全部文件生效」；非空时只对本批次内的文件建计划。
    // 这样勾选的语义才能贯穿到执行阶段（execute_rename 直接用这份 plan）。
    let filtered: Vec<FileEntry>;
    let source: &[FileEntry] = match targets {
        Some(t) if !t.is_empty() => {
            filtered = pick_by_targets(files.as_slice(), &t);
            if filtered.is_empty() {
                return Err(AppError::Invalid(
                    "选中的文件已不在列表中，请刷新后重试".into(),
                ));
            }
            &filtered
        }
        _ => files.as_slice(),
    };

    let plan = rename::build_plan(source, &rules)?;
    *state.plan.write() = Some(plan.clone());
    Ok(plan)
}

#[tauri::command]
pub async fn execute_rename(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> AppResult<BatchReport> {
    let plan = state
        .plan
        .read()
        .clone()
        .ok_or_else(|| AppError::Invalid("没有可执行的预览计划".into()))?;

    if plan.errors > 0 {
        return Err(AppError::Invalid(format!(
            "有 {} 个文件存在冲突，请先处理后再执行",
            plan.errors
        )));
    }

    let items: Vec<ExecItem> = plan
        .items
        .iter()
        .filter(|i| i.changed && i.level != crate::core::conflicts::Level::Error)
        .map(|i| ExecItem {
            old_path: i.old_path.clone(),
            new_path: i.new_path.clone(),
            needs_two_phase: i.needs_two_phase,
        })
        .collect();

    if items.is_empty() {
        return Err(AppError::Invalid("没有需要改名的文件".into()));
    }

    state.reset_cancel();
    let cancel = state.cancel.clone();
    let app2 = app.clone();
    let total_items = items.len();
    let started = std::time::Instant::now();

    let result = tokio::task::spawn_blocking(move || {
        executor::execute_rename(
            &items,
            |done, total, cur| emit_progress(&app2, "rename", done, total, cur),
            || cancel.load(Ordering::Relaxed),
        )
    })
    .await
    .map_err(|e| AppError::Internal(format!("执行任务异常：{e}")))?;

    // 写日志
    let mut j = journal::new_journal(
        format!("批量改名 {} 个文件", result.ok.len()),
        "rename",
    );
    j.entries = result
        .ok
        .iter()
        .map(|(old, new)| JournalEntry {
            after: new.clone(),
            before: old.clone(),
        })
        .collect();
    let journal_id = if !j.entries.is_empty() {
        let id = j.id.clone();
        journal::save(&j)?;
        journal::prune(100);
        Some(id)
    } else {
        None
    };

    // 就地更新内存中的文件列表，避免前端再扫一遍
    {
        let mut files = state.files.write();
        for (old, new) in &result.ok {
            if let Some(entry) = files.iter_mut().find(|f| &f.path == old) {
                let p = std::path::Path::new(new);
                entry.path = new.clone();
                entry.name = p
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                entry.stem = p
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                entry.ext = p
                    .extension()
                    .map(|s| s.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                entry.dir = p
                    .parent()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
            }
        }
        if result.cancelled {
            // 取消时剩余文件仍是旧名，稳妥起见让前端重新扫描
        }
    }
    *state.plan.write() = None;

    let _ = total_items;
    let report = BatchReport {
        ok: result.ok.len(),
        failed: result
            .failed
            .iter()
            .map(|f| FailureItem {
                path: f.old_path.clone(),
                reason: f.reason.clone(),
            })
            .collect(),
        cancelled: result.cancelled,
        journal_id,
        elapsed_ms: started.elapsed().as_millis() as u64,
    };
    let _ = app.emit("task:done", &report);
    Ok(report)
}

// ---------------------------------------------------------------------------
// 时间 / 属性 / 后缀
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchOptions {
    /// 目标文件路径，空数组表示「全部」
    #[serde(default)]
    pub targets: Vec<String>,
    /// 时间设置（绝对时间）
    #[serde(default)]
    pub times: Option<TimeSpec>,
    /// 相对偏移（毫秒），与 times 二选一
    #[serde(default)]
    pub shift: Option<TimeSpec>,
    #[serde(default)]
    pub attrs: Option<AttrSpec>,
}

// ---------------------------------------------------------------------------
// 目标筛选
// ---------------------------------------------------------------------------

/// 按 targets 从当前列表里挑出文件。
/// targets 的元素按 **path** 匹配；额外兼容 id（前端早期误传 UUID 时
/// 会静默变成零命中，进而被当成「全部」，这里直接让它匹配不上或命中，都更安全）。
fn pick_by_targets(files: &[FileEntry], targets: &[String]) -> Vec<FileEntry> {
    let set: std::collections::HashSet<&str> = targets.iter().map(|s| s.as_str()).collect();
    let exact: Vec<FileEntry> = files
        .iter()
        .filter(|f| set.contains(f.path.as_str()) || set.contains(f.id.as_str()))
        .cloned()
        .collect();
    if !exact.is_empty() {
        return exact;
    }
    // 大小写 / 分隔符风格不一致时降级做不区分大小写匹配
    let ci: std::collections::HashSet<String> = targets.iter().map(|s| s.to_lowercase()).collect();
    files
        .iter()
        .filter(|f| ci.contains(&f.path.to_lowercase()) || ci.contains(&f.id.to_lowercase()))
        .cloned()
        .collect()
}

fn pick_targets(state: &State<'_, SharedState>, targets: &[String]) -> Vec<FileEntry> {
    let files = state.files.read();
    if targets.is_empty() {
        files.iter().filter(|f| !f.is_dir).cloned().collect()
    } else {
        pick_by_targets(files.as_slice(), targets)
            .into_iter()
            .filter(|f| !f.is_dir)
            .collect()
    }
}

#[tauri::command]
pub async fn apply_times(
    app: AppHandle,
    state: State<'_, SharedState>,
    opts: BatchOptions,
) -> AppResult<BatchReport> {
    let targets = pick_targets(&state, &opts.targets);
    if targets.is_empty() {
        return Err(AppError::Invalid("没有可操作的文件".into()));
    }
    if opts.times.is_none() && opts.shift.is_none() {
        return Err(AppError::Invalid("没有设置要修改的时间".into()));
    }

    state.reset_cancel();
    let cancel = state.cancel.clone();
    let app2 = app.clone();
    let started = std::time::Instant::now();
    let total = targets.len();

    let out = tokio::task::spawn_blocking(move || {
        let mut ok_entries: Vec<JournalEntry> = Vec::new();
        let mut failed: Vec<FailureItem> = Vec::new();
        let mut cancelled = false;

        for (i, f) in targets.iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                cancelled = true;
                break;
            }
            let path = std::path::Path::new(&f.path);
            let res = if let Some(spec) = opts.times {
                times::write_times(path, &spec).map(|before| {
                    let before_json = serde_json::to_string(&before).unwrap_or_default();
                    let after_json = serde_json::to_string(&TimeSpec {
                        created_ms: spec.created_ms.or(before.created_ms),
                        modified_ms: spec.modified_ms.or(before.modified_ms),
                        accessed_ms: spec.accessed_ms.or(before.accessed_ms),
                    })
                    .unwrap_or_default();
                    (before_json, after_json)
                })
            } else if let Some(shift) = opts.shift {
                times::shift_times(
                    path,
                    shift.created_ms.unwrap_or(0),
                    shift.modified_ms.unwrap_or(0),
                    shift.accessed_ms.unwrap_or(0),
                )
                .map(|before| {
                    let before_json = serde_json::to_string(&before).unwrap_or_default();
                    let after_json = serde_json::to_string(&TimeSpec {
                        created_ms: before.created_ms.map(|v| v + shift.created_ms.unwrap_or(0)),
                        modified_ms: before.modified_ms.map(|v| v + shift.modified_ms.unwrap_or(0)),
                        accessed_ms: before.accessed_ms.map(|v| v + shift.accessed_ms.unwrap_or(0)),
                    })
                    .unwrap_or_default();
                    (before_json, after_json)
                })
            } else {
                unreachable!()
            };

            match res {
                Ok((before_json, after_json)) => {
                    ok_entries.push(JournalEntry {
                        after: f.path.clone(),
                        before: before_json,
                    });
                    let _ = after_json;
                }
                Err(e) => failed.push(FailureItem {
                    path: f.path.clone(),
                    reason: e.to_string(),
                }),
            }
            emit_progress(&app2, "times", i + 1, total, &f.path);
        }

        (ok_entries, failed, cancelled)
    })
    .await
    .map_err(|e| AppError::Internal(format!("任务异常：{e}")))?;

    let (ok_entries, failed, cancelled) = out;
    let mut j = journal::new_journal(format!("修改时间属性 {} 个文件", ok_entries.len()), "times");
    j.entries = ok_entries.clone();
    let journal_id = if !j.entries.is_empty() {
        let id = j.id.clone();
        journal::save(&j)?;
        Some(id)
    } else {
        None
    };

    Ok(BatchReport {
        ok: ok_entries.len(),
        failed,
        cancelled,
        journal_id,
        elapsed_ms: started.elapsed().as_millis() as u64,
    })
}

#[tauri::command]
pub async fn apply_attrs(
    app: AppHandle,
    state: State<'_, SharedState>,
    opts: BatchOptions,
) -> AppResult<BatchReport> {
    let targets = pick_targets(&state, &opts.targets);
    if targets.is_empty() {
        return Err(AppError::Invalid("没有可操作的文件".into()));
    }
    let spec = opts
        .attrs
        .ok_or_else(|| AppError::Invalid("没有设置要修改的属性".into()))?;
    if spec.is_empty() {
        return Err(AppError::Invalid("没有勾选任何属性".into()));
    }

    state.reset_cancel();
    let cancel = state.cancel.clone();
    let app2 = app.clone();
    let started = std::time::Instant::now();
    let total = targets.len();

    let out = tokio::task::spawn_blocking(move || {
        let mut ok_entries: Vec<JournalEntry> = Vec::new();
        let mut failed: Vec<FailureItem> = Vec::new();
        let mut cancelled = false;
        for (i, f) in targets.iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                cancelled = true;
                break;
            }
            match attrs::write_attrs(std::path::Path::new(&f.path), &spec) {
                Ok(before) => {
                    ok_entries.push(JournalEntry {
                        after: f.path.clone(),
                        before: serde_json::to_string(&before).unwrap_or_default(),
                    });
                }
                Err(e) => failed.push(FailureItem {
                    path: f.path.clone(),
                    reason: e.to_string(),
                }),
            }
            emit_progress(&app2, "attrs", i + 1, total, &f.path);
        }
        (ok_entries, failed, cancelled)
    })
    .await
    .map_err(|e| AppError::Internal(format!("任务异常：{e}")))?;

    let (ok_entries, failed, cancelled) = out;
    let mut j = journal::new_journal(format!("修改文件属性 {} 个文件", ok_entries.len()), "attrs");
    j.entries = ok_entries.clone();
    let journal_id = if !j.entries.is_empty() {
        let id = j.id.clone();
        journal::save(&j)?;
        Some(id)
    } else {
        None
    };

    Ok(BatchReport {
        ok: ok_entries.len(),
        failed,
        cancelled,
        journal_id,
        elapsed_ms: started.elapsed().as_millis() as u64,
    })
}

// ---------------------------------------------------------------------------
// 撤销
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn list_journals() -> Vec<Journal> {
    journal::load_all().into_iter().take(30).collect()
}

#[tauri::command]
pub async fn undo_journal(state: State<'_, SharedState>, id: String) -> AppResult<BatchReport> {
    let all = journal::load_all();
    let j = all
        .into_iter()
        .find(|x| x.id == id)
        .ok_or_else(|| AppError::NotFound(format!("找不到撤销记录 {id}")))?;
    if j.undone {
        return Err(AppError::Invalid("该操作已经撤销过了".into()));
    }

    let started = std::time::Instant::now();
    let mut ok = 0usize;
    let mut failed: Vec<FailureItem> = Vec::new();

    match j.kind.as_str() {
        "rename" => {
            // 逆序回滚
            for e in j.entries.iter().rev() {
                match journal::undo_rename_entry(e) {
                    Ok(_) => ok += 1,
                    Err(reason) => failed.push(FailureItem {
                        path: e.after.clone(),
                        reason,
                    }),
                }
            }
        }
        "times" => {
            for e in j.entries.iter() {
                let spec: TimeSpec = serde_json::from_str(&e.before).unwrap_or_default();
                match times::write_times(std::path::Path::new(&e.after), &spec) {
                    Ok(_) => ok += 1,
                    Err(err) => failed.push(FailureItem {
                        path: e.after.clone(),
                        reason: err.to_string(),
                    }),
                }
            }
        }
        "attrs" => {
            for e in j.entries.iter() {
                let before: attrs::AttrFlags = serde_json::from_str(&e.before).unwrap_or_default();
                let spec = AttrSpec {
                    readonly: Some(before.readonly),
                    hidden: Some(before.hidden),
                    system: Some(before.system),
                    archive: Some(before.archive),
                };
                match attrs::write_attrs(std::path::Path::new(&e.after), &spec) {
                    Ok(_) => ok += 1,
                    Err(err) => failed.push(FailureItem {
                        path: e.after.clone(),
                        reason: err.to_string(),
                    }),
                }
            }
        }
        other => {
            return Err(AppError::Invalid(format!("不支持撤销的操作类型：{other}")));
        }
    }

    if failed.is_empty() {
        journal::mark_undone(&j.id)?;
    }

    // 路径回滚后内存列表已过期，清掉让前端重扫
    if j.kind == "rename" {
        state.files.write().clear();
        *state.plan.write() = None;
    }

    Ok(BatchReport {
        ok,
        failed,
        cancelled: false,
        journal_id: None,
        elapsed_ms: started.elapsed().as_millis() as u64,
    })
}

// ---------------------------------------------------------------------------
// 单项查询（供元数据面板使用）
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDetail {
    pub path: String,
    pub times: TimeSpec,
    pub attrs: attrs::AttrFlags,
}

#[tauri::command]
pub fn get_file_detail(path: String) -> AppResult<FileDetail> {
    let p = std::path::Path::new(&path);
    Ok(FileDetail {
        path: path.clone(),
        times: times::read_times(p)?,
        attrs: attrs::read_attrs(p)?,
    })
}
