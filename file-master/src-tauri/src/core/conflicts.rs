//! 冲突检测：在真正落盘之前，把所有会导致失败或误操作的情况全找出来。
//!
//! 这是批量改名工具最容易翻车的地方 —— 一旦改错，用户几百个文件就废了。
//! 所以规则是：**只要有一条 error，整批不允许执行**。

use serde::{Deserialize, Serialize};

/// Windows 文件名非法字符
pub const ILLEGAL_CHARS: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// Windows 保留设备名（主名部分，不含扩展名）
pub const RESERVED_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM0", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
    "COM8", "COM9", "LPT0", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Ok,
    Warn,
    Error,
}

/// 检查单个文件名是否合法，返回问题描述
pub fn name_issues(name: &str) -> Vec<(Level, String)> {
    let mut out = Vec::new();

    if name.trim().is_empty() {
        out.push((Level::Error, "文件名为空".into()));
        return out;
    }

    if let Some(c) = name.chars().find(|c| ILLEGAL_CHARS.contains(c)) {
        out.push((
            Level::Error,
            format!("包含非法字符 `{c}`（Windows 不允许 < > : \" / \\ | ? *）"),
        ));
    }

    if name.chars().any(|c| (c as u32) < 32) {
        out.push((Level::Error, "包含不可见控制字符".into()));
    }

    if name.ends_with(' ') || name.ends_with('.') {
        out.push((Level::Error, "结尾不能是空格或点号".into()));
    }

    // 保留设备名
    let stem = name.split('.').next().unwrap_or(name).trim().to_uppercase();
    if RESERVED_NAMES.contains(&stem.as_str()) {
        out.push((
            Level::Error,
            format!("`{stem}` 是 Windows 保留设备名，不能作为文件名"),
        ));
    }

    if name.chars().count() > 200 {
        out.push((Level::Warn, "文件名过长，可能超出路径长度限制".into()));
    }

    out
}

/// 检查完整路径长度
pub fn path_issue(path: &str) -> Option<(Level, String)> {
    // Windows 传统 MAX_PATH 限制 260
    if path.chars().count() > 259 {
        return Some((
            Level::Warn,
            "完整路径超过 259 字符，Windows 可能无法处理（需开启长路径支持）".into(),
        ));
    }
    None
}

/// 清洗文件名：把非法字符换成下划线，去掉尾部空格与点
pub fn sanitize(name: &str) -> String {
    let mut s: String = name
        .chars()
        .filter(|c| (*c as u32) >= 32)
        .map(|c| if ILLEGAL_CHARS.contains(&c) { '_' } else { c })
        .collect();
    while s.ends_with(' ') || s.ends_with('.') {
        s.pop();
    }
    if s.is_empty() {
        s.push('_');
    }
    s
}
