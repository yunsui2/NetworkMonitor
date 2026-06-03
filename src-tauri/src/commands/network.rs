use crate::collector::{AdapterInfo, AppState};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn get_full_state(state: State<'_, Arc<Mutex<AppState>>>) -> Result<AppState, String> {
    Ok(state.lock().clone())
}

#[tauri::command]
pub fn get_adapters(state: State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<AdapterInfo>, String> {
    Ok(state.lock().adapters.clone())
}

#[tauri::command]
pub fn select_adapter(name: String, state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut s = state.lock();
    if !s.adapters.iter().any(|a| a.name == name) {
        return Err(format!("Adapter '{}' not found", name));
    }
    s.selected_adapter = name.clone();
    // 先取出需要的值
    let (ip, conn) = if let Some(a) = s.adapters.iter().find(|a| a.name == name) {
        (a.ip_address.clone(), !a.ip_address.is_empty())
    } else {
        (String::new(), false)
    };
    s.ip_address = ip;
    s.connected = conn;
    Ok(())
}
