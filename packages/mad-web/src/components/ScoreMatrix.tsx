import type { ComplianceStatus, EvaluationReport, Pillar } from "../types";
import { PILLAR_LABELS, STATUS_LABELS } from "../types";

interface ScoreMatrixProps {
  pillars: Pillar[];
  evaluation: EvaluationReport;
  onCycle: (vendorId: string, requirementId: string) => void;
}

function statusClass(s: ComplianceStatus): string {
  return `cell status-${s}`;
}

function getStatus(
  evaluation: EvaluationReport,
  vendorId: string,
  requirementId: string,
): ComplianceStatus {
  const vendor = evaluation.vendors.find((v) => v.vendor.id === vendorId);
  if (!vendor) return "untested";
  for (const pillar of vendor.pillars) {
    const req = pillar.requirements.find((r) => r.requirement_id === requirementId);
    if (req) return req.status;
  }
  return "untested";
}

export function ScoreMatrix({ pillars, evaluation, onCycle }: ScoreMatrixProps) {
  const vendors = evaluation.vendors.map((v) => v.vendor);
  const requirements = pillars.flatMap((p) =>
    p.requirements.map((r) => ({ ...r, pillarId: p.id })),
  );

  return (
    <section className="score-matrix">
      <h2>Scoring Matrix</h2>
      <p className="intro">
        Click any cell to cycle compliance status: untested → compliant → partial →
        non-compliant. Scores update automatically with severity weighting.
      </p>

      <div className="legend">
        {(Object.keys(STATUS_LABELS) as ComplianceStatus[]).map((s) => (
          <span key={s} className={statusClass(s)}>
            {STATUS_LABELS[s]}
          </span>
        ))}
      </div>

      <div className="matrix-scroll">
        <table className="matrix">
          <thead>
            <tr>
              <th className="sticky-col">Criterion</th>
              <th className="sticky-col">Pillar</th>
              <th className="sticky-col">Sev.</th>
              {vendors.map((v) => (
                <th key={v.id} className="vendor-col">
                  <span className="vendor-name">{v.name}</span>
                  <span className="vendor-score">
                    {evaluation.vendors
                      .find((e) => e.vendor.id === v.id)
                      ?.overall_score.overall_score_percent.toFixed(0)}
                    %
                  </span>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {requirements.map((req) => (
              <tr key={req.id}>
                <td className="sticky-col req-cell">
                  <code>{req.id}</code>
                  <span>{req.title}</span>
                </td>
                <td className="sticky-col">{PILLAR_LABELS[req.pillarId]}</td>
                <td className="sticky-col">
                  <span className={`badge sev-${req.severity}`}>{req.severity}</span>
                </td>
                {vendors.map((v) => {
                  const status = getStatus(evaluation, v.id, req.id);
                  return (
                    <td key={v.id}>
                      <button
                        type="button"
                        className={statusClass(status)}
                        onClick={() => onCycle(v.id, req.id)}
                        title={`${STATUS_LABELS[status]} — click to change`}
                      >
                        {STATUS_LABELS[status].charAt(0)}
                      </button>
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <style>{`
        .score-matrix h2 { margin: 0; color: var(--mad-navy); }
        .intro { color: var(--mad-text-muted); font-size: 0.9rem; margin: 0.25rem 0 1rem; }
        .legend { display: flex; gap: 0.5rem; flex-wrap: wrap; margin-bottom: 1rem; }
        .legend span { font-size: 0.75rem; font-weight: 600; padding: 0.2rem 0.5rem;
          border-radius: 4px; }
        .matrix-scroll { overflow: auto; background: white; border-radius: 10px;
          box-shadow: 0 2px 8px rgba(10,22,40,0.08); max-height: 70vh; }
        .matrix { border-collapse: collapse; font-size: 0.8rem; min-width: 100%; }
        .matrix th, .matrix td { padding: 0.4rem 0.5rem; border: 1px solid #e8eaed;
          text-align: center; white-space: nowrap; }
        .matrix th { background: var(--mad-navy-light); color: white; position: sticky; top: 0; z-index: 2; }
        .sticky-col { position: sticky; left: 0; z-index: 1; background: white;
          text-align: left !important; max-width: 200px; white-space: normal !important; }
        .matrix thead .sticky-col { background: var(--mad-navy-light); z-index: 3; }
        .req-cell { display: flex; flex-direction: column; gap: 0.15rem; }
        .req-cell code { font-size: 0.7rem; color: var(--mad-cyan-dim); }
        .vendor-col { min-width: 90px; }
        .vendor-name { display: block; font-size: 0.75rem; }
        .vendor-score { display: block; font-family: var(--font-mono); color: var(--mad-cyan);
          font-size: 0.85rem; }
        .matrix button {
          width: 100%; min-width: 36px; height: 32px; border: none; border-radius: 4px;
          font-weight: 700; font-size: 0.75rem; cursor: pointer; transition: transform 0.1s;
        }
        .matrix button:hover { transform: scale(1.08); }
        .status-compliant { background: #e8f5e9; color: var(--mad-compliant); }
        .status-partial { background: #fff8e1; color: var(--mad-partial); }
        .status-non_compliant { background: #fde8ea; color: var(--mad-gap); }
        .status-untested { background: #eceff1; color: var(--mad-text-muted); }
        .badge { font-size: 0.65rem; font-weight: 700; text-transform: uppercase;
          padding: 0.1rem 0.35rem; border-radius: 3px; }
        .sev-critical { background: #fde8ea; color: var(--mad-critical); }
        .sev-high { background: #fff3e0; color: var(--mad-high); }
        .sev-medium { background: #fff8e1; color: #b8860b; }
      `}</style>
    </section>
  );
}
