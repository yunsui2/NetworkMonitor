import type { NetworkStatus } from "../types";

interface FooterStatusProps {
  status: NetworkStatus;
}

export default function FooterStatus({ status }: FooterStatusProps) {
  return (
    <div className="footer-status">
      <span className={"footer-dot " + (status.connected ? "on" : "off")} />
      <span>{status.connected ? "已连接" : "未连接"}</span>
      <span className="sep" />
      <span>{status.connected ? status.ip_address : "—"}</span>
      <span className="sep" />
      <span>{status.adapter_name}</span>
    </div>
  );
}
