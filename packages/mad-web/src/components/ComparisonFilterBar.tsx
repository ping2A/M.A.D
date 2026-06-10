import { useMemo, useState } from "react";
import type { EvaluationReport } from "../types";
import { useLocale } from "../i18n/LocaleContext";
import { collectVendorTags } from "../utils/comparisonFilter";

interface ComparisonFilterBarProps {
  evaluation: EvaluationReport;
  selectedVendorIds: Set<string>;
  activeTags: Set<string>;
  onSelectedVendorIdsChange: (ids: Set<string>) => void;
  onActiveTagsChange: (tags: Set<string>) => void;
}

export function ComparisonFilterBar({
  evaluation,
  selectedVendorIds,
  activeTags,
  onSelectedVendorIdsChange,
  onActiveTagsChange,
}: ComparisonFilterBarProps) {
  const { t, format } = useLocale();
  const allVendors = evaluation.vendors;
  const allIds = useMemo(() => allVendors.map((v) => v.vendor.id), [allVendors]);
  const knownTags = useMemo(() => collectVendorTags(evaluation), [evaluation]);
  const [expanded, setExpanded] = useState(true);

  const selectAll = () => onSelectedVendorIdsChange(new Set(allIds));
  const clearAll = () => onSelectedVendorIdsChange(new Set());
  const clearTags = () => onActiveTagsChange(new Set());

  const toggleVendor = (id: string) => {
    const next = new Set(selectedVendorIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    onSelectedVendorIdsChange(next);
  };

  const toggleTag = (tag: string) => {
    const next = new Set(activeTags);
    if (next.has(tag)) next.delete(tag);
    else next.add(tag);
    onActiveTagsChange(next);
  };

  const visibleCount = allVendors.filter(
    (v) =>
      selectedVendorIds.has(v.vendor.id) &&
      (activeTags.size === 0 || (v.vendor.tags ?? []).some((t) => activeTags.has(t))),
  ).length;

  return (
    <div className="compare-filter">
      <div className="compare-filter-header">
        <div>
          <strong>{t.compare.filterTitle}</strong>
          <span className="compare-filter-count">
            {format(t.compare.filterCount, {
              count: visibleCount,
              total: allVendors.length,
            })}
          </span>
        </div>
        <button type="button" className="filter-toggle" onClick={() => setExpanded(!expanded)}>
          {expanded ? t.common.collapse : t.common.expand}
        </button>
      </div>

      {expanded && (
        <>
          <div className="filter-section">
            <div className="filter-label-row">
              <span className="filter-label">{t.compare.filterVendors}</span>
              <span className="filter-actions">
                <button type="button" onClick={selectAll}>
                  {t.compare.selectAll}
                </button>
                <button type="button" onClick={clearAll}>
                  {t.compare.clearAll}
                </button>
              </span>
            </div>
            <div className="vendor-chips">
              {allVendors.map((v) => {
                const checked = selectedVendorIds.has(v.vendor.id);
                return (
                  <label key={v.vendor.id} className={`vendor-chip ${checked ? "on" : ""}`}>
                    <input
                      type="checkbox"
                      checked={checked}
                      onChange={() => toggleVendor(v.vendor.id)}
                    />
                    <span>{v.vendor.name}</span>
                    {(v.vendor.tags ?? []).length > 0 && (
                      <span className="chip-tags">{(v.vendor.tags ?? []).join(" · ")}</span>
                    )}
                  </label>
                );
              })}
            </div>
          </div>

          <div className="filter-section">
            <div className="filter-label-row">
              <span className="filter-label">{t.compare.filterTags}</span>
              {activeTags.size > 0 && (
                <button type="button" onClick={clearTags}>
                  {t.compare.clearTags}
                </button>
              )}
            </div>
            {knownTags.length === 0 ? (
              <p className="filter-empty">{t.compare.noTags}</p>
            ) : (
              <div className="tag-chips">
                {knownTags.map((tag) => (
                  <button
                    key={tag}
                    type="button"
                    className={`tag-chip ${activeTags.has(tag) ? "active" : ""}`}
                    onClick={() => toggleTag(tag)}
                  >
                    {tag}
                  </button>
                ))}
              </div>
            )}
            <p className="filter-hint">{t.compare.filterTagsHint}</p>
          </div>
        </>
      )}

      <style>{`
        .compare-filter {
          background: white;
          border-radius: var(--mad-radius);
          padding: 1rem 1.25rem;
          margin-bottom: 1rem;
          box-shadow: var(--mad-shadow);
          border: 1px solid var(--mad-border);
        }
        .compare-filter-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 1rem;
        }
        .compare-filter-header strong { color: var(--mad-navy); }
        .compare-filter-count {
          margin-left: 0.5rem;
          font-size: 0.8rem;
          color: var(--mad-text-muted);
          font-weight: 500;
        }
        .filter-toggle {
          background: none;
          border: none;
          color: var(--mad-cyan-dim);
          font-size: 0.8rem;
          font-weight: 600;
          cursor: pointer;
          font-family: inherit;
        }
        .filter-section { margin-top: 1rem; }
        .filter-label-row {
          display: flex;
          align-items: center;
          justify-content: space-between;
          margin-bottom: 0.5rem;
        }
        .filter-label {
          font-size: 0.78rem;
          font-weight: 700;
          text-transform: uppercase;
          letter-spacing: 0.05em;
          color: var(--mad-text-muted);
        }
        .filter-actions { display: flex; gap: 0.5rem; }
        .filter-actions button, .filter-label-row > button {
          background: none;
          border: none;
          color: var(--mad-cyan-dim);
          font-size: 0.78rem;
          font-weight: 600;
          cursor: pointer;
          font-family: inherit;
        }
        .vendor-chips { display: flex; flex-wrap: wrap; gap: 0.45rem; }
        .vendor-chip {
          display: inline-flex;
          align-items: center;
          gap: 0.35rem;
          padding: 0.35rem 0.65rem;
          border-radius: 999px;
          border: 1px solid var(--mad-border);
          background: var(--mad-bg);
          font-size: 0.8rem;
          cursor: pointer;
          user-select: none;
        }
        .vendor-chip.on {
          border-color: var(--mad-cyan);
          background: rgba(0, 180, 216, 0.1);
        }
        .vendor-chip input { accent-color: var(--mad-cyan); }
        .chip-tags {
          font-size: 0.68rem;
          color: var(--mad-text-muted);
          max-width: 12ch;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }
        .tag-chips { display: flex; flex-wrap: wrap; gap: 0.4rem; }
        .tag-chip {
          padding: 0.3rem 0.7rem;
          border-radius: 999px;
          border: 1px solid var(--mad-border);
          background: white;
          font-size: 0.78rem;
          font-weight: 600;
          color: var(--mad-navy);
          cursor: pointer;
          font-family: inherit;
        }
        .tag-chip.active {
          background: var(--mad-navy);
          color: white;
          border-color: var(--mad-navy);
        }
        .filter-empty, .filter-hint {
          margin: 0.35rem 0 0;
          font-size: 0.78rem;
          color: var(--mad-text-muted);
        }
      `}</style>
    </div>
  );
}
