import { useLocale } from "../i18n/LocaleContext";

interface StatsBarProps {
  version: string;
  pillars: number;
  requirements: number;
  critical: number;
  vendors: number;
}

export function StatsBar({ version, pillars, requirements, critical, vendors }: StatsBarProps) {
  const { t } = useLocale();

  return (
    <div className="stats-bar">
      <div className="stat">
        <span className="stat-value">{version}</span>
        <span className="stat-label">{t.stats.policyVersion}</span>
      </div>
      <div className="stat">
        <span className="stat-value">{pillars}</span>
        <span className="stat-label">{t.common.pillars}</span>
      </div>
      <div className="stat">
        <span className="stat-value">{requirements}</span>
        <span className="stat-label">{t.stats.requirements}</span>
      </div>
      <div className="stat">
        <span className="stat-value critical">{critical}</span>
        <span className="stat-label">{t.stats.critical}</span>
      </div>
      <div className="stat">
        <span className="stat-value">{vendors}</span>
        <span className="stat-label">{t.stats.vendors}</span>
      </div>
      <style>{`
        .stats-bar {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
          gap: 1rem;
          background: var(--mad-navy-light);
          border-radius: 10px;
          padding: 1.25rem;
          color: white;
        }
        .stat { text-align: center; }
        .stat-value {
          display: block;
          font-size: 1.5rem;
          font-weight: 700;
          font-family: var(--font-mono);
          color: var(--mad-cyan);
        }
        .stat-value.critical { color: #ff6b6b; }
        .stat-label {
          display: block;
          font-size: 0.75rem;
          text-transform: uppercase;
          letter-spacing: 0.05em;
          color: var(--mad-silver);
          margin-top: 0.25rem;
        }
      `}</style>
    </div>
  );
}
