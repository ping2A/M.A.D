import type { Pillar } from "../types";
import { useLocale } from "../i18n/LocaleContext";

const PILLAR_ICONS: Record<string, string> = {
  cybersecurity_dlp: "🛡️",
  dfir: "🔍",
  platform_os: "📱",
};

interface PillarCardProps {
  pillar: Pillar;
  selected?: boolean;
  onClick?: () => void;
}

export function PillarCard({ pillar, selected, onClick }: PillarCardProps) {
  const { t } = useLocale();
  const critical = pillar.requirements.filter((r) => r.severity === "critical").length;

  return (
    <button
      type="button"
      className={`pillar-card ${selected ? "selected" : ""}`}
      onClick={onClick}
    >
      <span className="pillar-icon">{PILLAR_ICONS[pillar.id] ?? "◆"}</span>
      <h3 className="pillar-name">{pillar.name}</h3>
      <p className="pillar-desc">{pillar.description}</p>
      <div className="pillar-meta">
        <span>
          {pillar.requirements.length} {t.pillarCard.requirements}
        </span>
        <span className="critical">
          {critical} {t.pillarCard.critical}
        </span>
      </div>
      <style>{`
        .pillar-card {
          display: block;
          width: 100%;
          text-align: left;
          background: white;
          border: 2px solid transparent;
          border-radius: 10px;
          padding: 1.25rem;
          cursor: pointer;
          transition: border-color 0.2s, box-shadow 0.2s, transform 0.15s;
          box-shadow: 0 2px 8px rgba(10, 22, 40, 0.08);
        }
        .pillar-card:hover {
          border-color: var(--mad-cyan-dim);
          box-shadow: 0 4px 16px rgba(0, 180, 216, 0.15);
          transform: translateY(-2px);
        }
        .pillar-card.selected {
          border-color: var(--mad-cyan);
          box-shadow: 0 4px 20px rgba(0, 180, 216, 0.25);
        }
        .pillar-icon { font-size: 1.5rem; }
        .pillar-name {
          margin: 0.5rem 0 0.25rem;
          font-size: 1rem;
          color: var(--mad-navy);
        }
        .pillar-desc {
          margin: 0;
          font-size: 0.85rem;
          color: var(--mad-text-muted);
          line-height: 1.4;
          display: -webkit-box;
          -webkit-line-clamp: 3;
          -webkit-box-orient: vertical;
          overflow: hidden;
        }
        .pillar-meta {
          display: flex;
          gap: 1rem;
          margin-top: 0.75rem;
          font-size: 0.8rem;
          font-family: var(--font-mono);
          color: var(--mad-text-muted);
        }
        .pillar-meta .critical {
          color: var(--mad-critical);
          font-weight: 600;
        }
      `}</style>
    </button>
  );
}
