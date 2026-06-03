import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppState, AdapterInfo } from "../types";

const DEFAULT_STATE: AppState = {
  adapters: [],
  selected_adapter: "",
  speed: { download_bytes_per_sec: 0, upload_bytes_per_sec: 0 },
  today: { download: 0, upload: 0, total: 0 },
  month: { download: 0, upload: 0, total: 0 },
  status: { connected: false, ip_address: "", adapter_name: "" },
};

export function useNetworkData() {
  const [state, setState] = useState<AppState>(DEFAULT_STATE);
  const [theme, setTheme] = useState<"light" | "dark">("light");

  // 从后端拉取全量状态
  const fetchState = useCallback(async () => {
    try {
      const data = await invoke<AppState>("get_full_state");
      setState(data);
    } catch (e) {
      console.error("Failed to fetch state:", e);
    }
  }, []);

  // 切换网卡
  const selectAdapter = useCallback(async (name: string) => {
    try {
      await invoke("select_adapter", { name });
      await fetchState();
    } catch (e) {
      console.error("Failed to switch adapter:", e);
    }
  }, [fetchState]);

  // 切换主题
  const toggleTheme = useCallback((t: "light" | "dark") => {
    setTheme(t);
    document.documentElement.setAttribute("data-theme", t);
  }, []);

  // 定时刷新 (1秒)
  useEffect(() => {
    fetchState();
    const timer = setInterval(fetchState, 1000);
    return () => clearInterval(timer);
  }, [fetchState]);

  return {
    state,
    theme,
    selectAdapter,
    toggleTheme,
  };
}
