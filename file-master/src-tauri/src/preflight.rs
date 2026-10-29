//! 启动前置检查。
//!
//! 绿色便携版 exe 没有安装器，所以 Tauri 的 `webviewInstallMode` 对我们无效。
//! 一旦目标机器缺 WebView2，Tauri 会在创建窗口时失败 —— 表现是**双击完全没反应**，
//! 用户会以为软件是坏的，这是最伤售后的失败模式。
//!
//! 这里在创建任何窗口之前先探测运行时：缺失则弹一个中文对话框说明原因，
//! 并支持一键打开微软官方下载页。加了这一步，失败就从「没反应」变成「明确告诉你缺什么」。

#[cfg(windows)]
pub fn ensure_webview2() -> bool {
    // 成功返回版本号字符串，失败说明运行时不可用
    match tauri::webview_version() {
        Ok(_) => true,
        Err(_) => {
            show_missing_dialog();
            false
        }
    }
}

#[cfg(not(windows))]
pub fn ensure_webview2() -> bool {
    true
}

#[cfg(windows)]
fn show_missing_dialog() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    #[link(name = "user32")]
    extern "system" {
        fn MessageBoxW(hwnd: isize, text: *const u16, caption: *const u16, u_type: u32) -> i32;
    }

    #[link(name = "shell32")]
    extern "system" {
        fn ShellExecuteW(
            hwnd: isize,
            operation: *const u16,
            file: *const u16,
            parameters: *const u16,
            directory: *const u16,
            show_cmd: i32,
        ) -> isize;
    }

    const MB_OKCANCEL: u32 = 0x0000_0001;
    const MB_ICONWARNING: u32 = 0x0000_0030;
    const IDOK: i32 = 1;
    const SW_SHOWNORMAL: i32 = 1;

    // 用 concat! 逐行拼接：Rust 的 "\" 续行会吞掉下一行所有前导空白（含全角空格），
    // 想保留列表缩进就不能用续行写法
    let text = concat!(
        "启动失败：缺少 Microsoft Edge WebView2 运行环境。\n\n",
        "这是 Windows 自带的网页渲染组件，Windows 11 全部预装，Windows 10 绝大多数也已内置。\n",
        "你的系统里没有检测到它，所以界面无法显示。\n\n",
        "点「确定」打开微软官方下载页：\n",
        "  1. 下载「Evergreen Bootstrapper」（约 2MB，在线安装）\n",
        "  2. 双击运行，全程自动，约 1 分钟\n",
        "  3. 装完重新打开本程序即可\n\n",
        "如果这台电脑不能上网，请在能上网的电脑上下载「Evergreen Standalone Installer」",
        "拷贝过来安装。",
    );

    let caption = "Landc 文件大师";
    let t = wide(text);
    let c = wide(caption);

    let ret = unsafe { MessageBoxW(0, t.as_ptr(), c.as_ptr(), MB_OKCANCEL | MB_ICONWARNING) };

    if ret == IDOK {
        let url = "https://developer.microsoft.com/zh-cn/microsoft-edge/webview2/";
        let op = wide("open");
        let u = wide(url);
        unsafe {
            ShellExecuteW(
                0,
                op.as_ptr(),
                u.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            );
        }
    }
}
