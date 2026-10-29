//! 文件条目与时间/属性的公共模型。
//! 前后端统一使用毫秒时间戳（i64, UTC epoch millis），避免时区歧义。

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::SystemTime;

pub fn to_ms(t: Option<SystemTime>) -> Option<i64> {
    t.map(|st| {
        let dt: chrono::DateTime<chrono::Utc> = st.into();
        dt.timestamp_millis()
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub id: String,
    /// 完整路径（Windows 反斜杠）
    pub path: String,
    /// 所在目录
    pub dir: String,
    /// 文件名（含扩展名）
    pub name: String,
    /// 主名（不含扩展名）
    pub stem: String,
    /// 扩展名，不含点，小写
    pub ext: String,
    pub size: u64,
    pub is_dir: bool,
    pub created_ms: Option<i64>,
    pub modified_ms: Option<i64>,
    pub accessed_ms: Option<i64>,
    pub readonly: bool,
    pub hidden: bool,
    pub system: bool,
    pub archive: bool,
    /// 该条目的元数据摘要（类型标签，供前端显示彩色角标）
    pub kind: String,
}

impl FileEntry {
    pub fn from_path(path: &Path, meta: &std::fs::Metadata, attrs: Option<u32>) -> Self {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let ext = path
            .extension()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let dir = path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        // Windows 属性位
        const FILE_ATTRIBUTE_READONLY: u32 = 0x1;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
        const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x20;

        let (readonly, hidden, system, archive) = match attrs {
            Some(f) => (
                f & FILE_ATTRIBUTE_READONLY != 0,
                f & FILE_ATTRIBUTE_HIDDEN != 0,
                f & FILE_ATTRIBUTE_SYSTEM != 0,
                f & FILE_ATTRIBUTE_ARCHIVE != 0,
            ),
            None => (meta.permissions().readonly(), false, false, false),
        };

        FileEntry {
            id: uuid::Uuid::new_v4().to_string(),
            path: path.to_string_lossy().to_string(),
            dir,
            name,
            stem,
            ext: ext.clone(),
            size: if meta.is_dir() { 0 } else { meta.len() },
            is_dir: meta.is_dir(),
            created_ms: to_ms(meta.created().ok()),
            modified_ms: to_ms(meta.modified().ok()),
            accessed_ms: to_ms(meta.accessed().ok()),
            readonly,
            hidden,
            system,
            archive,
            kind: kind_of(&ext, meta.is_dir()),
        }
    }
}

/// 文件类型归类，用于前端角标与「按类型筛选」
pub fn kind_of(ext: &str, is_dir: bool) -> String {
    if is_dir {
        return "folder".into();
    }
    match ext {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tif" | "tiff" | "heic" | "avif"
        | "svg" | "ico" | "psd" => "image",
        "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "csv" | "rtf" | "odt" => "office",
        "pdf" => "pdf",
        "mp3" | "flac" | "wav" | "aac" | "m4a" | "ogg" | "wma" | "ape" | "opus" => "audio",
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "ts" | "m4v" => "video",
        "zip" | "rar" | "7z" | "tar" | "gz" | "xz" | "bz2" => "archive",
        "exe" | "msi" | "dll" | "bat" | "cmd" | "ps1" => "exec",
        "txt" | "md" | "log" | "json" | "xml" | "yaml" | "yml" | "ini" | "html" | "css"
        | "js" | "py" | "go" | "rs" | "java" | "c" | "cpp" | "h" => "text",
        _ => "other",
    }
    .into()
}
