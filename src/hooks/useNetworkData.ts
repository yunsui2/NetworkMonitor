import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppState } from "../types";

const DEFAULT_STATE: AppState = {
  adapters: [],
  selected_adapter: "",
  today: { download: 0, upload: 0, total: 0 },
  month: { download: 0, upload: 0, total: 0 },
  daily_records: [],
  monthly_records: [],
  ip_address: "",
  connected: false,
};

export function useNetworkData() {
  const [state, setState] = useState<AppState>(DEFAULT_STATE);
  const [theme, setTheme] = useState<"light" | "dark">("light");

  const fetchState = useCallback(async () => {
    try {
      const data = await invoke<AppState>("get_full_state");
      setState(data);
    } catch (e) {
      console.error("Failed to fetch state:", e);
    }
  }, []);

  const selectAdapter = useCallback(
    async (name: string) => {
      try {
        await invoke("select_adapter", { name });
        await fetchState();
      } catch (e) {
        console.error("Failed to switch adapter:", e);
      }
    },
    [fetchState]
  );

  const toggleTheme = useCallback((t: "light" | "dark") => {
    setTheme(t);
    document.documentElement.setAttribute("data-theme", t);
  }, []);

  useEffect(() => {
    fetchState();
    const timer = setInterval(fetchState, 1000);
    return () => clearInterval(timer);
  }, [fetchState]);

  return { state, theme, selectAdapter, toggleTheme };
}
