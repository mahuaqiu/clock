use tauri::{Emitter, WebviewWindowBuilder};

// 解析命令行参数
fn parse_args() -> (Option<String>, Option<(f64, f64)>) {
    let args: Vec<String> = std::env::args().collect();
    let mut text = None;
    let mut center = None;

    for arg in &args {
        if let Some(value) = arg.strip_prefix("--text=") {
            text = Some(value.to_string());
        } else if let Some(value) = arg.strip_prefix("--center=") {
            let parts: Vec<&str> = value.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(x), Ok(y)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                    center = Some((x, y));
                }
            }
        }
    }

    (text, center)
}

// 退出应用命令
#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 先解析参数（在窗口创建之前）
    let (text_value, center_value) = parse_args();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![quit_app])
        .setup(move |app| {
            // 从配置获取窗口信息
            let config = app.config().clone();
            let windows_config = &config.app.windows;

            if let Some(window_config) = windows_config.first() {
                // 使用 WebviewWindowBuilder 手动创建窗口
                let mut builder = WebviewWindowBuilder::from_config(app, window_config)?;

                if let Some((cx, cy)) = center_value {
                    // 窗口逻辑尺寸（从配置获取）
                    let logical_w = window_config.width as f64;
                    let logical_h = window_config.height as f64;
                    let x = cx - logical_w / 2.0;
                    let y = cy - logical_h / 2.0;
                    builder = builder.position(x, y);
                } else {
                    builder = builder.center();
                }

                let window = builder.build().expect("创建窗口失败");
                window.set_title("毫秒时钟").unwrap();

                // 发送 text 参数给前端
                if let Some(text) = &text_value {
                    let _ = window.emit("setup-args", serde_json::json!({ "text": text }));
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
