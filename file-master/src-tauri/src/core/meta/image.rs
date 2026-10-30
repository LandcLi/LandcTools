//! 图片 EXIF（JPEG / PNG / TIFF / WebP / HEIC / AVIF / JXL）。
//!
//! little_exif 是「原地读改写」，没法像 zip/PDF 那样对着临时路径凭空生成，
//! 所以先整份复制到临时文件、在副本上改、再原子替换回原路径。
//! 这样即使写入过程中断电或抛错，原图也不会变成半张。

use super::{clean, MetaFields};
use crate::error::{AppError, AppResult};
use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use std::collections::BTreeMap;
use std::path::Path;

fn empty_fields() -> MetaFields {
    MetaFields {
        comment: Some(String::new()),
        artist: Some(String::new()),
        copyright: Some(String::new()),
        software: Some(String::new()),
        ..Default::default()
    }
}

/// little_exif 的 `new_from_path` 在文件里**没有** EXIF 时报错（PNG 用的是
/// `ErrorKind::Other` + "No metadata found!"），而不是返回空元数据。
/// 这种情况不是错误，只是还没打标签，所以按空元数据处理。
fn load(path: &Path) -> AppResult<Metadata> {
    match Metadata::new_from_path(path) {
        Ok(m) => Ok(m),
        Err(e) if e.to_string().contains("No metadata found") => Ok(Metadata::new()),
        Err(e) => Err(AppError::Invalid(format!("EXIF 解析失败：{e}"))),
    }
}

pub fn read(path: &Path) -> AppResult<MetaFields> {
    let meta = load(path)?;

    let mut f = empty_fields();
    for tag in &meta {
        match tag {
            ExifTag::ImageDescription(s) => f.comment = Some(s.clone()),
            ExifTag::Artist(s) => f.artist = Some(s.clone()),
            ExifTag::Copyright(s) => f.copyright = Some(s.clone()),
            ExifTag::Software(s) => f.software = Some(s.clone()),
            _ => {}
        }
    }
    Ok(f)
}

pub fn write(path: &Path, patch: &BTreeMap<String, String>) -> AppResult<MetaFields> {
    let before = read(path)?;

    super::atomic_replace(path, |tmp| {
        std::fs::copy(path, tmp)?;
        // fs::copy 会把只读属性一起带过来，清掉才能写 EXIF
        if let Ok(m) = std::fs::metadata(tmp) {
            if m.permissions().readonly() {
                let mut perm = m.permissions();
                #[allow(clippy::permissions_set_readonly_false)]
                perm.set_readonly(false);
                let _ = std::fs::set_permissions(tmp, perm);
            }
        }
        let mut meta = load(tmp)?;

        for (key, val) in patch.iter() {
            let v = clean(val);
            // 空值 = 删掉这条 EXIF，避免留下空描述干扰看图软件
            match key.as_str() {
                "comment" => {
                    if v.is_empty() {
                        meta.remove_tag(ExifTag::ImageDescription(String::new()));
                    } else {
                        meta.set_tag(ExifTag::ImageDescription(v));
                    }
                }
                "artist" => {
                    if v.is_empty() {
                        meta.remove_tag(ExifTag::Artist(String::new()));
                    } else {
                        meta.set_tag(ExifTag::Artist(v));
                    }
                }
                "copyright" => {
                    if v.is_empty() {
                        meta.remove_tag(ExifTag::Copyright(String::new()));
                    } else {
                        meta.set_tag(ExifTag::Copyright(v));
                    }
                }
                "software" => {
                    if v.is_empty() {
                        meta.remove_tag(ExifTag::Software(String::new()));
                    } else {
                        meta.set_tag(ExifTag::Software(v));
                    }
                }
                _ => {}
            }
        }

        meta.write_to_file(tmp)
            .map_err(|e| AppError::Io(format!("写入 EXIF 失败：{e}")))?;
        Ok(())
    })?;

    Ok(before)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_image_fields() {
        let f = empty_fields();
        assert!(f.comment.is_some());
        // 图片没有「专辑」「年份」这类概念
        assert!(f.album.is_none());
        assert!(f.year.is_none());
    }
}
