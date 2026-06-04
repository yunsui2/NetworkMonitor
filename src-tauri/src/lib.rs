mod collector;
mod commands;
mod storage;

use collector::AppState;
use parking_lot::Mutex;
use std::fs;
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};

/// 窗口宽高比
const ASPECT_RATIO: f64 = 283.0 / 188.0;

pub fn run() {
    let app_state = Arc::new(Mutex::new(AppState::new()));

    tauri::Builder::default()
        .setup(move |app| {
            // 数据目录
            let data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            let _ = fs::create_dir_all(&data_dir);
            let data_file = data_dir.join("network_monitor_stats.json");

            // ---- 迁移旧格式数据 ----
            migrate_old_format(&data_file);

            // 启动采集线程
            let state_clone = app_state.clone();
            std::thread::spawn(move || {
                collector::run_collector(state_clone, data_file);
            });

            app.manage(app_state.clone());

            // ---- 窗口设置 ----
            if let Some(window) = app.get_webview_window("main") {
                // 等比缩放
                let w = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Resized(size) = event {
                        let new_h = (size.width as f64 / ASPECT_RATIO).round() as u32;
                        if (size.height as f64 - new_h as f64).abs() > 1.0 {
                            let _ = w.set_size(tauri::PhysicalSize::new(size.width, new_h));
                        }
                    }
                });

                // 关闭 → 最小化到托盘
                let win = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win.hide();
                    }
                });
            }

            // ---- 系统托盘 ----
            let icon_img = Image::from_bytes(include_bytes!("../icons/32x32.png"))?;

            let show_item = MenuItemBuilder::with_id("show", "显示").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;
            let tray_menu = MenuBuilder::new(app)
                .item(&show_item)
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(icon_img)
                .menu(&tray_menu)
                .tooltip("Flux")
                .on_menu_event(move |app_handle, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app_handle.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => {
                        app_handle.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray_icon, event| {
                    if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                        // 托盘图标也支持双击打开
                        if let Some(w) = tray_icon.app_handle().get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_full_state,
            commands::select_adapter,
            commands::get_adapters,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 将 V1 旧格式迁移到 V2 新格式
fn migrate_old_format(data_file: &std::path::Path) {
    if !data_file.exists() {
        return;
    }

    let content = match fs::read_to_string(data_file) {
        Ok(c) => c,
        Err(_) => return,
    };

    // V1 旧格式: {"date":"...","month_key":"...","today_download":...,"today_upload":...,"month_download":...,"month_upload":...}
    #[derive(serde::Deserialize)]
    struct OldFormat {
        date: String,
        today_download: u64,
        today_upload: u64,
    }

    // 探测是否为旧格式（有 date 字段但没有 daily_records）
    if let Ok(old) = serde_json::from_str::<OldFormat>(&content) {
        // 检查是否已经是新格式
        if content.contains("\"daily_records\"") {
            return; // 已是新格式，跳过
        }

        // 迁移：将旧的今日数据写入每日记录
        let new_data = serde_json::json!({
            "daily_records": {
                old.date.clone(): {
                    "date": old.date,
                    "download": old.today_download,
                    "upload": old.today_upload,
                    "total": old.today_download + old.today_upload
                }
            }
        });

        if let Ok(json) = serde_json::to_string(&new_data) {
            let _ = fs::write(data_file, json);
            eprintln!("Migrated old format data to new format");
        }
    }
}
