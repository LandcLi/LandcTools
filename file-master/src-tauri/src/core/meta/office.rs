//! OOXML（docx / xlsx / pptx）文档属性。
//!
//! 属性存在 zip 包内的 `docProps/core.xml`，是一份很小的 XML。
//! 写入方式是**整包重建**：逐条按原始压缩数据搬运其它条目，只替换 core.xml。
//! 用 `raw_copy_file` 而不是解压再压，既快又保证其它部件逐字节不变
//! （有些 Office 文件带数字签名，重压会破坏签名之外的一致性）。

use super::{clean, MetaFields};
use crate::error::{AppError, AppResult};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::Path;

const CORE: &str = "docProps/core.xml";

/// 支持的字段（同时也是「写入后返回哪些值」的白名单）
const FIELDS: &[&str] = &["title", "artist", "subject", "keywords", "comment"];

fn empty_fields() -> MetaFields {
    MetaFields {
        title: Some(String::new()),
        artist: Some(String::new()),
        subject: Some(String::new()),
        keywords: Some(String::new()),
        comment: Some(String::new()),
        ..Default::default()
    }
}

fn read_entry<R: Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
    name: &str,
) -> AppResult<Option<String>> {
    let mut f = match zip.by_name(name) {
        Ok(f) => f,
        Err(zip::result::ZipError::FileNotFound) => return Ok(None),
        Err(e) => return Err(AppError::Io(format!("读取 {name} 失败：{e}"))),
    };
    let mut buf = String::new();
    f.read_to_string(&mut buf)
        .map_err(|e| AppError::Io(format!("读取 {name} 失败：{e}")))?;
    Ok(Some(buf))
}

pub fn read(path: &Path) -> AppResult<MetaFields> {
    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| AppError::Invalid(format!("这不是有效的 Office 文档（zip 结构损坏）：{e}")))?;

    let Some(xml) = read_entry(&mut zip, CORE)? else {
        // 没有 core.xml 说明这份文档属性区是空的
        return Ok(empty_fields());
    };
    Ok(parse_core(&xml))
}

/// 标签名（去命名空间前缀）→ 字段名
fn field_of(name: &str) -> Option<&'static str> {
    let local = name.rsplit(':').next().unwrap_or(name);
    match local {
        "title" => Some("title"),
        "creator" => Some("artist"),
        "subject" => Some("subject"),
        "keywords" => Some("keywords"),
        "description" => Some("comment"),
        _ => None,
    }
}

fn set_field(f: &mut MetaFields, key: &str, val: String) {
    match key {
        "title" => f.title = Some(val),
        "artist" => f.artist = Some(val),
        "subject" => f.subject = Some(val),
        "keywords" => f.keywords = Some(val),
        "comment" => f.comment = Some(val),
        _ => {}
    }
}

/// 反转义 XML 实体。只处理 5 个预定义实体 + 数字字符引用，够 office 属性用。
fn unescape(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        let semi = tail.find(';').filter(|&n| n <= 10);
        let Some(semi) = semi else {
            out.push_str(tail);
            return out;
        };
        let ent = &tail[1..semi];
        let rep: Option<String> = match ent {
            "amp" => Some("&".into()),
            "lt" => Some("<".into()),
            "gt" => Some(">".into()),
            "quot" => Some("\"".into()),
            "apos" => Some("'".into()),
            _ if ent.starts_with("#x") || ent.starts_with("#X") => {
                u32::from_str_radix(&ent[2..], 16).ok().and_then(char::from_u32).map(|c| c.to_string())
            }
            _ if ent.starts_with('#') => ent[1..].parse::<u32>().ok().and_then(char::from_u32).map(|c| c.to_string()),
            _ => None,
        };
        match rep {
            Some(r) => out.push_str(&r),
            None => out.push_str(&tail[..=semi]),
        }
        rest = &tail[semi + 1..];
    }
    out.push_str(rest);
    out
}

fn parse_core(xml: &str) -> MetaFields {
    use quick_xml::events::Event;

    let mut f = empty_fields();
    let mut reader = quick_xml::Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut cur: Option<&'static str> = None;
    let mut buf = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                cur = field_of(e.name().as_ref());
                buf.clear();
            }
            Ok(Event::Text(t)) => {
                if cur.is_some() {
                    buf.push_str(&unescape(t.into_inner().as_ref()));
                }
            }
            Ok(Event::CData(t)) => {
                if cur.is_some() {
                    buf.push_str(t.into_inner().as_ref());
                }
            }
            // quick-xml 0.42 把 &amp; / &#20013; 单独抛成 GeneralRef，
            // 不再混在 Text 里，必须自己还原。
            Ok(Event::GeneralRef(r)) => {
                if cur.is_some() {
                    let name = r.into_inner();
                    buf.push_str(&unescape(&format!("&{name};")));
                }
            }
            Ok(Event::End(_)) => {
                if let Some(k) = cur.take() {
                    set_field(&mut f, k, buf.trim().to_string());
                    buf.clear();
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    f
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            // XML 1.0 不允许的控制字符直接丢弃
            c if (c as u32) < 0x20 && c != '\t' && c != '\n' && c != '\r' => {}
            c => out.push(c),
        }
    }
    out
}

/// 在已有 core.xml 上打补丁，保留 revision / created / modified / lastModifiedBy
fn patch_core(xml: &str, patch: &BTreeMap<String, String>) -> String {
    let mut out = xml.to_string();
    for (key, val) in patch.iter() {
        if !FIELDS.contains(&key.as_str()) {
            continue;
        }
        let v = clean(val);
        // 兼容带前缀与不带前缀两种写法
        let candidates: &[&str] = match key.as_str() {
            "title" => &["dc:title", "title"],
            "artist" => &["dc:creator", "creator"],
            "subject" => &["dc:subject", "subject"],
            "keywords" => &["cp:keywords", "keywords"],
            "comment" => &["dc:description", "description"],
            _ => &[],
        };
        if candidates.is_empty() {
            continue;
        }

        let mut replaced = false;
        for tag in candidates {
            let open = format!("<{tag}>");
            let close = format!("</{tag}>");
            if let (Some(a), Some(b)) = (out.find(&open), out.find(&close)) {
                if b > a {
                    let new = format!("<{tag}>{}</{tag}>", esc(&v));
                    out.replace_range(a..b + close.len(), &new);
                    replaced = true;
                    break;
                }
            }
            // 自闭合形式 <dc:title/>
            let selfc = format!("<{tag}/>");
            if let Some(a) = out.find(&selfc) {
                let new = format!("<{tag}>{}</{tag}>", esc(&v));
                out.replace_range(a..a + selfc.len(), &new);
                replaced = true;
                break;
            }
        }
        if !replaced {
            // 原来没这个标签，插到根节点结束标签之前
            if let Some(end) = out.rfind("</cp:coreProperties>") {
                let t = candidates[0];
                let new = format!("<{t}>{}</{t}>", esc(&v));
                out.insert_str(end, &new);
            }
        }
    }
    out
}

const EMPTY_CORE: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" "#,
    r#"xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" "#,
    r#"xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"></cp:coreProperties>"#
);

pub fn write(path: &Path, patch: &BTreeMap<String, String>) -> AppResult<MetaFields> {
    let before = read(path)?;

    let old_xml = {
        let file = std::fs::File::open(path)?;
        let mut zip = zip::ZipArchive::new(file)
            .map_err(|e| AppError::Invalid(format!("这不是有效的 Office 文档：{e}")))?;
        read_entry(&mut zip, CORE)?
    };

    let new_xml = match &old_xml {
        Some(x) => patch_core(x, patch),
        None => patch_core(EMPTY_CORE, patch),
    };

    super::atomic_replace(path, |tmp| {
        let src = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(src)
            .map_err(|e| AppError::Invalid(format!("这不是有效的 Office 文档：{e}")))?;
        let out = std::fs::File::create(tmp)?;
        let mut zw = zip::ZipWriter::new(out);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for i in 0..archive.len() {
            let entry = archive
                .by_index_raw(i)
                .map_err(|e| AppError::Io(format!("读取压缩包条目失败：{e}")))?;
            let name = entry.name().unwrap_or_default().to_string();
            if name == CORE {
                continue;
            }
            zw.raw_copy_file(entry)
                .map_err(|e| AppError::Io(format!("复制压缩包条目失败：{e}")))?;
        }

        zw.start_file(CORE, opts)
            .map_err(|e| AppError::Io(format!("写入属性失败：{e}")))?;
        zw.write_all(new_xml.as_bytes())?;
        zw.finish()
            .map_err(|e| AppError::Io(format!("保存文档失败：{e}")))?;
        Ok(())
    })?;

    Ok(before)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_patch_roundtrip() {
        let xml = concat!(
            r#"<?xml version="1.0"?><cp:coreProperties xmlns:cp="c" xmlns:dc="d">"#,
            r#"<dc:title>旧标题</dc:title><dc:creator>老王</dc:creator>"#,
            r#"<cp:revision>7</cp:revision></cp:coreProperties>"#
        );
        let f = parse_core(xml);
        assert_eq!(f.title.as_deref(), Some("旧标题"));
        assert_eq!(f.artist.as_deref(), Some("老王"));

        let mut patch = BTreeMap::new();
        patch.insert("title".to_string(), "新标题 & <测试>".to_string());
        patch.insert("subject".to_string(), "主题A".to_string());
        let out = patch_core(xml, &patch);
        assert!(out.contains("<dc:title>新标题 &amp; &lt;测试&gt;</dc:title>"));
        assert!(out.contains("<dc:subject>主题A</dc:subject>"));
        // 原有字段不能丢
        assert!(out.contains("<cp:revision>7</cp:revision>"));
        assert!(out.contains("<dc:creator>老王</dc:creator>"));
    }

    #[test]
    fn patch_clears_when_empty() {
        let xml = r#"<cp:coreProperties xmlns:dc="d"><dc:title>X</dc:title></cp:coreProperties>"#;
        let mut patch = BTreeMap::new();
        patch.insert("title".to_string(), String::new());
        let out = patch_core(xml, &patch);
        assert!(out.contains("<dc:title></dc:title>"));
    }

    #[test]
    fn patch_self_closing() {
        let xml = r#"<cp:coreProperties xmlns:dc="d"><dc:title/></cp:coreProperties>"#;
        let mut patch = BTreeMap::new();
        patch.insert("title".to_string(), "A".to_string());
        assert!(patch_core(xml, &patch).contains("<dc:title>A</dc:title>"));
    }

    #[test]
    fn entities_roundtrip() {
        let xml = r#"<cp:coreProperties xmlns:dc="d"><dc:title>A &amp; B &#20013;</dc:title></cp:coreProperties>"#;
        let f = parse_core(xml);
        assert_eq!(f.title.as_deref(), Some("A & B 中"));
    }
}
