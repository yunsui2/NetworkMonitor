use super::AdapterInfo;
use windows::Win32::NetworkManagement::IpHelper::{
    GetIfTable2, FreeMibTable, MIB_IF_TABLE2,
};

/// 获取物理网络适配器列表（仅以太网和 WiFi）
pub fn get_adapters() -> Vec<AdapterInfo> {
    let mut adapters = Vec::new();

    unsafe {
        let mut table_ptr: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
        if GetIfTable2(&mut table_ptr).is_err() || table_ptr.is_null() {
            return adapters;
        }

        let table = &*table_ptr;
        let num = table.NumEntries as usize;
        let first_row_ptr: *const _ = &table.Table[0];

        for i in 0..num {
            let row = &*first_row_ptr.add(i);

            let alias = read_u16_slice(&row.Alias);
            let desc = read_u16_slice(&row.Description);
            let if_type = row.Type;

            // 1. 只要物理网卡类型
            if if_type != 6 && if_type != 71 {
                continue;
            }

            // 2. 只要活跃接口 (OperStatus == 1)
            if row.OperStatus.0 != 1 {
                continue;
            }

            // 3. 过滤 NDIS 过滤器、虚拟接口、VPN 等
            let combined = format!("{} {}", alias.to_lowercase(), desc.to_lowercase());
            let blacklist = [
                "wfp", "lightweight", "filter", "qos", "scheduler",
                "wan miniport", "virtual", "vpn", "sangfor", "tunnel",
                "meta", "clash", "mihomo", "flclash", "bluetooth",
                "kernel debug", "teredo", "6to4", "ip-https",
                "wi-fi direct", "wifi direct", "miniport",
                "loopback", "pseudo",
            ];
            let blocked = blacklist.iter().any(|kw| combined.contains(kw));
            if blocked {
                continue;
            }

            if alias.is_empty() {
                continue;
            }

            let idx = row.InterfaceIndex;
            let display = if !desc.is_empty() { desc } else { alias.clone() };
            let ip = get_local_ip();

            adapters.push(AdapterInfo {
                name: format!("{}", idx),       // InterfaceIndex 作为唯一 key
                alias: display,                  // 显示用描述名称
                ip_address: ip,
                is_default: false,
            });
        }

        let _ = FreeMibTable(table_ptr as *mut _);
    }

    // 默认选第一个有 IP 的（活跃的物理网卡）
    if let Some(first) = adapters.iter_mut().find(|a| !a.ip_address.is_empty()) {
        first.is_default = true;
    } else if let Some(first) = adapters.first_mut() {
        first.is_default = true;
    }

    adapters
}

fn read_u16_slice(arr: &[u16]) -> String {
    let len = arr.iter().position(|&c| c == 0).unwrap_or(arr.len());
    if len == 0 {
        return String::new();
    }
    String::from_utf16_lossy(&arr[..len])
}

fn get_local_ip() -> String {
    use std::net::UdpSocket;
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    String::new()
}
