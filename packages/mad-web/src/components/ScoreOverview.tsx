import { useMemo } from "react";
import type { EvaluationReport } from "../types";
import { useLocale } from "../i18n/LocaleContext";
import { buildFilteredEvaluation, collectVendorTags } from "../utils/comparisonFilter";
import { formatMoney } from "../utils/pricing";
import { rankScore, rankVendors, usesCompositeRanking } from "../utils/ranking";
import { scoreColor } from "../utils/scoring";

interface ScoreOverviewProps {
  evaluation: EvaluationReport;
  activeTags: Set<string>;
  onActiveTagsChange: (tags: Set<string>) => void;
  onSelectVendor?: (vendorId: string) => void;
}

export function ScoreOverview({
  evaluation,
  activeTags,
  onActiveTagsChange,
  onSelectVendor,
}: ScoreOverviewProps) {
  const { t, format } = useLocale();
  const knownTags = useMemo(() => collectVendorTags(evaluation), [evaluation]);
  const allVendorIds = useMemo(
    () => new Set(evaluation.vendors.map((v) => v.vendor.id)),
    [evaluation.vendors],
  );
  const filteredEvaluation = useMemo(
    () => buildFilteredEvaluation(evaluation, allVendorIds, activeTags),
    [evaluation, allVendorIds, activeTags],
  );
  const ranked = useMemo(() => rankVendors(filteredEvaluation), [filteredEvaluation]);
  const compositeMode = usesCompositeRanking(filteredEvaluation);

  const toggleTag = (tag: string) => {
    const next = new Set(activeTags);
    if (next.has(tag)) next.delete(tag);
    else next.add(tag);
    onActiveTagsChange(next);
  };

  if (evaluation.vendors.length === 0) return null;

  return (
    <div className="score-overview">
      <div className="score-overview-header">
        <div className="score-overview-title">
          <span className="score-overview-label">{t.scoreOverview.label}</span>
          <span className="score-overview-hint">{t.scoreOverview.hint}</span>
        </div>
        {activeTags.size > 0 && (
          <span className="score-overview-count">
            {format(t.scoreOverview.filterCount, {
              count: ranked.length,
              total: evaluation.vendors.length,
            })}
          </span>
        )}
      </div>

      {knownTags.length > 0 && (
        <div className="score-tag-filter" role="group" aria-label={t.scoreOverview.filterTags}>
          <div className="score-tag-filter-row">
            <span className="score-tag-filter-label">{t.scoreOverview.filterTags}</span>
            {activeTags.size > 0 && (
              <button type="button" className="score-tag-clear" onClick={() => onActiveTagsChange(new Set())}>
                {t.scoreOverview.clearTags}
              </button>
            )}
          </div>
          <div className="score-tag-chips">
            {knownTags.map((tag) => (
              <button
                key={tag}
                type="button"
                className={`score-tag-chip ${activeTags.has(tag) ? "active" : ""}`}
                onClick={() => toggleTag(tag)}
                aria-pressed={activeTags.has(tag)}
              >
                {tag}
              </button>
            ))}
          </div>
          <p className="score-tag-hint">{t.scoreOverview.filterTagsHint}</p>
        </div>
      )}

      {knownTags.length === 0 && (
        <p className="score-tag-empty">{t.scoreOverview.noTags}</p>
      )}

      {ranked.length === 0 ? (
        <p className="score-filter-empty">{t.scoreOverview.noVendorsFiltered}</p>
      ) : (
        <div className="score-chips">
          {ranked.map((r, i) => {
            const os = r.overall_score;
            const pct = rankScore(r, filteredEvaluation);
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
      )}

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
          align-items: flex-start;
          justify-content: space-between;
          gap: 0.75rem;
          margin-bottom: 0.75rem;
          flex-wrap: wrap;
        }
        .score-overview-title {
          display: flex;
          align-items: baseline;
          gap: 0.75rem;
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
        .score-overview-count {
          font-size: 0.75rem;
          font-weight: 600;
          color: var(--mad-silver);
        }
        .score-tag-filter { margin-bottom: 0.85rem; }
        .score-tag-filter-row {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 0.5rem;
          margin-bottom: 0.45rem;
        }
        .score-tag-filter-label {
          font-size: 0.72rem;
          font-weight: 700;
          text-transform: uppercase;
          letter-spacing: 0.05em;
          color: rgba(255, 255, 255, 0.65);
        }
        .score-tag-clear {
          background: none;
          border: none;
          color: var(--mad-cyan);
          font-size: 0.75rem;
          font-weight: 600;
          cursor: pointer;
          font-family: inherit;
          padding: 0;
        }
        .score-tag-chips { display: flex; flex-wrap: wrap; gap: 0.4rem; }
        .score-tag-chip {
          padding: 0.28rem 0.65rem;
          border-radius: 999px;
          border: 1px solid rgba(255, 255, 255, 0.25);
          background: rgba(255, 255, 255, 0.06);
          font-size: 0.76rem;
          font-weight: 600;
          color: white;
          cursor: pointer;
          font-family: inherit;
          transition: background 0.15s, border-color 0.15s;
        }
        .score-tag-chip:hover {
          background: rgba(255, 255, 255, 0.12);
          border-color: var(--mad-cyan);
        }
        .score-tag-chip.active {
          background: var(--mad-cyan);
          color: var(--mad-navy);
          border-color: var(--mad-cyan);
        }
        .score-tag-hint, .score-tag-empty, .score-filter-empty {
          margin: 0.4rem 0 0;
          font-size: 0.72rem;
          color: rgba(255, 255, 255, 0.55);
          line-height: 1.4;
        }
        .score-filter-empty { color: #ff8a94; font-weight: 600; }
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
