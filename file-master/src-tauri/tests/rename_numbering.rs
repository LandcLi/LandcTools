//! 改名流水线的回归测试：重点盯住「编号会不会污染扩展名」。
//!
//! 这些都是用户真实会遇到的形状：多段后缀、名字里带点、名字里带编号样式数字。

use landc_file_master_lib::core::rename::{build_plan, ExtMode, NumPos, RenameRules};
use landc_file_master_lib::model::FileEntry;

/// 用真实文件名造一个条目（走和扫描完全一样的 from_path 路径，避免自己拍脑袋拆后缀）
fn entry(name: &str) -> FileEntry {
    let dir = scratch_dir();
    let p = dir.join(name);
    if !p.exists() {
        let _ = std::fs::write(&p, b"x");
    }
    let meta = std::fs::metadata(&p).expect("metadata");
    FileEntry::from_path(&p, &meta, None)
}

/// 临时目录，只用于拿 metadata
fn scratch_dir() -> &'static std::path::PathBuf {
    static DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let d = std::env::temp_dir().join("lcfm-rename-test");
        let _ = std::fs::create_dir_all(&d);
        d
    })
}

fn base() -> RenameRules {
    let mut r = RenameRules::default();
    r.numbering.start = 1;
    r.numbering.step = 1;
    r.numbering.pad = 3;
    r.numbering.position = NumPos::Prefix;
    r.sanitize = true;
    r
}

fn new_names(files: &[&str], mut r: RenameRules) -> Vec<String> {
    r.numbering.enabled = true;
    let entries: Vec<FileEntry> = files.iter().map(|n| entry(n)).collect();
    let plan = build_plan(&entries, &r).expect("plan");
    plan.items.iter().map(|i| i.new_name.clone()).collect()
}

#[test]
fn numbering_prefix_keeps_extension() {
    let out = new_names(&["照片.jpg", "报告.docx", "视频.mp4"], base());
    assert_eq!(out, vec!["001照片.jpg", "002报告.docx", "003视频.mp4"]);
}

#[test]
fn numbering_suffix_keeps_extension() {
    let mut r = base();
    r.numbering.position = NumPos::Suffix;
    let out = new_names(&["照片.jpg", "报告.docx"], r);
    assert_eq!(out, vec!["照片001.jpg", "报告002.docx"]);
}

#[test]
fn numbering_replaces_stem_only() {
    let mut r = base();
    r.numbering.position = NumPos::Replace;
    let out = new_names(&["照片.jpg", "报告.docx"], r);
    assert_eq!(out, vec!["001.jpg", "002.docx"]);
}

/// 多段后缀：`a.tar.gz` 的「主名」是 `a.tar`，编号后缀会插到 .tar 后面。
/// 这是 std::path::file_stem 的定义，不算 bug，但要确认扩展名本身没被吃掉。
#[test]
fn numbering_with_double_extension() {
    let mut r = base();
    r.numbering.position = NumPos::Suffix;
    let out = new_names(&["备份.tar.gz"], r);
    assert_eq!(out, vec!["备份.tar001.gz"]);
}

/// 名字里本身带点，不能被当成扩展名分隔
#[test]
fn numbering_with_dots_in_name() {
    let out = new_names(&["会议纪要-2024.03.12 终版.docx"], base());
    assert_eq!(out, vec!["001会议纪要-2024.03.12 终版.docx"]);
}

/// 编号 + 扩展名转换，两者互不干扰
#[test]
fn numbering_with_ext_replace_and_upper() {
    let mut r = base();
    r.ext_mode = ExtMode::Replace;
    r.ext_value = "webp".into();
    let out = new_names(&["照片.jpg"], r);
    assert_eq!(out, vec!["001照片.webp"]);

    let mut r2 = base();
    r2.ext_mode = ExtMode::Upper;
    let out2 = new_names(&["照片.jpg"], r2);
    assert_eq!(out2, vec!["001照片.JPG"]);
}

/// 编号 + 追加扩展名
#[test]
fn numbering_with_ext_add() {
    let mut r = base();
    r.ext_mode = ExtMode::Add;
    r.ext_value = "gz".into();
    let out = new_names(&["备份.tar"], r);
    assert_eq!(out, vec!["001备份.tar.gz"]);
}

/// 自定义模板里含括号 / 中括号 / 点，不能破坏扩展名
#[test]
fn numbering_custom_template() {
    let mut r = base();
    r.numbering.template = "({n})".into();
    r.numbering.position = NumPos::Suffix;
    let out = new_names(&["片头.mp4"], r);
    assert_eq!(out, vec!["片头(001).mp4"]);

    let mut r2 = base();
    r2.numbering.template = "第{n}集".into();
    r2.numbering.pad = 0;
    let out2 = new_names(&["片头.mp4"], r2);
    assert_eq!(out2, vec!["第1集片头.mp4"]);
}

/// 无扩展名文件：编号后不能凭空多出一个点
#[test]
fn numbering_without_extension() {
    let out = new_names(&["README", "Makefile"], base());
    assert_eq!(out, vec!["001README", "002Makefile"]);
}

/// 隐藏文件风格 `.gitignore`：file_stem 会把它整体当主名，不应生成 `001.gitignore`
#[test]
fn numbering_dotfile() {
    let out = new_names(&[".gitignore"], base());
    assert_eq!(out, vec!["001.gitignore"]);
}

/// 「替换后缀」但没填内容时，绝不能把扩展名剥掉（不可逆）
#[test]
fn empty_ext_replace_keeps_original() {
    let mut r = base();
    r.numbering.enabled = false;
    r.ext_mode = ExtMode::Replace;
    r.ext_value = "   ".into();
    let entries = vec![entry("照片.jpg")];
    let plan = build_plan(&entries, &r).expect("plan");
    assert_eq!(plan.items[0].new_name, "照片.jpg");
    assert!(!plan.items[0].changed, "空值不应产生任何改动");
}

/// 后缀面板场景：只给 ext 规则、其余全默认，编号绝不能跟着生效
#[test]
fn ext_only_rules_never_add_numbering() {
    let mut r = RenameRules::default();
    r.ext_mode = ExtMode::Lower;
    r.sanitize = true;
    let entries = vec![entry("照片.JPG"), entry("报告.DOCX")];
    let plan = build_plan(&entries, &r).expect("plan");
    let names: Vec<String> = plan.items.iter().map(|i| i.new_name.clone()).collect();
    assert_eq!(names, vec!["照片.jpg", "报告.docx"]);
}
