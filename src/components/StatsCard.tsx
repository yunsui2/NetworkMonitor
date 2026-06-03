import { formatBytes } from "../utils/format";

interface StatsCardProps {
  title: string;
  download: number;
  upload: number;
  total: number;
}

export default function StatsCard({ title, download, upload, total }: StatsCardProps) {
  return (
    <div className="stat-card">
      <div className="stat-card-header">{title}</div>
      <div className="stat-main-row">
        <div className="stat-block">
          <div className="stat-block-label">下载</div>
          <div className="stat-block-value dl">{formatBytes(download)}</div>
        </div>
        <div className="stat-block">
          <div className="stat-block-label">上传</div>
          <div className="stat-block-value ul">{formatBytes(upload)}</div>
        </div>
      </div>
      <div className="stat-divider" />
      <div className="stat-total-row">
        <span className="stat-total-label">总计</span>
        <span className="stat-total-value">{formatBytes(total)}</span>
      </div>
    </div>
  );
}
