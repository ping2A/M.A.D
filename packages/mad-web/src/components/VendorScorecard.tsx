import type { EvaluationResult } from "../types";
import { useLocale } from "../i18n/LocaleContext";

interface VendorScorecardProps {
  result: EvaluationResult;
  rank: number;
  detailed?: boolean;
}

function scoreColor(percent: number): string {
  if (percent >= 90) return "var(--mad-compliant)";
  if (percent >= 70) return "var(--mad-partial)";
  return "var(--mad-gap)";
}

export function VendorScorecard({ result, rank, detailed }: VendorScorecardProps) {
  const { t, statusLabel } = useLocale();
  const { vendor, overall_score, pillars } = result;
  const score = overall_score.overall_score_percent;

  return (
    <article className="vendor-card">
      <div className="vendor-rank">#{rank}</div>
      <div className="vendor-body">
        <div className="vendor-header">
          <h3 className="vendor-name">{vendor.name}</h3>
          <div className="vendor-score" style={{ color: scoreColor(score) }}>
            {score.toFixed(1)}%
          </div>
        </div>
        <p className="vendor-desc">{vendor.description}</p>

        <div className="pillar-bars">
          {pillars.map((p) => (
            <div key={p.pillar_id} className="pillar-bar-row">
              <span className="pillar-bar-label">{p.pillar_name}</span>
              {p.score.total > 0 ? (
                <>
                  <div className="pillar-bar-track">
                    <div
                      className="pillar-bar-fill"
                      style={{
                        width: `${p.score.score_percent}%`,
                        background: scoreColor(p.score.score_percent),
                      }}
                    />
                  </div>
                  <span className="pillar-bar-value">{p.score.score_percent.toFixed(0)}%</span>
                </>
              ) : (
                <>
                  <div className="pillar-bar-track pillar-bar-na" />
                  <span className="pillar-bar-value pillar-bar-na">{t.matrix.notApplicable}</span>
                </>
              )}
            </div>
          ))}
        </div>

        {detailed &&
          pillars.map((p) => (
            <details key={p.pillar_id} className="pillar-detail">
              <summary>
                {p.pillar_name} — {t.scorecards.breakdown}
              </summary>
              <table className="detail-table">
                <thead>
                  <tr>
                    <th>{t.common.id}</th>
                    <th>{t.scorecards.requirement}</th>
                    <th>{t.common.status}</th>
                    <th>{t.scorecards.notes}</th>
                  </tr>
                </thead>
                <tbody>
                  {p.requirements.map((r) => (
                    <tr key={r.requirement_id}>
                      <td><code>{r.requirement_id}</code></td>
                      <td>{r.title}</td>
                      <td className={`status-${r.status}`}>{statusLabel(r.status)}</td>
                      <td>{r.notes ?? t.common.none}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </details>
          ))}

        {overall_score.critical_gaps.length > 0 && (
          <div className="critical-gaps">
            <strong>{t.scorecards.criticalGaps}</strong>
            <ul>
              {overall_score.critical_gaps.map((gap) => (
                <li key={gap}>{gap}</li>
              ))}
            </ul>
          </div>
        )}
      </div>
      <style>{`
        .vendor-card {
          display: flex;
          gap: 1rem;
          background: white;
          border-radius: 10px;
          padding: 1.25rem;
          box-shadow: 0 2px 8px rgba(10, 22, 40, 0.08);
          border-left: 4px solid var(--mad-cyan);
        }
        .vendor-rank {
          font-size: 1.5rem;
          font-weight: 800;
          color: var(--mad-navy);
          opacity: 0.3;
          min-width: 2rem;
        }
        .vendor-body { flex: 1; }
        .vendor-header {
          display: flex;
          justify-content: space-between;
          align-items: baseline;
        }
        .vendor-name {
          margin: 0;
          font-size: 1.15rem;
          color: var(--mad-navy);
        }
        .vendor-score {
          font-size: 1.5rem;
          font-weight: 800;
          font-family: var(--font-mono);
        }
        .vendor-desc {
          margin: 0.25rem 0 1rem;
          font-size: 0.85rem;
          color: var(--mad-text-muted);
        }
        .pillar-bars { display: flex; flex-direction: column; gap: 0.5rem; }
        .pillar-bar-row {
          display: grid;
          grid-template-columns: 1fr 2fr auto;
          gap: 0.75rem;
          align-items: center;
        }
        .pillar-bar-label { font-size: 0.8rem; color: var(--mad-text-muted); }
        .pillar-bar-track {
          height: 8px;
          background: #e8eaed;
          border-radius: 4px;
          overflow: hidden;
        }
        .pillar-bar-fill {
          height: 100%;
          border-radius: 4px;
          transition: width 0.5s ease;
        }
        .pillar-bar-value {
          font-family: var(--font-mono);
          font-size: 0.8rem;
          font-weight: 600;
          min-width: 2.5rem;
          text-align: right;
        }
        .critical-gaps {
          margin-top: 1rem;
          padding: 0.75rem;
          background: #fde8ea;
          border-radius: 6px;
          font-size: 0.85rem;
        }
        .critical-gaps ul { margin: 0.5rem 0 0; padding-left: 1.25rem; }
        .critical-gaps li { margin-bottom: 0.25rem; }
        .pillar-detail { margin-top: 0.75rem; font-size: 0.85rem; }
        .pillar-detail summary {
          cursor: pointer;
          font-weight: 600;
          color: var(--mad-navy);
          padding: 0.5rem 0;
        }
        .detail-table {
          width: 100%;
          border-collapse: collapse;
          margin-top: 0.5rem;
          font-size: 0.8rem;
        }
        .detail-table th, .detail-table td {
          padding: 0.4rem 0.5rem;
          text-align: left;
          border-bottom: 1px solid #e8eaed;
        }
        .detail-table th { background: #f4f6f8; font-weight: 600; }
        .detail-table code { font-family: var(--font-mono); font-size: 0.75rem; }
        .detail-table .status-compliant { color: var(--mad-compliant); font-weight: 600; }
        .detail-table .status-partial { color: var(--mad-partial); font-weight: 600; }
        .detail-table .status-non_compliant { color: var(--mad-gap); font-weight: 600; }
        .detail-table .status-untested { color: var(--mad-text-muted); }
      `}</style>
    </article>
  );
}
