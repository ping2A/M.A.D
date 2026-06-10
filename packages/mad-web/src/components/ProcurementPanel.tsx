import { useEffect, useState } from "react";
import { useLocale } from "../i18n/LocaleContext";
import type { ProcurementConfig } from "../types";

interface ProcurementPanelProps {
  procurement: ProcurementConfig;
  onChange: (procurement: ProcurementConfig) => Promise<void>;
  collapsed?: boolean;
}

export function ProcurementPanel({
  procurement,
  onChange,
  collapsed: initialCollapsed,
}: ProcurementPanelProps) {
  const { t } = useLocale();
  const [collapsed, setCollapsed] = useState(initialCollapsed ?? false);
  const [local, setLocal] = useState(procurement);
  const [dirty, setDirty] = useState(false);

  useEffect(() => {
    if (!dirty) setLocal(procurement);
  }, [procurement, dirty]);

  const updateLocal = (patch: Partial<ProcurementConfig>) => {
    setLocal((prev) => ({ ...prev, ...patch }));
    setDirty(true);
  };

  const apply = async () => {
    await onChange(local);
    setDirty(false);
  };

  const reset = () => {
    setLocal(procurement);
    setDirty(false);
  };

  return (
    <div className={`procurement-panel ${collapsed ? "collapsed" : ""}`}>
      <div className="procurement-header">
        <h3>{t.procurement.title}</h3>
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
        <div className="procurement-body">
          <p className="procurement-intro">{t.procurement.intro}</p>

          <label className="toggle-row">
            <input
              type="checkbox"
              checked={local.use_price_in_ranking}
              onChange={(e) => updateLocal({ use_price_in_ranking: e.target.checked })}
            />
            <span>{t.procurement.usePriceInRanking}</span>
          </label>

          <div className="procurement-grid">
            <label>
              {t.procurement.deviceCount}
              <input
                type="number"
                min={1}
                step={1}
                value={local.device_count}
                onChange={(e) => updateLocal({ device_count: Math.max(1, +e.target.value || 1) })}
              />
            </label>
            <label>
              {t.procurement.priceWeight}
              <input
                type="range"
                min={0}
                max={100}
                step={5}
                value={local.price_weight_percent}
                onChange={(e) => updateLocal({ price_weight_percent: +e.target.value })}
              />
              <span className="range-val">{local.price_weight_percent}%</span>
            </label>
          </div>

          <p className="procurement-hint">{t.procurement.hint}</p>

          <div className="procurement-actions">
            <button type="button" className="btn btn-primary" onClick={apply} disabled={!dirty}>
              {t.common.saveChanges}
            </button>
            <button type="button" className="btn btn-secondary" onClick={reset} disabled={!dirty}>
              {t.common.reset}
            </button>
          </div>
        </div>
      )}

      <style>{`
        .procurement-panel {
          background: white;
          border-radius: var(--mad-radius);
          padding: 1rem 1.25rem;
          margin-bottom: 1rem;
          box-shadow: var(--mad-shadow);
          border-left: 4px solid var(--mad-partial);
        }
        .procurement-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 1rem;
        }
        .procurement-header h3 { margin: 0; color: var(--mad-navy); font-size: 0.95rem; }
        .procurement-intro, .procurement-hint {
          margin: 0.75rem 0 0;
          font-size: 0.82rem;
          color: var(--mad-text-muted);
          line-height: 1.5;
        }
        .procurement-body { margin-top: 0.5rem; }
        .toggle-row {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          font-size: 0.85rem;
          margin: 0.75rem 0;
        }
        .procurement-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 1rem;
        }
        .procurement-grid label {
          display: flex;
          flex-direction: column;
          gap: 0.35rem;
          font-size: 0.8rem;
          font-weight: 600;
          color: var(--mad-navy);
        }
        .procurement-grid input[type="number"] {
          padding: 0.45rem 0.6rem;
          border: 1px solid var(--mad-border);
          border-radius: 6px;
          font-family: inherit;
        }
        .range-val {
          font-family: var(--font-mono);
          font-size: 0.85rem;
          color: var(--mad-cyan-dim);
        }
        .procurement-actions {
          display: flex;
          gap: 0.5rem;
          margin-top: 1rem;
        }
      `}</style>
    </div>
  );
}
