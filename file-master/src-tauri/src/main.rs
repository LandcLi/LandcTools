#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // 便携版没有安装器，缺 WebView2 时 Tauri 会静默失败，所以必须先探测再启动
    if !landc_file_master_lib::preflight::ensure_webview2() {
        std::process::exit(1);
    }
    landc_file_master_lib::run()
}
