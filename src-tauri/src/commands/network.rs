use crate::collector::{AdapterInfo, AppState};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::State;

/// 获取完整应用状态（供前端 1 秒轮询）
#[tauri::command]
pub fn get_full_state(state: State<'_, Arc<Mutex<AppState>>>) -> Result<AppState, String> {
    let s = state.lock();
    Ok(s.clone())
}

/// 获取网卡列表
#[tauri::command]
pub fn get_adapters(state: State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<AdapterInfo>, String> {
    let s = state.lock();
    Ok(s.adapters.clone())
}

/// 切换监控网卡
#[tauri::command]
pub fn select_adapter(name: String, state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut s = state.lock();

    // 验证网卡存在
    if !s.adapters.iter().any(|a| a.name == name) {
        return Err(format!("Adapter '{}' not found", name));
    }

    s.selected_adapter = name;

    // 更新状态中的网卡名
    if let Some(adapter) = s.adapters.iter().find(|a| a.name == s.selected_adapter) {
        s.status.adapter_name = adapter.alias.clone();
    }

    Ok(())
}
