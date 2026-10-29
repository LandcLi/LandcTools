//! 文件时间属性读写。
//!
//! 关键点：Rust `std` 只能设置访问/修改时间（`filetime` crate 也是），
//! **创建时间必须走 Win32 `SetFileTime`**，否则「批量修改文件创建时间」这个卖点做不出来。
//! 所以这里用 `windows-sys` 裸签名的 `CreateFileW` + `GetFileTime` + `SetFileTime`。

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeSpec {
    /// `Some(ms)` 表示设为该时间，`None` 表示保持原值
    pub created_ms: Option<i64>,
    pub modified_ms: Option<i64>,
    pub accessed_ms: Option<i64>,
}

#[cfg(windows)]
mod imp {
    use super::*;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, HANDLE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, GetFileTime, SetFileTime, FILE_SHARE_DELETE, FILE_SHARE_READ,
        FILE_SHARE_WRITE, OPEN_EXISTING,
    };

    const FILE_READ_ATTRIBUTES: u32 = 0x0080;
    const FILE_WRITE_ATTRIBUTES: u32 = 0x0100;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    /// 1601-01-01 到 1970-01-01 的 100ns 间隔
    const EPOCH_DIFF: i128 = 116_444_736_000_000_000;

    fn wide(p: &Path) -> Vec<u16> {
        OsStr::new(p).encode_wide().chain(std::iter::once(0)).collect()
    }

    struct Handle(HANDLE);
    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }

    /// 以「只改属性」的权限打开句柄，只读文件也能打开
    fn open_attr(p: &Path) -> AppResult<Handle> {
        let w = wide(p);
        let h = unsafe {
            CreateFileW(
                w.as_ptr(),
                FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS,
                0,
            )
        };
        if h == INVALID_HANDLE_VALUE {
            let err = std::io::Error::last_os_error();
            return Err(match err.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    AppError::Denied(format!("{}: {err}", p.display()))
                }
                _ => AppError::Io(format!("{}: {err}", p.display())),
            });
        }
        Ok(Handle(h))
    }

    fn ft_to_ms(ft: FILETIME) -> i64 {
        let t = ((ft.dwHighDateTime as u64) << 32) | ft.dwLowDateTime as u64;
        ((t as i128 - EPOCH_DIFF) / 10_000) as i64
    }

    fn ms_to_ft(ms: i64) -> FILETIME {
        let ticks = (ms as i128) * 10_000 + EPOCH_DIFF;
        let t = if ticks < 0 { 0u64 } else { ticks as u64 };
        FILETIME {
            dwLowDateTime: (t & 0xFFFF_FFFF) as u32,
            dwHighDateTime: (t >> 32) as u32,
        }
    }

    pub fn read_times(path: &Path) -> AppResult<TimeSpec> {
        let h = open_attr(path)?;
        let (mut c, mut a, mut w) = (
            FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            },
            FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            },
            FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            },
        );
        let ok = unsafe { GetFileTime(h.0, &mut c, &mut a, &mut w) };
        if ok == 0 {
            return Err(AppError::Io(format!(
                "读取时间失败 {}: {}",
                path.display(),
                std::io::Error::last_os_error()
            )));
        }
        Ok(TimeSpec {
            created_ms: Some(ft_to_ms(c)),
            accessed_ms: Some(ft_to_ms(a)),
            modified_ms: Some(ft_to_ms(w)),
        })
    }

    pub fn write_times(path: &Path, spec: &TimeSpec) -> AppResult<TimeSpec> {
        let before = read_times(path)?;

        let want_c = spec.created_ms.or(before.created_ms);
        let want_a = spec.accessed_ms.or(before.accessed_ms);
        let want_w = spec.modified_ms.or(before.modified_ms);

        // 一个都没变就别开句柄写了
        if want_c == before.created_ms
            && want_a == before.accessed_ms
            && want_w == before.modified_ms
        {
            return Ok(before);
        }

        let h = open_attr(path)?;
        let c = ms_to_ft(want_c.unwrap_or(0));
        let a = ms_to_ft(want_a.unwrap_or(0));
        let w = ms_to_ft(want_w.unwrap_or(0));

        let ok = unsafe { SetFileTime(h.0, &c, &a, &w) };
        if ok == 0 {
            return Err(AppError::Io(format!(
                "写入时间失败 {}: {}",
                path.display(),
                std::io::Error::last_os_error()
            )));
        }
        Ok(before)
    }
}

#[cfg(not(windows))]
mod imp {
    use super::*;

    pub fn read_times(path: &Path) -> AppResult<TimeSpec> {
        let meta = std::fs::metadata(path)?;
        Ok(TimeSpec {
            created_ms: crate::model::to_ms(meta.created().ok()),
            accessed_ms: crate::model::to_ms(meta.accessed().ok()),
            modified_ms: crate::model::to_ms(meta.modified().ok()),
        })
    }

    pub fn write_times(path: &Path, spec: &TimeSpec) -> AppResult<TimeSpec> {
        let before = read_times(path)?;
        let ft = filetime::FileTime::now();
        let _ = ft;
        if let Some(ms) = spec.modified_ms {
            let t = filetime::FileTime::from_unix_time(ms / 1000, ((ms % 1000) * 1_000_000) as u32);
            filetime::set_file_mtime(path, t)?;
        }
        if let Some(ms) = spec.accessed_ms {
            let t = filetime::FileTime::from_unix_time(ms / 1000, ((ms % 1000) * 1_000_000) as u32);
            filetime::set_file_atime(path, t)?;
        }
        Ok(before)
    }
}

pub use imp::{read_times, write_times};

/// 按「原值 + 偏移」方式修改时间
pub fn shift_times(
    path: &Path,
    delta_created_ms: i64,
    delta_modified_ms: i64,
    delta_accessed_ms: i64,
) -> AppResult<TimeSpec> {
    let before = read_times(path)?;
    let spec = TimeSpec {
        created_ms: before.created_ms.map(|v| v + delta_created_ms),
        modified_ms: before.modified_ms.map(|v| v + delta_modified_ms),
        accessed_ms: before.accessed_ms.map(|v| v + delta_accessed_ms),
    };
    write_times(path, &spec)?;
    Ok(before)
}
