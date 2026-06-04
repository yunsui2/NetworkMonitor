import { useState, useCallback } from "react";
import { useNetworkData } from "./hooks/useNetworkData";
import { formatBytes } from "./utils/format";
import type { MonthlyRecord } from "./types";
import "./App.css";

export default function App() {
  const { state, theme, toggleTheme, selectAdapter } = useNetworkData();
  const [activeTab, setActiveTab] = useState<"overview" | "monthly">("overview");
  const [flipped, setFlipped] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);

  const onFlip = useCallback(() => setFlipped((v) => !v), []);

  // 本月标签文字
  const now = new Date();
  const monthLabel = `${now.getFullYear()}年${now.getMonth() + 1}月 · 本月累计`;

  // 月度列表（最近3个月）
  const recentMonths: MonthlyRecord[] = state.monthly_records.slice(0, 3);
  const maxMonthTotal = Math.max(...recentMonths.map((m) => m.total), 1);

  return (
    <div className="app-container">
      {/* 标题栏 */}
      <div className="titlebar">
        <div className="titlebar-left">
          <span className="app-name">Flux</span>
          <div className="tabs">
            <button
              className={"tab-btn" + (activeTab === "overview" ? " active" : "")}
              onClick={() => setActiveTab("overview")}
            >
              概览
            </button>
            <button
              className={"tab-btn" + (activeTab === "monthly" ? " active" : "")}
              onClick={() => setActiveTab("monthly")}
            >
              历史
            </button>
          </div>
        </div>
        <div className="titlebar-right">
          <div className="theme-switch">
            <button
              className={theme === "light" ? "active" : ""}
              onClick={() => toggleTheme("light")}
            >
              浅色
            </button>
            <button
              className={theme === "dark" ? "active" : ""}
              onClick={() => toggleTheme("dark")}
            >
              深色
            </button>
          </div>
          <button
            className="settings-btn"
            onClick={() => setSettingsOpen(!settingsOpen)}
            title="设置"
          >
            ⚙
          </button>
        </div>
      </div>

      {/* 设置面板 */}
      {settingsOpen && (
        <div className="settings-overlay show" onClick={() => setSettingsOpen(false)}>
          <div className="settings-panel" onClick={(e) => e.stopPropagation()}>
            <h4>设置</h4>
            <select
              value={state.selected_adapter}
              onChange={(e) => selectAdapter(e.target.value)}
            >
              {state.adapters.map((a) => (
                <option key={a.name} value={a.name}>
                  {a.alias}
                </option>
              ))}
            </select>
            <div className="info-row">
              <span>IP 地址</span>
              <span>{state.connected ? state.ip_address : "—"}</span>
            </div>
            <div className="info-row">
              <span>状态</span>
              <span>
                <span className={"dot " + (state.connected ? "on" : "off")} />
                {state.connected ? "已连接" : "未连接"}
              </span>
            </div>
          </div>
        </div>
      )}

      {/* 概览 Tab */}
      {activeTab === "overview" && (
        <div className="main">
          <div className="flip-container" onClick={onFlip}>
            <div className={"flip-inner" + (flipped ? " flipped" : "")}>
              {/* 正面：本月累计 */}
              <div className="flip-front">
                <div className="hero-label">{monthLabel}</div>
                <div className="hero-total">{formatBytes(state.month.total)}</div>
                <div className="hero-sub">
                  <span>▼ 下载 {formatBytes(state.month.download)}</span>
                  <span>▲ 上传 {formatBytes(state.month.upload)}</span>
                </div>
                <span className="hero-hint">点击查看今日</span>
              </div>
              {/* 背面：今日流量 */}
              <div className="flip-back">
                <div className="back-title">今日流量</div>
                <div className="back-row">
                  <span className="label">下载</span>
                  <span className="dl">{formatBytes(state.today.download)}</span>
                </div>
                <div className="back-row">
                  <span className="label">上传</span>
                  <span className="ul">{formatBytes(state.today.upload)}</span>
                </div>
                <div className="back-divider" />
                <div className="back-total">
                  <span>总计</span>
                  <span style={{ fontFamily: "var(--mono)" }}>
                    {formatBytes(state.today.total)}
                  </span>
                </div>
              </div>
            </div>
          </div>
          <div className="card-hint">点击卡片 本月 ⇄ 今日</div>
        </div>
      )}

      {/* 月度 Tab */}
      {activeTab === "monthly" && (
        <div className="main">
          <div className="month-list">
            {recentMonths.map((m) => (
              <div className="month-item" key={m.month}>
                <div className="month-item-header">
                  <span>{m.month}</span>
                  <span>{formatBytes(m.total)}</span>
                </div>
                <div className="month-bar-bg">
                  <div
                    className="month-bar-fill"
                    style={{ width: `${(m.total / maxMonthTotal) * 100}%` }}
                  />
                </div>
                <div className="month-item-sub">
                  <span>▼ 下载 {formatBytes(m.download)}</span>
                  <span>▲ 上传 {formatBytes(m.upload)}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
