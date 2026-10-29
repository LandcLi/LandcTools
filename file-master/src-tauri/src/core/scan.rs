//! 目录扫描：把用户选中的目录展开成可操作的条目列表。
//!
//! 设计要点：
//! - 只扫一遍、不递归读文件内容，元数据全部来自 `metadata()`，避免二次 stat
//! - Windows 属性位直接用 `std::os::windows::fs::MetadataExt::file_attributes()`，
//!   不再额外调 `GetFileAttributesW`，10 万文件也能几秒扫完
//! - 有硬上限（默认 20 万），防止用户误选 C 盘导致内存爆炸

use crate::error::AppResult;
use crate::model::FileEntry;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanOptions {
    pub dirs: Vec<String>,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub max_depth: Option<usize>,
    #[serde(default)]
    pub include_hidden: bool,
    #[serde(default)]
    pub include_dirs: bool,
    /// 扩展名白名单（小写、不带点），空表示全部
    #[serde(default)]
    pub exts: Vec<String>,
    #[serde(default)]
    pub name_filter: String,
    #[serde(default)]
    pub use_regex: bool,
    #[serde(default)]
    pub min_size: Option<u64>,
    #[serde(default)]
    pub max_size: Option<u64>,
    #[serde(default)]
    pub limit: Option<usize>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        ScanOptions {
            dirs: vec![],
            recursive: true,
            max_depth: None,
            include_hidden: false,
            include_dirs: false,
            exts: vec![],
            name_filter: String::new(),
            use_regex: false,
            min_size: None,
            max_size: None,
            limit: Some(200_000),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanOutcome {
    pub entries: Vec<FileEntry>,
    pub scanned: usize,
    pub matched: usize,
    pub truncated: bool,
    pub errors: Vec<String>,
    /// 实际扫描的根目录（规范化后）。前端用它显示「已添加的文件夹」并计算相对路径。
    pub roots: Vec<String>,
}

#[cfg(windows)]
fn is_hidden(meta: &std::fs::Metadata, name: &str) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0 || name.starts_with('.')
}

#[cfg(not(windows))]
fn is_hidden(_meta: &std::fs::Metadata, name: &str) -> bool {
    name.starts_with('.')
}

pub fn scan<F, C>(opts: &ScanOptions, mut on_progress: F, is_cancelled: C) -> AppResult<ScanOutcome>
where
    F: FnMut(usize, &str),
    C: Fn() -> bool,
{
    let limit = opts.limit.unwrap_or(200_000);

    // 扩展名白名单预归一化
    let ext_set: Vec<String> = opts
        .exts
        .iter()
        .map(|e| e.trim().trim_start_matches('.').to_lowercase())
        .filter(|e| !e.is_empty())
        .collect();

    // 名称过滤器
    let name_re: Option<Regex> = if opts.name_filter.is_empty() {
        None
    } else if opts.use_regex {
        Some(Regex::new(&opts.name_filter).map_err(|e| {
            crate::error::AppError::Invalid(format!("名称正则有误：{e}"))
        })?)
    } else {
        let escaped = regex::escape(&opts.name_filter);
        Some(Regex::new(&format!("(?i){escaped}")).unwrap())
    };

    let mut entries: Vec<FileEntry> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut scanned = 0usize;
    let mut truncated = false;
    // 实际生效的根目录（去掉末尾分隔符，保持用户原始大小写）
    let mut roots: Vec<String> = Vec::new();

    for dir in &opts.dirs {
        let root = PathBuf::from(dir);
        if !root.exists() {
            errors.push(format!("目录不存在：{dir}"));
            continue;
        }

        let normalized = dir
            .trim_end_matches(['\\', '/'])
            .to_string();
        let normalized = if normalized.is_empty() {
            dir.clone()
        } else {
            normalized
        };
        if !roots.iter().any(|r| r.eq_ignore_ascii_case(&normalized)) {
            roots.push(normalized);
        }

        let depth = if opts.recursive {
            opts.max_depth.unwrap_or(usize::MAX)
        } else {
            1
        };

        let walker = WalkDir::new(&root)
            .min_depth(1)
            .max_depth(depth)
            .follow_links(false)
            .into_iter();

        for item in walker {
            if is_cancelled() {
                break;
            }
            let entry = match item {
                Ok(e) => e,
                Err(e) => {
                    if errors.len() < 100 {
                        errors.push(format!("{e}"));
                    }
                    continue;
                }
            };

            scanned += 1;
            if scanned % 200 == 0 {
                on_progress(scanned, &entry.path().to_string_lossy());
            }

            let path = entry.path();
            let is_dir = entry.file_type().is_dir();
            if is_dir && !opts.include_dirs {
                continue;
            }

            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    if errors.len() < 100 {
                        errors.push(format!("{}: {e}", path.display()));
                    }
                    continue;
                }
            };

            let file_name = entry.file_name().to_string_lossy().to_string();
            if !opts.include_hidden && is_hidden(&meta, &file_name) {
                continue;
            }

            if !ext_set.is_empty() && !is_dir {
                let ext = path
                    .extension()
                    .map(|s| s.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                if !ext_set.iter().any(|e| *e == ext) {
                    continue;
                }
            }

            if let Some(re) = &name_re {
                if !re.is_match(&file_name) {
                    continue;
                }
            }

            if !is_dir {
                let len = meta.len();
                if let Some(min) = opts.min_size {
                    if len < min {
                        continue;
                    }
                }
                if let Some(max) = opts.max_size {
                    if len > max {
                        continue;
                    }
                }
            }

            if entries.len() >= limit {
                truncated = true;
                break;
            }

            #[cfg(windows)]
            let attrs = {
                use std::os::windows::fs::MetadataExt;
                Some(meta.file_attributes())
            };
            #[cfg(not(windows))]
            let attrs = None;

            entries.push(FileEntry::from_path(path, &meta, attrs));
        }

        if truncated || is_cancelled() {
            break;
        }
    }

    // 稳定排序：目录优先，其次按名称自然序（数字按数值比较，1.jpg 在 10.jpg 前面）
    entries.sort_by(|a, b| natural_cmp(&a.path, &b.path));

    let matched = entries.len();
    on_progress(scanned, "");

    Ok(ScanOutcome {
        entries,
        scanned,
        matched,
        truncated,
        errors,
        roots,
    })
}

/// 自然序比较：把连续数字当成数字比，避免 1,10,100,2 这种排序
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    let (mut i, mut j) = (0usize, 0usize);
    while i < ab.len() && j < bb.len() {
        if ab[i].is_ascii_digit() && bb[j].is_ascii_digit() {
            let si = i;
            while i < ab.len() && ab[i].is_ascii_digit() {
                i += 1;
            }
            let sj = j;
            while j < bb.len() && bb[j].is_ascii_digit() {
                j += 1;
            }
            let na = std::str::from_utf8(&ab[si..i]).unwrap_or("0");
            let nb = std::str::from_utf8(&bb[sj..j]).unwrap_or("0");
            // 去前导零后比长度，再比字典序
            let ta = na.trim_start_matches('0');
            let tb = nb.trim_start_matches('0');
            let ta = if ta.is_empty() { "0" } else { ta };
            let tb = if tb.is_empty() { "0" } else { tb };
            let ord = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb));
            if ord != Ordering::Equal {
                return ord;
            }
        } else {
            let ca = ab[i].to_ascii_lowercase();
            let cb = bb[j].to_ascii_lowercase();
            if ca != cb {
                return ca.cmp(&cb);
            }
            i += 1;
            j += 1;
        }
    }
    (ab.len() - i).cmp(&(bb.len() - j))
}

/// 判断路径是否为目录
pub fn is_dir(p: &Path) -> bool {
    p.is_dir()
}
