//! 标签读写端到端回归测试：用真实文件（docx 包 / PDF / PNG）跑完整链路。
//!
//! 这些用例的价值在于「拿真东西跑一遍」——单元测试只能证明 XML 拼得对，
//! 证明不了 zip 重打包后 Office 还能打开、lopdf 存完 PDF 还有 Info 字典。

use landc_file_master_lib::core::meta;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::PathBuf;

fn tmp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "lfm_meta_{tag}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn patch(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// 造一个最小 docx：只有 [Content_Types].xml 和 docProps/core.xml
fn make_docx(path: &std::path::Path, title: &str, revision: &str) {
    let f = std::fs::File::create(path).unwrap();
    let mut zw = zip::ZipWriter::new(f);
    let opts = zip::write::SimpleFileOptions::default();

    zw.start_file("[Content_Types].xml", opts).unwrap();
    zw.write_all(br#"<?xml version="1.0"?><Types/>"#).unwrap();

    zw.start_file("docProps/core.xml", opts).unwrap();
    let xml = format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" "#,
            r#"xmlns:dc="http://purl.org/dc/elements/1.1/">"#,
            r#"<dc:title>{}</dc:title><dc:creator>原作者</dc:creator><cp:revision>{}</cp:revision>"#,
            r#"</cp:coreProperties>"#
        ),
        title, revision
    );
    zw.write_all(xml.as_bytes()).unwrap();

    // 混一个大一点的部件，验证 raw_copy_file 搬运后内容不变
    zw.start_file("word/document.xml", opts).unwrap();
    zw.write_all(&vec![b'A'; 64 * 1024]).unwrap();
    zw.finish().unwrap();
}

#[test]
fn office_read_write_roundtrip() {
    let dir = tmp_dir("office");
    let p = dir.join("测试文档.docx");
    make_docx(&p, "旧标题", "3");

    let before = meta::read(&p);
    assert_eq!(before.format, meta::MetaFormat::Office);
    assert!(before.writable);
    assert_eq!(before.fields.title.as_deref(), Some("旧标题"));
    assert_eq!(before.fields.artist.as_deref(), Some("原作者"));

    let old = meta::write(
        &p,
        &patch(&[
            ("title", "新标题 & <符号>"),
            ("artist", "张三"),
            ("keywords", "合同,2026"),
        ]),
        true,
    )
    .unwrap();
    assert_eq!(old.title.as_deref(), Some("旧标题"), "应返回写入前的值");

    let after = meta::read(&p);
    assert_eq!(after.fields.title.as_deref(), Some("新标题 & <符号>"));
    assert_eq!(after.fields.artist.as_deref(), Some("张三"));
    assert_eq!(after.fields.keywords.as_deref(), Some("合同,2026"));
    // 没改的字段不能被清掉
    assert_eq!(after.fields.subject.as_deref(), Some(""));

    // 包内其他部件必须原样存在且完整
    let f = std::fs::File::open(&p).unwrap();
    let mut zip = zip::ZipArchive::new(f).unwrap();
    assert!(zip.by_name("word/document.xml").is_ok());
    assert!(zip.by_name("docProps/core.xml").is_ok());
    let mut doc = Vec::new();
    zip.by_name("word/document.xml")
        .unwrap()
        .read_to_end(&mut doc)
        .unwrap();
    assert_eq!(doc.len(), 64 * 1024);

    // revision 这类我们没碰的标签要保留
    let mut core = String::new();
    zip.by_name("docProps/core.xml")
        .unwrap()
        .read_to_string(&mut core)
        .unwrap();
    assert!(core.contains("<cp:revision>3</cp:revision>"), "core.xml: {core}");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn office_clear_field() {
    let dir = tmp_dir("office_clear");
    let p = dir.join("a.docx");
    make_docx(&p, "要清掉的标题", "1");

    meta::write(&p, &patch(&[("title", "")]), false).unwrap();
    let after = meta::read(&p);
    assert_eq!(after.fields.title.as_deref(), Some(""));
    std::fs::remove_dir_all(&dir).ok();
}

fn make_pdf(path: &std::path::Path) {
    use lopdf::dictionary;
    use lopdf::{Document, Object};

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 200.into(), 200.into()],
    });
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        }),
    );
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);
    doc.save(path).unwrap();
}

#[test]
fn pdf_read_write_roundtrip() {
    let dir = tmp_dir("pdf");
    let p = dir.join("测试.pdf");
    make_pdf(&p);

    let before = meta::read(&p);
    assert_eq!(before.format, meta::MetaFormat::Pdf);
    assert!(before.writable, "新建 PDF 应可写：{:?}", before.reason);
    assert_eq!(before.fields.title.as_deref(), Some(""));

    meta::write(
        &p,
        &patch(&[
            ("title", "中文标题测试"),
            ("artist", "李四"),
            ("subject", "主题"),
        ]),
        false,
    )
    .unwrap();

    let after = meta::read(&p);
    assert_eq!(after.fields.title.as_deref(), Some("中文标题测试"));
    assert_eq!(after.fields.artist.as_deref(), Some("李四"));
    assert_eq!(after.fields.subject.as_deref(), Some("主题"));

    // 存完还是能正常解析的 PDF
    let doc = lopdf::Document::load(&p).unwrap();
    assert!(doc.trailer.get(b"Info").is_ok());
    assert!(doc.get_pages().len() == 1);

    std::fs::remove_dir_all(&dir).ok();
}

/// 1x1 白点 PNG（标准最小 PNG，含 IHDR/IDAT/IEND）
fn make_png(path: &std::path::Path) {
    // 预先算好的最小 PNG 字节
    const PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    std::fs::write(path, PNG).unwrap();
}

#[test]
fn image_exif_roundtrip() {
    let dir = tmp_dir("image");
    let p = dir.join("测试.png");
    make_png(&p);

    let before = meta::read(&p);
    assert_eq!(before.format, meta::MetaFormat::Image);
    assert!(before.writable, "读取应正常：{:?}", before.reason);

    let r = meta::write(
        &p,
        &patch(&[("comment", "批量描述"), ("artist", "摄影师")]),
        false,
    );
    assert!(r.is_ok(), "写 EXIF 失败：{:?}", r.err());

    let after = meta::read(&p);
    assert_eq!(after.fields.comment.as_deref(), Some("批量描述"));
    assert_eq!(after.fields.artist.as_deref(), Some("摄影师"));

    // 图片本体不能被破坏
    let bytes = std::fs::read(&p).unwrap();
    assert_eq!(&bytes[..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    assert!(bytes.len() > 60);

    std::fs::remove_dir_all(&dir).ok();
}

/// 音频这条链路依赖一份真实音频样本。系统自带 wav 就拿来用，
/// 没有就跳过（CI 环境没有 Windows Media 目录）。
#[test]
fn audio_wav_roundtrip() {
    let src = std::path::PathBuf::from("C:/Windows/Media/Alarm01.wav");
    if !src.exists() {
        eprintln!("跳过：找不到系统音频样本");
        return;
    }
    let dir = tmp_dir("audio");
    let p = dir.join("测试音频.wav");
    std::fs::copy(&src, &p).unwrap();

    let before = meta::read(&p);
    assert_eq!(before.format, meta::MetaFormat::Audio);
    assert!(before.writable, "读取应正常：{:?}", before.reason);

    let r = meta::write(
        &p,
        &patch(&[
            ("title", "测试歌名"),
            ("artist", "歌手A"),
            ("album", "专辑B"),
            ("year", "2026"),
        ]),
        false,
    );
    assert!(r.is_ok(), "写音频标签失败：{:?}", r.err());

    let after = meta::read(&p);
    assert_eq!(after.fields.title.as_deref(), Some("测试歌名"));
    assert_eq!(after.fields.artist.as_deref(), Some("歌手A"));
    assert_eq!(after.fields.album.as_deref(), Some("专辑B"));
    assert_eq!(after.fields.year, Some(2026));

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn unsupported_format_reports_reason() {
    let dir = tmp_dir("bad");
    let p = dir.join("x.unknownext");
    std::fs::write(&p, b"hello").unwrap();
    let info = meta::read(&p);
    assert!(!info.writable);
    assert!(info.reason.is_some());
    assert!(meta::write(&p, &patch(&[("title", "x")]), false).is_err());
    std::fs::remove_dir_all(&dir).ok();
}
