import type { EvaluationReport } from "../types";
import { useLocale } from "../i18n/LocaleContext";
import { formatMoney } from "../utils/pricing";
import { rankScore, rankVendors, usesCompositeRanking } from "../utils/ranking";
import { scoreColor } from "../utils/scoring";

interface ScoreOverviewProps {
  evaluation: EvaluationReport;
  onSelectVendor?: (vendorId: string) => void;
}

export function ScoreOverview({ evaluation, onSelectVendor }: ScoreOverviewProps) {
  const { t, format } = useLocale();
  const ranked = rankVendors(evaluation);
  const compositeMode = usesCompositeRanking(evaluation);

  if (ranked.length === 0) return null;

  return (
    <div className="score-overview">
      <div className="score-overview-header">
        <span className="score-overview-label">{t.scoreOverview.label}</span>
        <span className="score-overview-hint">{t.scoreOverview.hint}</span>
      </div>
      <div className="score-chips">
        {ranked.map((r, i) => {
          const os = r.overall_score;
          const pct = rankScore(r, evaluation);
          const gaps = os.critical_gaps.length;
          const gapsLabel = gaps
            ? ` · ${gaps} ${gaps > 1 ? t.scoreOverview.gaps : t.scoreOverview.gap}`
            : "";
          return (
            <button
              key={r.vendor.id}
              type="button"
              className="score-chip"
              onClick={() => onSelectVendor?.(r.vendor.id)}
              title={format(t.scoreOverview.vendorTitle, {
                name: r.vendor.name,
                score: pct.toFixed(1),
                gaps: gapsLabel,
              })}
            >
              <span className="chip-rank">#{i + 1}</span>
              <span className="chip-name">{r.vendor.name}</span>
              <span className="chip-score" style={{ color: scoreColor(pct) }}>
                {pct.toFixed(0)}%
                {compositeMode && os.composite_score_percent != null ? "★" : ""}
              </span>
              {os.annual_cost_per_device != null && (
                <span className="chip-cost">
                  {formatMoney(os.annual_cost_per_device, os.price_currency ?? "USD", true)}
                  /yr
                </span>
              )}
              {gaps > 0 && (
                <span className="chip-gaps">
                  {gaps} {gaps > 1 ? t.scoreOverview.gaps : t.scoreOverview.gap}
                </span>
              )}
            </button>
          );
        })}
      </div>

      <style>{`
        .score-overview {
          background: linear-gradient(135deg, var(--mad-navy) 0%, var(--mad-blue) 100%);
          border-radius: var(--mad-radius);
          padding: 1rem 1.25rem;
          margin-bottom: 1.25rem;
          border: 1px solid rgba(0, 180, 216, 0.35);
        }
        .score-overview-header {
          display: flex;
          align-items: baseline;
          gap: 0.75rem;
          margin-bottom: 0.75rem;
          flex-wrap: wrap;
        }
        .score-overview-label {
          font-size: 0.75rem;
          font-weight: 700;
          text-transform: uppercase;
          letter-spacing: 0.08em;
          color: var(--mad-cyan);
        }
        .score-overview-hint {
          font-size: 0.78rem;
          color: var(--mad-silver);
          opacity: 0.85;
        }
        .score-chips { display: flex; gap: 0.6rem; flex-wrap: wrap; }
        .score-chip {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          background: rgba(255, 255, 255, 0.08);
          border: 1px solid rgba(255, 255, 255, 0.15);
          border-radius: 8px;
          padding: 0.5rem 0.85rem;
          color: white;
          cursor: pointer;
          font-family: inherit;
          transition: background 0.15s, border-color 0.15s;
        }
        .score-chip:hover {
          background: rgba(255, 255, 255, 0.14);
          border-color: var(--mad-cyan);
        }
        .chip-rank { font-size: 0.7rem; font-weight: 800; opacity: 0.45; }
        .chip-name { font-weight: 600; font-size: 0.88rem; }
        .chip-score {
          font-family: var(--font-mono);
          font-weight: 800;
          font-size: 1rem;
        }
        .chip-cost { font-size: 0.68rem; color: var(--mad-silver); font-weight: 600; }
        .chip-gaps { font-size: 0.68rem; color: #ff8a94; font-weight: 600; }
      `}</style>
    </div>
  );
}
