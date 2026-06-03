import { useState } from "react";
import { useNetworkData } from "./hooks/useNetworkData";
import StatsCard from "./components/StatsCard";
import FooterStatus from "./components/FooterStatus";
import "./App.css";

export default function App() {
  const { state, theme, selectAdapter, toggleTheme } = useNetworkData();
  const [pinActive, setPinActive] = useState(true);

  return (
    <div className="app-container">
      {/* 标题栏 */}
      <div className="title-bar">
        <span className="app-title">Network Monitor</span>
        <div className="title-bar-right">
          {/* 主题切换 */}
          <div className="theme-toggle">
            <button
              className={"theme-option" + (theme === "light" ? " active" : "")}
              onClick={() => toggleTheme("light")}
            >
              浅色
            </button>
            <button
              className={"theme-option" + (theme === "dark" ? " active" : "")}
              onClick={() => toggleTheme("dark")}
            >
              深色
            </button>
          </div>

          {/* 网卡选择 */}
          <select
            className="adapter-select"
            value={state.selected_adapter}
            onChange={(e) => selectAdapter(e.target.value)}
          >
            {state.adapters.map((adapter) => (
              <option key={adapter.name} value={adapter.name}>
                {adapter.alias || adapter.name}
              </option>
            ))}
          </select>

          {/* 关闭 */}
          <button className="icon-btn" title="关闭">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>

          {/* 最小化 */}
          <button className="icon-btn" title="最小化到托盘">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
          </button>

          {/* 置顶 */}
          <button
            className={"icon-btn" + (pinActive ? " active" : "")}
            onClick={() => setPinActive(!pinActive)}
            title="窗口置顶"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2">
              <line x1="12" y1="5" x2="12" y2="19" />
              <polyline points="8 9 12 5 16 9" />
            </svg>
          </button>
        </div>
      </div>

      {/* 主内容 */}
      <div className="main-content">
        <div className="stats-grid">
          <StatsCard
            title="今日流量"
            download={state.today.download}
            upload={state.today.upload}
            total={state.today.total}
          />
          <StatsCard
            title="本月流量"
            download={state.month.download}
            upload={state.month.upload}
            total={state.month.total}
          />
        </div>

        {/* 底部网络状态 */}
        <FooterStatus status={state.status} />
      </div>
    </div>
  );
}
