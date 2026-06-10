import type { ScoringConfig } from "../types";

interface ScoringPanelProps {
  scoring: ScoringConfig;
  onChange: (scoring: ScoringConfig) => Promise<void>;
}

export function ScoringPanel({ scoring, onChange }: ScoringPanelProps) {
  const update = (patch: Partial<ScoringConfig>) => {
    onChange({ ...scoring, ...patch });
  };

  return (
    <div className="scoring-panel">
      <h3>Scoring Model</h3>
      <label className="toggle">
        <input
          type="checkbox"
          checked={scoring.use_severity_weighting}
          onChange={(e) => update({ use_severity_weighting: e.target.checked })}
        />
        Severity weighting (critical ×{scoring.critical_weight}, high ×{scoring.high_weight}, medium ×{scoring.medium_weight})
      </label>
      <div className="weights">
        <span>Compliant = {scoring.compliant_points}</span>
        <span>Partial = {scoring.partial_points}</span>
        <span>Non-compliant = {scoring.non_compliant_points}</span>
      </div>
      <style>{`
        .scoring-panel {
          background: var(--mad-navy-light); color: white; border-radius: 8px;
          padding: 1rem 1.25rem; margin-bottom: 1.25rem; font-size: 0.85rem;
        }
        .scoring-panel h3 { margin: 0 0 0.5rem; font-size: 0.9rem; color: var(--mad-cyan); }
        .toggle { display: flex; align-items: center; gap: 0.5rem; cursor: pointer; }
        .weights { display: flex; gap: 1rem; margin-top: 0.5rem; color: var(--mad-silver);
          font-family: var(--font-mono); font-size: 0.75rem; flex-wrap: wrap; }
      `}</style>
    </div>
  );
}
