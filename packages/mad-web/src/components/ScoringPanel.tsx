import { useEffect, useState } from "react";
import { useLocale } from "../i18n/LocaleContext";
import type { ScoringConfig } from "../types";

interface ScoringPanelProps {
  scoring: ScoringConfig;
  onChange: (scoring: ScoringConfig) => Promise<void>;
  collapsed?: boolean;
}

export function ScoringPanel({ scoring, onChange, collapsed: initialCollapsed }: ScoringPanelProps) {
  const { t, format } = useLocale();
  const [collapsed, setCollapsed] = useState(initialCollapsed ?? false);
  const [local, setLocal] = useState(scoring);
  const [dirty, setDirty] = useState(false);

  useEffect(() => {
    if (!dirty) setLocal(scoring);
  }, [scoring, dirty]);

  const updateLocal = (patch: Partial<ScoringConfig>) => {
    setLocal((prev) => ({ ...prev, ...patch }));
    setDirty(true);
  };

  const apply = async () => {
    await onChange(local);
    setDirty(false);
  };

  const reset = () => {
    setLocal(scoring);
    setDirty(false);
  };

  return (
    <div className={`scoring-panel ${collapsed ? "collapsed" : ""}`}>
      <div className="scoring-header">
        <h3>{t.scoring.title}</h3>
        <button
          type="button"
          className="toggle-btn"
          onClick={() => setCollapsed(!collapsed)}
          aria-expanded={!collapsed}
        >
          {collapsed ? t.common.expand : t.common.collapse}
        </button>
      </div>

      {!collapsed && (
        <div className="scoring-body">
          <label className="toggle-row">
            <input
              type="checkbox"
              checked={local.use_severity_weighting}
              onChange={(e) => updateLocal({ use_severity_weighting: e.target.checked })}
            />
            <span>
              {format(t.scoring.severityWeighting, {
                critical: local.critical_weight,
                high: local.high_weight,
                medium: local.medium_weight,
              })}
            </span>
          </label>

          <div className="scoring-grid">
            <fieldset>
              <legend>{t.scoring.statusPoints}</legend>
              <label>
                {t.scoring.compliant}
                <input
                  type="number"
                  step="0.1"
                  min="0"
                  value={local.compliant_points}
                  onChange={(e) => updateLocal({ compliant_points: +e.target.value })}
                />
              </label>
              <label>
                {t.scoring.partial}
                <input
                  type="number"
                  step="0.1"
                  min="0"
                  value={local.partial_points}
                  onChange={(e) => updateLocal({ partial_points: +e.target.value })}
                />
              </label>
              <label>
                {t.scoring.nonCompliant}
                <input
                  type="number"
                  step="0.1"
                  min="0"
                  value={local.non_compliant_points}
                  onChange={(e) => updateLocal({ non_compliant_points: +e.target.value })}
                />
              </label>
              <label>
                {t.scoring.untested}
                <input
                  type="number"
                  step="0.1"
                  min="0"
                  value={local.untested_points}
                  onChange={(e) => updateLocal({ untested_points: +e.target.value })}
                />
              </label>
            </fieldset>

            <fieldset>
              <legend>{t.scoring.severityWeights}</legend>
              <label>
                {t.scoring.critical}
                <input
                  type="number"
                  step="0.1"
                  min="0"
                  value={local.critical_weight}
                  onChange={(e) => updateLocal({ critical_weight: +e.target.value })}
                  disabled={!local.use_severity_weighting}
                />
              </label>
              <label>
                {t.scoring.high}
                <input
                  type="number"
                  step="0.1"
                  min="0"
                  value={local.high_weight}
                  onChange={(e) => updateLocal({ high_weight: +e.target.value })}
                  disabled={!local.use_severity_weighting}
                />
              </label>
              <label>
                {t.scoring.medium}
                <input
                  type="number"
                  step="0.1"
                  min="0"
                  value={local.medium_weight}
                  onChange={(e) => updateLocal({ medium_weight: +e.target.value })}
                  disabled={!local.use_severity_weighting}
                />
              </label>
            </fieldset>
          </div>

          {dirty && (
            <div className="scoring-actions">
              <button type="button" className="btn btn-primary" onClick={apply}>
                {t.scoring.apply}
              </button>
              <button type="button" className="btn btn-secondary" onClick={reset}>
                {t.common.reset}
              </button>
            </div>
          )}
        </div>
      )}

      <style>{`
        .scoring-panel {
          background: var(--mad-navy-light);
          color: white;
          border-radius: var(--mad-radius);
          padding: 1rem 1.25rem;
          margin-bottom: 1.25rem;
          border: 1px solid rgba(0, 180, 216, 0.25);
        }
        .scoring-panel.collapsed { padding-bottom: 0.85rem; }
        .scoring-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
        }
        .scoring-panel h3 {
          margin: 0;
          font-size: 0.9rem;
          color: var(--mad-cyan);
          text-transform: uppercase;
          letter-spacing: 0.06em;
        }
        .toggle-btn {
          background: rgba(255,255,255,0.1);
          border: 1px solid rgba(255,255,255,0.2);
          color: var(--mad-silver);
          padding: 0.25rem 0.6rem;
          border-radius: 4px;
          font-size: 0.75rem;
          cursor: pointer;
        }
        .toggle-btn:hover { background: rgba(255,255,255,0.15); }
        .scoring-body { margin-top: 0.85rem; }
        .toggle-row {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          cursor: pointer;
          font-size: 0.85rem;
        }
        .scoring-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 1rem;
          margin-top: 1rem;
        }
        fieldset {
          border: 1px solid rgba(255,255,255,0.15);
          border-radius: 8px;
          padding: 0.75rem;
          margin: 0;
        }
        legend {
          font-size: 0.72rem;
          text-transform: uppercase;
          letter-spacing: 0.05em;
          color: var(--mad-silver);
          padding: 0 0.25rem;
        }
        fieldset label {
          display: flex;
          justify-content: space-between;
          align-items: center;
          gap: 0.5rem;
          font-size: 0.8rem;
          margin-bottom: 0.4rem;
        }
        fieldset input[type="number"] {
          width: 4rem;
          padding: 0.3rem 0.4rem;
          border: 1px solid rgba(255,255,255,0.2);
          border-radius: 4px;
          background: rgba(0,0,0,0.2);
          color: white;
          font-family: var(--font-mono);
          font-size: 0.85rem;
        }
        fieldset input:disabled { opacity: 0.4; }
        .scoring-actions {
          display: flex;
          gap: 0.5rem;
          margin-top: 1rem;
        }
      `}</style>
    </div>
  );
}
