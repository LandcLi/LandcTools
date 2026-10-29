//! Windows 文件属性（只读 / 隐藏 / 系统 / 归档）读写。
//! `Option<bool>` 语义：`None` = 不改动，`Some(x)` = 设为 x。

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttrSpec {
    pub readonly: Option<bool>,
    pub hidden: Option<bool>,
    pub system: Option<bool>,
    pub archive: Option<bool>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttrFlags {
    pub readonly: bool,
    pub hidden: bool,
    pub system: bool,
    pub archive: bool,
}

impl AttrSpec {
    pub fn is_empty(&self) -> bool {
        self.readonly.is_none()
            && self.hidden.is_none()
            && self.system.is_none()
            && self.archive.is_none()
    }
}

pub const ATTR_READONLY: u32 = 0x1;
pub const ATTR_HIDDEN: u32 = 0x2;
pub const ATTR_SYSTEM: u32 = 0x4;
pub const ATTR_ARCHIVE: u32 = 0x20;
const INVALID_ATTRIBUTES: u32 = u32::MAX;

#[cfg(windows)]
mod imp {
    use super::*;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileAttributesW, SetFileAttributesW,
    };

    fn wide(p: &Path) -> Vec<u16> {
        OsStr::new(p).encode_wide().chain(std::iter::once(0)).collect()
    }

    pub fn read_flags(path: &Path) -> AppResult<u32> {
        let w = wide(path);
        let f = unsafe { GetFileAttributesW(w.as_ptr()) };
        if f == INVALID_ATTRIBUTES {
            return Err(AppError::Io(format!(
                "读取属性失败 {}: {}",
                path.display(),
                std::io::Error::last_os_error()
            )));
        }
        Ok(f)
    }

    pub fn write_flags(path: &Path, flags: u32) -> AppResult<()> {
        let w = wide(path);
        let ok = unsafe { SetFileAttributesW(w.as_ptr(), flags) };
        if ok == 0 {
            return Err(AppError::Io(format!(
                "写入属性失败 {}: {}",
                path.display(),
                std::io::Error::last_os_error()
            )));
        }
        Ok(())
    }
}

#[cfg(not(windows))]
mod imp {
    use super::*;

    pub fn read_flags(path: &Path) -> AppResult<u32> {
        let meta = std::fs::metadata(path)?;
        Ok(if meta.permissions().readonly() {
            ATTR_READONLY
        } else {
            ATTR_ARCHIVE
        })
    }

    pub fn write_flags(path: &Path, flags: u32) -> AppResult<()> {
        let meta = std::fs::metadata(path)?;
        let mut perms = meta.permissions();
        perms.set_readonly(flags & ATTR_READONLY != 0);
        std::fs::set_permissions(path, perms)?;
        Ok(())
    }
}

pub fn flags_to_struct(f: u32) -> AttrFlags {
    AttrFlags {
        readonly: f & ATTR_READONLY != 0,
        hidden: f & ATTR_HIDDEN != 0,
        system: f & ATTR_SYSTEM != 0,
        archive: f & ATTR_ARCHIVE != 0,
    }
}

pub fn read_attrs(path: &Path) -> AppResult<AttrFlags> {
    Ok(flags_to_struct(imp::read_flags(path)?))
}

/// 按 spec 合并属性位并写回，返回改动前的属性
pub fn write_attrs(path: &Path, spec: &AttrSpec) -> AppResult<AttrFlags> {
    let before_flags = imp::read_flags(path)?;
    let before = flags_to_struct(before_flags);
    if spec.is_empty() {
        return Ok(before);
    }

    let mut flags = before_flags;
    let mut apply = |mask: u32, want: Option<bool>| {
        if let Some(v) = want {
            if v {
                flags |= mask;
            } else {
                flags &= !mask;
            }
        }
    };
    apply(ATTR_READONLY, spec.readonly);
    apply(ATTR_HIDDEN, spec.hidden);
    apply(ATTR_SYSTEM, spec.system);
    apply(ATTR_ARCHIVE, spec.archive);

    if flags != before_flags {
        imp::write_flags(path, flags)?;
    }
    Ok(before)
}
