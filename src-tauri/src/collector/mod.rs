pub mod adapter;
pub mod traffic;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterInfo {
    pub name: String,
    pub alias: String,
    pub ip_address: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficStats {
    pub download: u64,
    pub upload: u64,
    pub total: u64,
}

impl TrafficStats {
    pub fn new() -> Self { Self { download: 0, upload: 0, total: 0 } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyRecord {
    pub date: String,
    pub download: u64,
    pub upload: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyRecord {
    pub month: String,
    pub download: u64,
    pub upload: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub adapters: Vec<AdapterInfo>,
    pub selected_adapter: String,
    pub today: TrafficStats,
    pub month: TrafficStats,
    pub daily_records: Vec<DailyRecord>,
    pub monthly_records: Vec<MonthlyRecord>,
    pub ip_address: String,
    pub connected: bool,
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
            today: TrafficStats::new(),
            month: TrafficStats::new(),
            daily_records: Vec::new(),
            monthly_records: Vec::new(),
            ip_address: String::new(),
            connected: false,
            last_in_bytes: 0,
            last_out_bytes: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedData {
    daily_records: BTreeMap<String, DailyRecord>,
}

/// 后台采集线程
pub fn run_collector(state: Arc<Mutex<AppState>>, data_file: PathBuf) {
    // 1. 从磁盘恢复历史数据
    let mut daily_map: BTreeMap<String, DailyRecord> = BTreeMap::new();
    if data_file.exists() {
        if let Ok(content) = fs::read_to_string(&data_file) {
            if let Ok(saved) = serde_json::from_str::<PersistedData>(&content) {
                daily_map = saved.daily_records;
            }
        }
    }

    // 2. 初始化网卡
    {
        let mut s = state.lock();
        s.adapters = adapter::get_adapters();
        if !s.adapters.is_empty() {
            let (name, _alias, ip) = {
                let d = s.adapters.iter().find(|a| a.is_default)
                    .unwrap_or(&s.adapters[0]);
                (d.name.clone(), d.alias.clone(), d.ip_address.clone())
            };
            s.selected_adapter = name;
            s.ip_address = ip.clone();
            s.connected = !ip.is_empty();
        }
    }

    let mut last_save_day = String::new();
    let mut tick: u64 = 0;

    // 3. 主循环
    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        tick += 1;

        let today_str = chrono::Local::now().format("%Y-%m-%d").to_string();
        let month_str = chrono::Local::now().format("%Y-%m").to_string();
        let mut s = state.lock();

        // 采集网速
        let (dl, ul) = if !s.selected_adapter.is_empty() {
            traffic::get_adapter_speed(&mut s).unwrap_or((0, 0))
        } else {
            (0, 0)
        };

        // 累加到内存中的 today
        s.today.download += dl;
        s.today.upload += ul;
        s.today.total = s.today.download + s.today.upload;

        // 更新/插入当日记录
        let entry = daily_map.entry(today_str.clone()).or_insert_with(|| DailyRecord {
            date: today_str.clone(),
            download: 0,
            upload: 0,
            total: 0,
        });
        entry.download += dl;
        entry.upload += ul;
        entry.total = entry.download + entry.upload;

        // 计算本月累计 = 当月所有天之和
        let month_dl: u64 = daily_map.iter()
            .filter(|(date, _)| date.starts_with(&month_str))
            .map(|(_, r)| r.download).sum();
        let month_ul: u64 = daily_map.iter()
            .filter(|(date, _)| date.starts_with(&month_str))
            .map(|(_, r)| r.upload).sum();
        s.month.download = month_dl;
        s.month.upload = month_ul;
        s.month.total = month_dl + month_ul;

        // 更新前端的每日记录列表（最近7天）
        s.daily_records = daily_map.iter().rev().take(7).map(|(_, r)| r.clone()).collect();

        // 计算月度聚合（最近3个月）
        let mut month_agg: BTreeMap<String, (u64, u64)> = BTreeMap::new();
        for (date, r) in daily_map.iter() {
            let m = date[..7].to_string();
            let (mdl, mul) = month_agg.entry(m).or_insert((0, 0));
            *mdl += r.download;
            *mul += r.upload;
        }
        s.monthly_records = month_agg.iter().rev().take(3).map(|(m, (dl, ul))| {
            let label = format!("{}年{}月", &m[..4], &m[5..].trim_start_matches('0'));
            MonthlyRecord { month: label, download: *dl, upload: *ul, total: *dl + *ul }
        }).collect();

        // 每30秒存盘
        if tick % 30 == 0 {
            let data = PersistedData { daily_records: daily_map.clone() };
            if let Ok(json) = serde_json::to_string(&data) {
                let _ = fs::write(&data_file, json);
            }
            last_save_day = today_str.clone();
        }

        // 跨天：切换时今日归零
        if today_str != last_save_day && !last_save_day.is_empty() {
            {
                s.today = TrafficStats::new();
            }
            last_save_day = today_str.clone();
        }
    }
}
