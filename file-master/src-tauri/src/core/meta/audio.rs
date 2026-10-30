//! 音频标签（MP3 / FLAC / M4A / OGG / WAV / WMA …）。
//!
//! 交给 lofty 统一处理各容器，这里只做「归一化字段 ↔ 标签项」的映射。
//! 两个坑：
//! 1. MP3 没有标签时 `primary_tag_type()` 会返回 Id3v1，而 Id3v1 存不了中文，
//!    必须强制升级成 Id3v2。
//! 2. 用 `TagExt::save_to_path` 而不是 `TaggedFile::save`——后者依赖读取时
//!    的只读文件句柄，写会失败。

use super::{clean, MetaFields};
use crate::error::{AppError, AppResult};
use lofty::config::WriteOptions;
use lofty::file::TaggedFileExt;
use lofty::tag::items::Timestamp;
use lofty::tag::{Accessor, ItemKey, Tag, TagExt, TagType};
use std::collections::BTreeMap;
use std::path::Path;

fn empty_fields() -> MetaFields {
    MetaFields {
        title: Some(String::new()),
        artist: Some(String::new()),
        album: Some(String::new()),
        genre: Some(String::new()),
        year: Some(0),
        comment: Some(String::new()),
        ..Default::default()
    }
}

/// 挑一个能存中文的标签类型
fn writable_tag_type(tagged: &lofty::file::TaggedFile) -> TagType {
    let tt = tagged.primary_tag_type();
    match tt {
        TagType::Id3v1 => TagType::Id3v2,
        other => other,
    }
}

pub fn read(path: &Path) -> AppResult<MetaFields> {
    let tagged = lofty::read_from_path(path)
        .map_err(|e| AppError::Invalid(format!("音频标签解析失败：{e}")))?;

    let mut f = empty_fields();
    let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) else {
        return Ok(f);
    };

    f.title = Some(tag.title().map(|s| s.to_string()).unwrap_or_default());
    f.artist = Some(tag.artist().map(|s| s.to_string()).unwrap_or_default());
    f.album = Some(tag.album().map(|s| s.to_string()).unwrap_or_default());
    f.genre = Some(tag.genre().map(|s| s.to_string()).unwrap_or_default());
    f.comment = Some(tag.comment().map(|s| s.to_string()).unwrap_or_default());
    f.year = Some(tag.date().map(|d| d.year as i32).unwrap_or(0));
    Ok(f)
}

pub fn write(path: &Path, patch: &BTreeMap<String, String>) -> AppResult<MetaFields> {
    let before = read(path)?;

    let tagged = lofty::read_from_path(path)
        .map_err(|e| AppError::Invalid(format!("音频标签解析失败：{e}")))?;
    let tag_type = writable_tag_type(&tagged);

    let mut tag: Tag = tagged
        .tag(tag_type)
        .or_else(|| tagged.primary_tag())
        .cloned()
        .unwrap_or_else(|| Tag::new(tag_type));

    // 字符串字段：空值表示删除该标签项，而不是写入空串
    macro_rules! text_field {
        ($key:literal, $set:ident, $remove:ident) => {
            if let Some(raw) = patch.get($key) {
                let v = clean(raw);
                if v.is_empty() {
                    tag.$remove();
                } else {
                    tag.$set(v);
                }
            }
        };
    }

    text_field!("title", set_title, remove_title);
    text_field!("artist", set_artist, remove_artist);
    text_field!("album", set_album, remove_album);
    text_field!("genre", set_genre, remove_genre);
    text_field!("comment", set_comment, remove_comment);

    if let Some(raw) = patch.get("year") {
        let y = clean(raw).parse::<u16>().unwrap_or(0);
        if y == 0 {
            tag.remove_key(ItemKey::Year);
            tag.remove_key(ItemKey::RecordingDate);
        } else {
            // 只写年份，月日留空——批量打标签几乎只关心年份
            tag.set_date(Timestamp {
                year: y,
                month: None,
                day: None,
                hour: None,
                minute: None,
                second: None,
            });
        }
    }

    tag.save_to_path(path, WriteOptions::default())
        .map_err(|e| AppError::Io(format!("写入音频标签失败：{e}")))?;

    Ok(before)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_fields_shape() {
        let f = empty_fields();
        assert!(f.title.is_some());
        assert!(f.year.is_some());
        // 音频没有「软件」「版权」这类字段
        assert!(f.software.is_none());
    }
}
