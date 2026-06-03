mod collector;
mod commands;
mod storage;

use collector::AppState;
use parking_lot::Mutex;
use std::fs;
use std::sync::Arc;
use tauri::Manager;

pub fn run() {
    let app_state = Arc::new(Mutex::new(AppState::new()));

    tauri::Builder::default()
        .setup(move |app| {
            // 确定数据文件路径
            let data_dir = app.path().app_data_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let _ = fs::create_dir_all(&data_dir);
            let data_file = data_dir.join("network_monitor_stats.json");

            // 启动后台采集线程（内部会先恢复数据，再开始采集，定时存盘）
            let state_clone = app_state.clone();
            std::thread::spawn(move || {
                collector::run_collector(state_clone, data_file);
            });

            app.manage(app_state.clone());

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
