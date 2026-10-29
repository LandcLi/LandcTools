//! 改名规则引擎。
//!
//! 设计：所有规则组成一条**固定顺序的流水线**，顺序即产品语义：
//!
//! ```text
//! 主名 → 全角转半角 → 剔除字符 → 查找替换 → 大小写 → 去首尾空格
//!      → 去所有空格 → 插入文本 → 加前缀 → 加后缀 → 编号 → 非法字符清洗
//! 扩展名 → 保持 / 替换 / 转小写 / 转大写 / 追加
//! ```
//!
//! 之所以把顺序写死而不是让用户拖拽排序：普通买家理解不了「规则优先级」，
//! 给一个确定的行为反而不会有惊吓。

use crate::core::conflicts::{self, Level};
use crate::error::{AppError, AppResult};
use crate::model::FileEntry;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CaseMode {
    #[default]
    None,
    Lower,
    Upper,
    Title,
    Sentence,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NumPos {
    #[default]
    Prefix,
    Suffix,
    Replace,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExtMode {
    #[default]
    Keep,
    Replace,
    Lower,
    Upper,
    Add,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Numbering {
    pub enabled: bool,
    pub start: i64,
    pub step: i64,
    /// 补零位数，0 表示不补
    pub pad: usize,
    /// 模板，用 {n} 占位，如 "({n})"、"-第{n}集"
    pub template: String,
    pub position: NumPos,
    /// true = 每个目录独立从 start 开始计数
    pub per_dir: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameRules {
    // 1 字符归一
    pub normalize_width: bool,
    // 2 剔除
    pub remove_chars: String,
    // 3 替换
    pub find: String,
    pub replace_with: String,
    pub use_regex: bool,
    pub case_sensitive: bool,
    // 4 大小写
    pub case_mode: CaseMode,
    // 5 空白
    pub trim: bool,
    pub remove_spaces: bool,
    // 6 插入
    pub insert_text: String,
    /// 插入位置，负数表示从末尾算（-1 = 最后一个字符前）
    pub insert_position: i64,
    // 7 前后缀
    pub prefix: String,
    pub suffix: String,
    // 8 编号
    pub numbering: Numbering,
    // 9 扩展名
    pub ext_mode: ExtMode,
    pub ext_value: String,
    // 10 兜底清洗
    pub sanitize: bool,
}

impl RenameRules {
    pub fn is_noop(&self) -> bool {
        !self.normalize_width
            && self.remove_chars.is_empty()
            && self.find.is_empty()
            && self.case_mode == CaseMode::None
            && !self.trim
            && !self.remove_spaces
            && self.insert_text.is_empty()
            && self.prefix.is_empty()
            && self.suffix.is_empty()
            && !self.numbering.enabled
            && self.ext_mode == ExtMode::Keep
            && !self.sanitize
    }

    /// 对主名应用整条流水线
    pub fn transform_stem(&self, stem: &str, seq: usize) -> String {
        let mut s = stem.to_string();

        if self.normalize_width {
            s = to_half_width(&s);
        }

        if !self.remove_chars.is_empty() {
            let set: Vec<char> = self.remove_chars.chars().collect();
            s = s.chars().filter(|c| !set.contains(c)).collect();
        }

        if !self.find.is_empty() {
            s = self.do_replace(&s);
        }

        match self.case_mode {
            CaseMode::None => {}
            CaseMode::Lower => s = s.to_lowercase(),
            CaseMode::Upper => s = s.to_uppercase(),
            CaseMode::Title => s = to_title_case(&s),
            CaseMode::Sentence => s = to_sentence_case(&s),
        }

        if self.trim {
            s = s.trim().to_string();
        }
        if self.remove_spaces {
            s = s.chars().filter(|c| !c.is_whitespace()).collect();
        }

        if !self.insert_text.is_empty() {
            s = insert_at(&s, &self.insert_text, self.insert_position);
        }

        if !self.prefix.is_empty() {
            s = format!("{}{}", self.prefix, s);
        }
        if !self.suffix.is_empty() {
            s = format!("{}{}", s, self.suffix);
        }

        if self.numbering.enabled {
            s = self.apply_numbering(&s, seq);
        }

        if self.sanitize {
            s = conflicts::sanitize(&s);
        }

        s
    }

    fn do_replace(&self, s: &str) -> String {
        if self.use_regex {
            let pattern = if self.case_sensitive {
                self.find.clone()
            } else {
                format!("(?i){}", self.find)
            };
            match Regex::new(&pattern) {
                Ok(re) => re.replace_all(s, self.replace_with.as_str()).to_string(),
                // 正则非法时原样返回，前端会在预览时立刻看出来
                Err(_) => s.to_string(),
            }
        } else if self.case_sensitive {
            s.replace(&self.find, &self.replace_with)
        } else {
            // 大小写不敏感的普通替换
            let pattern = format!("(?i){}", regex::escape(&self.find));
            match Regex::new(&pattern) {
                Ok(re) => re.replace_all(s, self.replace_with.as_str()).to_string(),
                Err(_) => s.replace(&self.find, &self.replace_with),
            }
        }
    }

    fn apply_numbering(&self, s: &str, seq: usize) -> String {
        let n = self.numbering.start + (seq as i64) * self.numbering.step;
        let num = format_num(n, self.numbering.pad);
        let tpl = if self.numbering.template.trim().is_empty() {
            "{n}"
        } else {
            &self.numbering.template
        };
        let token = tpl.replace("{n}", &num);
        match self.numbering.position {
            NumPos::Prefix => format!("{token}{s}"),
            NumPos::Suffix => format!("{s}{token}"),
            NumPos::Replace => token,
        }
    }

    pub fn transform_ext(&self, ext: &str) -> String {
        match self.ext_mode {
            ExtMode::Keep => ext.to_string(),
            ExtMode::Replace => self.ext_value.trim().trim_start_matches('.').to_lowercase(),
            ExtMode::Lower => ext.to_lowercase(),
            ExtMode::Upper => ext.to_uppercase(),
            ExtMode::Add => {
                let add = self.ext_value.trim().trim_start_matches('.').to_lowercase();
                if add.is_empty() {
                    ext.to_string()
                } else if ext.is_empty() {
                    add
                } else {
                    format!("{ext}.{add}")
                }
            }
        }
    }
}

fn format_num(n: i64, pad: usize) -> String {
    if pad == 0 || n < 0 {
        return n.to_string();
    }
    let neg = n < 0;
    let abs = n.unsigned_abs().to_string();
    let padded = format!("{:0>width$}", abs, width = pad);
    if neg {
        format!("-{padded}")
    } else {
        padded
    }
}

/// 全角 → 半角（含全角空格）
pub fn to_half_width(s: &str) -> String {
    s.chars()
        .map(|c| match c as u32 {
            0x3000 => ' ',
            0xFF01..=0xFF5E => char::from_u32(c as u32 - 0xFEE0).unwrap_or(c),
            _ => c,
        })
        .collect()
}

fn insert_at(s: &str, text: &str, pos: i64) -> String {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let idx = if pos < 0 {
        (len + pos).max(0)
    } else {
        pos.min(len)
    } as usize;
    let mut out: String = chars[..idx].iter().collect();
    out.push_str(text);
    out.extend(chars[idx..].iter());
    out
}

fn to_title_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_sep = true;
    for c in s.chars() {
        let is_sep = c == ' ' || c == '_' || c == '-' || c == '.';
        if prev_sep && c.is_alphabetic() {
            out.extend(c.to_uppercase());
        } else if !is_sep {
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
        prev_sep = is_sep;
    }
    out
}

fn to_sentence_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut first = true;
    for c in s.chars() {
        if first && c.is_alphabetic() {
            out.extend(c.to_uppercase());
            first = false;
        } else {
            if !first {
                out.extend(c.to_lowercase());
            } else {
                out.push(c);
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// 预览计划
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanItem {
    pub id: String,
    pub dir: String,
    pub old_name: String,
    pub new_name: String,
    pub old_path: String,
    pub new_path: String,
    pub changed: bool,
    /// ok / warn / error
    pub level: Level,
    pub issues: Vec<String>,
    /// 该目标路径是否被本批次其他文件占用（需要两阶段改名）
    pub needs_two_phase: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenamePlan {
    pub items: Vec<PlanItem>,
    pub total: usize,
    pub changed: usize,
    pub unchanged: usize,
    pub errors: usize,
    pub warnings: usize,
    /// 正则等规则本身的语法错误
    pub rule_error: Option<String>,
}

/// 根据规则为每条记录计算新名字，并做全量冲突检测
pub fn build_plan(entries: &[FileEntry], rules: &RenameRules) -> AppResult<RenamePlan> {
    // 规则自检：正则非法直接拦下来
    if rules.use_regex && !rules.find.is_empty() {
        let pattern = if rules.case_sensitive {
            rules.find.clone()
        } else {
            format!("(?i){}", rules.find)
        };
        if let Err(e) = Regex::new(&pattern) {
            return Ok(RenamePlan {
                items: vec![],
                total: 0,
                changed: 0,
                unchanged: 0,
                errors: 0,
                warnings: 0,
                rule_error: Some(format!("正则表达式有误：{e}")),
            });
        }
    }

    let mut items: Vec<PlanItem> = Vec::with_capacity(entries.len());
    let mut counters: HashMap<String, usize> = HashMap::new();
    // 本批次所有源文件路径（小写），用于判断「目标路径其实是我们自己人」
    let mut source_set: HashMap<String, ()> = HashMap::new();

    for e in entries {
        source_set.insert(e.path.to_lowercase(), ());
    }

    for e in entries {
        // 目录改名会连带影响子项路径，第一阶段不支持，明确跳过并提示
        if e.is_dir {
            items.push(PlanItem {
                id: e.id.clone(),
                dir: e.dir.clone(),
                old_name: e.name.clone(),
                new_name: e.name.clone(),
                old_path: e.path.clone(),
                new_path: e.path.clone(),
                changed: false,
                level: Level::Warn,
                issues: vec!["目录改名暂不支持，已跳过".into()],
                needs_two_phase: false,
            });
            continue;
        }

        let key = if rules.numbering.enabled && rules.numbering.per_dir {
            e.dir.clone()
        } else {
            String::new()
        };
        let seq = *counters.get(&key).unwrap_or(&0);
        counters.insert(key, seq + 1);

        let new_stem = rules.transform_stem(&e.stem, seq);
        let new_ext = rules.transform_ext(&e.ext);
        let new_name = if new_ext.is_empty() {
            new_stem.clone()
        } else {
            format!("{new_stem}.{new_ext}")
        };

        let new_path = if e.dir.is_empty() {
            new_name.clone()
        } else {
            Path::new(&e.dir)
                .join(&new_name)
                .to_string_lossy()
                .to_string()
        };

        items.push(PlanItem {
            id: e.id.clone(),
            dir: e.dir.clone(),
            old_name: e.name.clone(),
            new_name,
            old_path: e.path.clone(),
            new_path,
            changed: false, // 下面统一算
            level: Level::Ok,
            issues: vec![],
            needs_two_phase: false,
        });
    }

    // ---- 单条校验 ----
    for item in items.iter_mut() {
        if item.old_name == item.new_name {
            item.changed = false;
            continue;
        }
        item.changed = true;

        let mut issues: Vec<(Level, String)> = Vec::new();
        issues.extend(conflicts::name_issues(&item.new_name));
        if let Some(p) = conflicts::path_issue(&item.new_path) {
            issues.push(p);
        }
        for (lv, msg) in issues {
            if lv == Level::Error {
                item.level = Level::Error;
            } else if item.level == Level::Ok {
                item.level = Level::Warn;
            }
            item.issues.push(msg);
        }
    }

    // ---- 批次内重名检测（Windows 大小写不敏感） ----
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, item) in items.iter().enumerate() {
        if !item.changed {
            continue;
        }
        let key = format!(
            "{}\u{1}{}",
            item.dir.to_lowercase(),
            item.new_name.to_lowercase()
        );
        groups.entry(key).or_default().push(i);
    }
    for (_, idxs) in groups.iter() {
        if idxs.len() > 1 {
            for &i in idxs {
                items[i].level = Level::Error;
                items[i].issues.push(format!(
                    "批次内有 {} 个文件会变成同一个名字 `{}`",
                    idxs.len(),
                    items[i].new_name
                ));
            }
        }
    }

    // ---- 目标路径已被占用检测 ----
    for item in items.iter_mut() {
        if !item.changed {
            continue;
        }
        let target = Path::new(&item.new_path);
        let target_lower = item.new_path.to_lowercase();
        if target.exists() && !source_set.contains_key(&target_lower) {
            item.level = Level::Error;
            item.issues
                .push("目标位置已存在同名文件".to_string());
        }
        if source_set.contains_key(&target_lower) && target_lower != item.old_path.to_lowercase() {
            item.needs_two_phase = true;
        }
    }

    // 只要有一条需要两阶段，整批都按两阶段走更安全（代码简单、无边界 case）
    let batch_needs_tp = items.iter().any(|i| i.needs_two_phase);
    if batch_needs_tp {
        for item in items.iter_mut() {
            if item.changed && item.level != Level::Error {
                item.needs_two_phase = true;
            }
        }
    }

    let changed = items.iter().filter(|i| i.changed).count();
    let errors = items.iter().filter(|i| i.level == Level::Error).count();
    let warnings = items.iter().filter(|i| i.level == Level::Warn).count();

    Ok(RenamePlan {
        total: items.len(),
        changed,
        unchanged: items.len() - changed,
        errors,
        warnings,
        rule_error: None,
        items,
    })
}
