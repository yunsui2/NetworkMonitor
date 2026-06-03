use super::AppState;
use windows::Win32::NetworkManagement::IpHelper::{GetIfTable2, FreeMibTable, MIB_IF_TABLE2};

/// 获取当前网卡速度，使用 AppState 中保存的上次计数器计算差值
/// 返回 (download_bytes_per_sec, upload_bytes_per_sec)
pub fn get_adapter_speed(state: &mut AppState) -> Option<(u64, u64)> {
    let adapter_name = state.selected_adapter.clone();
    if adapter_name.is_empty() {
        return Some((0, 0));
    }

    unsafe {
        let mut table_ptr: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
        if GetIfTable2(&mut table_ptr).is_err() || table_ptr.is_null() {
            return None;
        }

        let table = &*table_ptr;
        let num = table.NumEntries as usize;
        let first_row_ptr: *const _ = &table.Table[0];

        for i in 0..num {
            let row = &*first_row_ptr.add(i);
            let idx = format!("{}", row.InterfaceIndex);

            if idx != adapter_name {
                continue;
            }

            let cur_in = row.InOctets;
            let cur_out = row.OutOctets;

            // 首次采集：记录基线
            if state.last_in_bytes == 0 && state.last_out_bytes == 0 {
                state.last_in_bytes = cur_in;
                state.last_out_bytes = cur_out;
                let _ = FreeMibTable(table_ptr as *mut _);
                return Some((0, 0));
            }

            // 计算差值
            let dl = if cur_in >= state.last_in_bytes {
                cur_in - state.last_in_bytes
            } else {
                0
            };
            let ul = if cur_out >= state.last_out_bytes {
                cur_out - state.last_out_bytes
            } else {
                0
            };

            state.last_in_bytes = cur_in;
            state.last_out_bytes = cur_out;

            let _ = FreeMibTable(table_ptr as *mut _);
            return Some((dl, ul));
        }

        let _ = FreeMibTable(table_ptr as *mut _);
    }

    Some((0, 0))
}
