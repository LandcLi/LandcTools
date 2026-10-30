//! PDF 文档属性（Info 字典）。
//!
//! 只碰 `/Info` 里的 5 个文本键，不动页面内容与 XMP。
//! 中文必须写成「UTF-16BE + BOM」的 PDF 字面量字符串，否则阅读器会按
//! PDFDocEncoding 显示成乱码——这是 PDF 属性能不能正确显示中文的唯一关键。

use super::{clean, MetaFields};
use crate::error::{AppError, AppResult};
use lopdf::{Dictionary, Document, Object, StringFormat};
use std::collections::BTreeMap;
use std::path::Path;

/// 超过这个大小就不读进内存了（lopdf 是全量加载）
const MAX_BYTES: u64 = 512 * 1024 * 1024;

/// 归一化字段名 → PDF Info 键
const KEYS: &[(&str, &str)] = &[
    ("title", "Title"),
    ("artist", "Author"),
    ("subject", "Subject"),
    ("keywords", "Keywords"),
    ("software", "Creator"),
];

fn empty_fields() -> MetaFields {
    MetaFields {
        title: Some(String::new()),
        artist: Some(String::new()),
        subject: Some(String::new()),
        keywords: Some(String::new()),
        software: Some(String::new()),
        ..Default::default()
    }
}

fn encode(s: &str) -> Object {
    if s.is_ascii() {
        return Object::String(s.as_bytes().to_vec(), StringFormat::Literal);
    }
    let mut bytes = Vec::with_capacity(2 + s.len() * 2);
    bytes.push(0xFE);
    bytes.push(0xFF);
    for u in s.encode_utf16() {
        bytes.extend_from_slice(&u.to_be_bytes());
    }
    Object::String(bytes, StringFormat::Literal)
}

fn decode(o: &Object) -> Option<String> {
    let bytes = match o {
        Object::String(b, _) => b,
        Object::Name(n) => return Some(String::from_utf8_lossy(n).to_string()),
        _ => return None,
    };
    Some(decode_bytes(bytes))
}

fn decode_bytes(b: &[u8]) -> String {
    if b.len() >= 2 && b[0] == 0xFE && b[1] == 0xFF {
        let units: Vec<u16> = b[2..]
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16_lossy(&units);
    }
    if b.len() >= 2 && b[0] == 0xFF && b[1] == 0xFE {
        let units: Vec<u16> = b[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16_lossy(&units);
    }
    // 大多数纯 ASCII 属性走这里；非 ASCII 老文件尽力而为
    String::from_utf8_lossy(b).to_string()
}

fn info_dict(doc: &Document) -> Option<Dictionary> {
    let id = doc.trailer.get(b"Info").ok()?.as_reference().ok()?;
    doc.get_object(id).ok()?.as_dict().ok().cloned()
}

fn guard(path: &Path) -> AppResult<()> {
    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    if size > MAX_BYTES {
        return Err(AppError::Unsupported(
            "PDF 超过 512MB，为避免占用过多内存已跳过".into(),
        ));
    }
    Ok(())
}

pub fn read(path: &Path) -> AppResult<MetaFields> {
    guard(path)?;
    let doc = Document::load(path)
        .map_err(|e| AppError::Invalid(format!("PDF 解析失败（可能已损坏或加密）：{e}")))?;
    if doc.is_encrypted() {
        return Err(AppError::Unsupported("PDF 已加密，属性无法编辑".into()));
    }

    let mut f = empty_fields();
    let Some(dict) = info_dict(&doc) else {
        return Ok(f);
    };

    for (field, key) in KEYS {
        let v = dict.get(key.as_bytes()).ok().and_then(decode).unwrap_or_default();
        match *field {
            "title" => f.title = Some(v),
            "artist" => f.artist = Some(v),
            "subject" => f.subject = Some(v),
            "keywords" => f.keywords = Some(v),
            "software" => f.software = Some(v),
            _ => {}
        }
    }
    Ok(f)
}

pub fn write(path: &Path, patch: &BTreeMap<String, String>) -> AppResult<MetaFields> {
    guard(path)?;
    let before = read(path)?;

    let mut doc = Document::load(path)
        .map_err(|e| AppError::Invalid(format!("PDF 解析失败（可能已损坏或加密）：{e}")))?;
    if doc.is_encrypted() {
        return Err(AppError::Unsupported("PDF 已加密，属性无法编辑".into()));
    }

    let info_id = doc
        .trailer
        .get(b"Info")
        .ok()
        .and_then(|o| o.as_reference().ok());

    let mut dict = info_dict(&doc).unwrap_or_else(Dictionary::new);

    for (field, key) in KEYS {
        if let Some(raw) = patch.get(*field) {
            let v = clean(raw);
            dict.set(key.as_bytes(), encode(&v));
        }
    }

    match info_id {
        Some(id) => {
            doc.objects.insert(id, Object::Dictionary(dict));
        }
        None => {
            let id = doc.add_object(Object::Dictionary(dict));
            doc.trailer.set("Info", Object::Reference(id));
        }
    }

    super::atomic_replace(path, |tmp| {
        let mut d = doc;
        d.save(tmp)
            .map_err(|e| AppError::Io(format!("PDF 保存失败：{e}")))?;
        Ok(())
    })?;

    Ok(before)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chinese_roundtrip() {
        let o = encode("中文标题");
        assert!(matches!(&o, Object::String(b, _) if b.starts_with(&[0xFE, 0xFF])));
        assert_eq!(decode(&o).as_deref(), Some("中文标题"));
    }

    #[test]
    fn ascii_roundtrip() {
        let o = encode("Hello PDF");
        assert_eq!(decode(&o).as_deref(), Some("Hello PDF"));
        // 纯 ASCII 不该带 BOM，避免老阅读器显示成怪字符
        assert!(matches!(&o, Object::String(b, _) if b.first() != Some(&0xFE)));
    }
}
