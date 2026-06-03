pub mod adapter;
pub mod traffic;

use chrono::Datelike;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

/// 网卡信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterInfo {
    pub name: String,
    pub alias: String,
    pub ip_address: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedData {
    pub download_bytes_per_sec: u64,
    pub upload_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficStats {
    pub download: u64,
    pub upload: u64,
    pub total: u64,
}

impl TrafficStats {
    pub fn new() -> Self {
        Self { download: 0, upload: 0, total: 0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub connected: bool,
    pub ip_address: String,
    pub adapter_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub adapters: Vec<AdapterInfo>,
    pub selected_adapter: String,
    pub speed: SpeedData,
    pub today: TrafficStats,
    pub month: TrafficStats,
    pub status: NetworkStatus,
    #[serde(skip)]
    pub last_in_bytes: u64,
    #[serde(skip)]
    pub last_out_bytes: u64,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            adapters: Vec::new(),
            selected_adapter: String::new(),
            speed: SpeedData { download_bytes_per_sec: 0, upload_bytes_per_sec: 0 },
            today: TrafficStats::new(),
            month: TrafficStats::new(),
            status: NetworkStatus {
                connected: false,
                ip_address: String::new(),
                adapter_name: String::new(),
            },
            last_in_bytes: 0,
            last_out_bytes: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedStats {
    date: String,
    month_key: String,
    today_download: u64,
    today_upload: u64,
    month_download: u64,
    month_upload: u64,
}

/// 后台采集线程：恢复数据 → 每秒采集 → 每30秒存盘
pub fn run_collector(state: Arc<Mutex<AppState>>, data_file: PathBuf) {
    // 1. 从磁盘恢复统计数据
    if data_file.exists() {
        if let Ok(content) = fs::read_to_string(&data_file) {
            if let Ok(saved) = serde_json::from_str::<PersistedStats>(&content) {
                let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                let month_key = chrono::Local::now().format("%Y-%m").to_string();
                let mut s = state.lock();
                // 只有日期匹配才恢复（否则从零开始）
                if saved.date == today {
                    s.today.download = saved.today_download;
                    s.today.upload = saved.today_upload;
                    s.today.total = saved.today_download + saved.today_upload;
                }
                if saved.month_key == month_key {
                    s.month.download = saved.month_download;
                    s.month.upload = saved.month_upload;
                    s.month.total = saved.month_download + saved.month_upload;
                }
            }
        }
    }

    // 2. 初始化网卡列表
    {
        let mut s = state.lock();
        s.adapters = adapter::get_adapters();
        if !s.adapters.is_empty() {
            let default_name;
            let default_alias;
            let default_ip;
            {
                let default = s.adapters.iter().find(|a| a.is_default)
                    .unwrap_or(&s.adapters[0]);
                default_name = default.name.clone();
                default_alias = default.alias.clone();
                default_ip = default.ip_address.clone();
            }
            s.selected_adapter = default_name;
            s.status.adapter_name = default_alias;
            let has_ip = !default_ip.is_empty();
            s.status.ip_address = default_ip;
            s.status.connected = has_ip;
        }
    }

    let mut last_day = chrono::Local::now().day();
    let mut last_month = chrono::Local::now().month();
    let mut tick: u64 = 0;

    // 3. 主循环
    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        tick += 1;

        let today = chrono::Local::now();
        let mut s = state.lock();

        // 每日/每月重置
        if today.day() != last_day {
            s.today = TrafficStats::new();
            last_day = today.day();
        }
        if today.month() != last_month {
            s.month = TrafficStats::new();
            last_month = today.month();
        }

        // 获取当前网速并累加
        if !s.selected_adapter.is_empty() {
            let speed_opt = traffic::get_adapter_speed(&mut s);
            if let Some(speed) = speed_opt {
                s.speed = speed.clone();
                s.today.download += speed.download_bytes_per_sec;
                s.today.upload += speed.upload_bytes_per_sec;
                s.today.total = s.today.download + s.today.upload;
                s.month.download += speed.download_bytes_per_sec;
                s.month.upload += speed.upload_bytes_per_sec;
                s.month.total = s.month.download + s.month.upload;
            }
            s.status.connected = !s.status.ip_address.is_empty();
        }

        // 每 30 秒存盘
        if tick % 30 == 0 {
            let data = PersistedStats {
                date: today.format("%Y-%m-%d").to_string(),
                month_key: today.format("%Y-%m").to_string(),
                today_download: s.today.download,
                today_upload: s.today.upload,
                month_download: s.month.download,
                month_upload: s.month.upload,
            };
            if let Ok(json) = serde_json::to_string(&data) {
                let _ = fs::write(&data_file, json);
            }
        }
    }
}
