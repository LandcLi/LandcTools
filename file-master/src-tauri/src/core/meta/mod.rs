//! 标签 / 元数据编辑。
//!
//! 四类格式各自解析容器，但对外只暴露一套归一化字段，
//! 前端按 `format` 决定显示哪些输入框。
//!
//! 写入语义是**部分更新**：`patch` 里出现的键才改，没出现的保持原值。
//! 值为空串表示清空该字段（不是忽略）。

pub mod audio;
pub mod image;
pub mod office;
pub mod pdf;

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// 字段模型
// ---------------------------------------------------------------------------

/// 归一化字段。`None` = 该格式不提供这个字段（前端不显示输入框）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaFields {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    pub comment: Option<String>,
    pub genre: Option<String>,
    pub year: Option<i32>,
    pub software: Option<String>,
    pub copyright: Option<String>,
}

impl MetaFields {
    /// 该格式支持的可写字段名（顺序即界面顺序）
    pub fn supported(format: MetaFormat) -> &'static [&'static str] {
        match format {
            MetaFormat::Office => &["title", "artist", "subject", "keywords", "comment"],
            MetaFormat::Pdf => &["title", "artist", "subject", "keywords", "software"],
            MetaFormat::Audio => &["title", "artist", "album", "genre", "year", "comment"],
            MetaFormat::Image => &["comment", "artist", "copyright", "software"],
            MetaFormat::None => &[],
        }
    }

    /// 把 patch 里属于本格式、且值非默认的项取出来
    pub fn pick(patch: &BTreeMap<String, String>) -> Vec<(String, String)> {
        patch
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// 转成 patch，用于撤销时把旧值写回去。
    /// `None` 表示该格式本来就没有这个字段，跳过。
    pub fn to_patch(&self) -> BTreeMap<String, String> {
        let mut p = BTreeMap::new();
        let text: [(&str, &Option<String>); 7] = [
            ("title", &self.title),
            ("artist", &self.artist),
            ("album", &self.album),
            ("subject", &self.subject),
            ("keywords", &self.keywords),
            ("comment", &self.comment),
            ("genre", &self.genre),
        ];
        for (k, v) in text {
            if let Some(v) = v {
                p.insert(k.to_string(), v.clone());
            }
        }
        if let Some(v) = &self.software {
            p.insert("software".into(), v.clone());
        }
        if let Some(v) = &self.copyright {
            p.insert("copyright".into(), v.clone());
        }
        if let Some(y) = self.year {
            let s = if y > 0 { y.to_string() } else { String::new() };
            p.insert("year".into(), s);
        }
        p
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetaFormat {
    Office,
    Pdf,
    Audio,
    Image,
    None,
}

impl MetaFormat {
    pub fn of_ext(ext: &str) -> MetaFormat {
        match ext {
            "docx" | "xlsx" | "pptx" | "docm" | "xlsm" | "pptm" => MetaFormat::Office,
            "pdf" => MetaFormat::Pdf,
            "mp3" | "flac" | "m4a" | "mp4" | "ogg" | "oga" | "opus" | "wav" | "wma" | "aac"
            | "aiff" | "ape" | "wv" => MetaFormat::Audio,
            "jpg" | "jpeg" | "png" | "tif" | "tiff" | "webp" | "heic" | "heif" | "avif" | "jxl" => {
                MetaFormat::Image
            }
            _ => MetaFormat::None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            MetaFormat::Office => "Office 文档",
            MetaFormat::Pdf => "PDF 文档",
            MetaFormat::Audio => "音频文件",
            MetaFormat::Image => "图片",
            MetaFormat::None => "不支持",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaInfo {
    pub path: String,
    pub format: MetaFormat,
    pub format_label: String,
    /// 能否写入；false 时 `reason` 说明原因（如加密 PDF）
    pub writable: bool,
    pub reason: Option<String>,
    pub fields: MetaFields,
}

// ---------------------------------------------------------------------------
// 对外入口
// ---------------------------------------------------------------------------

fn ext_of(path: &Path) -> String {
    path.extension()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

pub fn read(path: &Path) -> MetaInfo {
    let ext = ext_of(path);
    let format = MetaFormat::of_ext(&ext);
    let mut info = MetaInfo {
        path: path.to_string_lossy().to_string(),
        format,
        format_label: format.label().to_string(),
        writable: format != MetaFormat::None,
        reason: None,
        fields: MetaFields::default(),
    };

    let res: AppResult<MetaFields> = match format {
        MetaFormat::Office => office::read(path),
        MetaFormat::Pdf => pdf::read(path),
        MetaFormat::Audio => audio::read(path),
        MetaFormat::Image => image::read(path),
        MetaFormat::None => Err(AppError::Unsupported(format!(
            "暂不支持 .{ext} 格式的标签编辑"
        ))),
    };

    match res {
        Ok(f) => info.fields = f,
        Err(e) => {
            info.writable = false;
            info.reason = Some(e.to_string());
        }
    }
    info
}

/// 写入。返回写入前的字段值，供撤销用。
///
/// `keep_mtime` 为真时，写完把修改时间还原——批量打标签通常是整理动作，
/// 不该让文件在资源管理器里"全都变成今天"。
pub fn write(
    path: &Path,
    patch: &BTreeMap<String, String>,
    keep_mtime: bool,
) -> AppResult<MetaFields> {
    if patch.is_empty() {
        return Err(AppError::Invalid("没有要写入的字段".into()));
    }
    let ext = ext_of(path);
    let format = MetaFormat::of_ext(&ext);
    if format == MetaFormat::None {
        return Err(AppError::Unsupported(format!(
            "暂不支持 .{ext} 格式的标签写入"
        )));
    }
    if !path.is_file() {
        return Err(AppError::NotFound(format!("文件不存在：{}", path.display())));
    }

    // 一个字段都不匹配时直接报错。否则会「显示成功但什么都没改」，
    // 批量打标签时最难排查的就是这种静默失败。
    let supported = MetaFields::supported(format);
    let hit = patch
        .keys()
        .filter(|k| supported.contains(&k.as_str()))
        .count();
    if hit == 0 {
        return Err(AppError::Unsupported(format!(
            "{} 不支持你勾选的标签字段",
            format.label()
        )));
    }

    let before_mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok();

    let before = match format {
        MetaFormat::Office => office::write(path, patch),
        MetaFormat::Pdf => pdf::write(path, patch),
        MetaFormat::Audio => audio::write(path, patch),
        MetaFormat::Image => image::write(path, patch),
        MetaFormat::None => unreachable!(),
    }?;

    if keep_mtime {
        if let Some(t) = before_mtime {
            let ft = filetime::FileTime::from_system_time(t);
            let _ = filetime::set_file_mtime(path, ft);
        }
    }

    Ok(before)
}

// ---------------------------------------------------------------------------
// 公共工具
// ---------------------------------------------------------------------------

/// 原子替换：先把新内容写到同目录临时文件，再 rename 覆盖。
///
/// 这样即使写入中途失败，原文件也完好。`fs::rename` 在 Windows 上是
/// MoveFileEx + REPLACE_EXISTING，可以直接覆盖（前提是没有别的进程锁着）。
pub(crate) fn atomic_replace<F>(path: &Path, build: F) -> AppResult<()>
where
    F: FnOnce(&Path) -> AppResult<()>,
{
    let tmp = tmp_path_for(path);
    let _ = std::fs::remove_file(&tmp);

    if let Err(e) = build(&tmp) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }

    // 目标只读时 rename 会失败，先去掉只读位
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.permissions().readonly() {
            let mut perm = meta.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            perm.set_readonly(false);
            let _ = std::fs::set_permissions(path, perm);
        }
    }

    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        AppError::Io(format!("替换原文件失败：{e}"))
    })
}

/// 临时文件名。**必须保留原扩展名**：little_exif 按扩展名认格式，
/// 拿 `.tmp` 结尾的路径会直接报 "Unsupported file type"。
fn tmp_path_for(path: &Path) -> PathBuf {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "tmp".into());
    let ext = path
        .extension()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    // 带上进程号，避免多实例同时处理同一文件时互相踩
    let tail = if ext.is_empty() {
        format!("lfm{}.tmp", std::process::id())
    } else {
        format!("lfm{}.tmp.{ext}", std::process::id())
    };
    dir.join(format!("{stem}.{tail}"))
}

/// 清理用户输入：去掉首尾空白与不可见的零宽字符。
pub(crate) fn clean(s: &str) -> String {
    s.trim()
        .trim_matches(|c: char| c.is_control() || c == '\u{feff}' || c == '\u{200b}')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_by_ext() {
        assert_eq!(MetaFormat::of_ext("docx"), MetaFormat::Office);
        assert_eq!(MetaFormat::of_ext("PDF"), MetaFormat::None); // 调用方已小写
        assert_eq!(MetaFormat::of_ext("pdf"), MetaFormat::Pdf);
        assert_eq!(MetaFormat::of_ext("flac"), MetaFormat::Audio);
        assert_eq!(MetaFormat::of_ext("jpeg"), MetaFormat::Image);
        assert_eq!(MetaFormat::of_ext("exe"), MetaFormat::None);
    }

    #[test]
    fn supported_fields_per_format() {
        assert!(MetaFields::supported(MetaFormat::Audio).contains(&"album"));
        assert!(!MetaFields::supported(MetaFormat::Image).contains(&"album"));
        assert!(MetaFields::supported(MetaFormat::None).is_empty());
    }
}
