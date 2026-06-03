/** 网卡信息 */
export interface AdapterInfo {
  name: string;
  alias: string;
  ip_address: string;
  is_default: boolean;
}

/** 流量统计 */
export interface TrafficStats {
  download: number;
  upload: number;
  total: number;
}

/** 每日记录 */
export interface DailyRecord {
  date: string;
  download: number;
  upload: number;
  total: number;
}

/** 月度记录 */
export interface MonthlyRecord {
  month: string;
  download: number;
  upload: number;
  total: number;
}

/** Tauri 后端暴露的完整状态 */
export interface AppState {
  adapters: AdapterInfo[];
  selected_adapter: string;
  today: TrafficStats;
  month: TrafficStats;
  daily_records: DailyRecord[];
  monthly_records: MonthlyRecord[];
  ip_address: string;
  connected: boolean;
}
