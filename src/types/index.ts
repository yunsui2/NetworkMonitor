/** 网卡信息 */
export interface AdapterInfo {
  name: string;
  alias: string;
  ip_address: string;
  is_default: boolean;
}

/** 网速数据 */
export interface SpeedData {
  download_bytes_per_sec: number;
  upload_bytes_per_sec: number;
}

/** 流量统计数据 */
export interface TrafficStats {
  download: number; // 字节
  upload: number;
  total: number;
}

/** 网络状态 */
export interface NetworkStatus {
  connected: boolean;
  ip_address: string;
  adapter_name: string;
}

/** Tauri 后端暴露的完整状态 */
export interface AppState {
  adapters: AdapterInfo[];
  selected_adapter: string;
  speed: SpeedData;
  today: TrafficStats;
  month: TrafficStats;
  status: NetworkStatus;
}
