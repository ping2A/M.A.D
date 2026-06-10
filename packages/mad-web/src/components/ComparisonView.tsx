import type { EvaluationReport, Pillar, PillarId } from "../types";
import { PILLAR_LABELS } from "../types";

interface ComparisonViewProps {
  evaluation: EvaluationReport;
  pillars: Pillar[];
}

const PILLAR_ORDER: PillarId[] = ["cybersecurity_dlp", "dfir", "platform_os"];

const CHART_COLORS = ["#00b4d8", "#1e3a5f", "#28a745", "#fd7e14", "#6f42c1"];

function scoreColor(pct: number): string {
  if (pct >= 90) return "var(--mad-compliant)";
  if (pct >= 70) return "var(--mad-partial)";
  return "var(--mad-gap)";
}

function RadarChart({ evaluation }: { evaluation: EvaluationReport }) {
  const cx = 160;
  const cy = 160;
  const maxR = 120;
  const n = PILLAR_ORDER.length;
  const angleStep = (2 * Math.PI) / n;

  const axisPoints = PILLAR_ORDER.map((_, i) => {
    const a = -Math.PI / 2 + i * angleStep;
    return { x: cx + maxR * Math.cos(a), y: cy + maxR * Math.sin(a), label: PILLAR_LABELS[PILLAR_ORDER[i]] };
  });

  const gridLevels = [0.25, 0.5, 0.75, 1.0];

  return (
    <div className="radar-wrap">
      <svg viewBox="0 0 320 340" className="radar">
        {gridLevels.map((level) => (
          <polygon
            key={level}
            points={PILLAR_ORDER.map((_, i) => {
              const a = -Math.PI / 2 + i * angleStep;
              const r = maxR * level;
              return `${cx + r * Math.cos(a)},${cy + r * Math.sin(a)}`;
            }).join(" ")}
            fill="none"
            stroke="#dde1e6"
            strokeWidth="1"
          />
        ))}
        {axisPoints.map((p, i) => (
          <g key={i}>
            <line x1={cx} y1={cy} x2={p.x} y2={p.y} stroke="#dde1e6" />
            <text x={p.x} y={p.y} textAnchor="middle" dominantBaseline="middle"
              className="axis-label" transform={`translate(${(p.x - cx) * 0.18}, ${(p.y - cy) * 0.18})`}>
              {p.label.split(" ")[0]}
            </text>
          </g>
        ))}
        {evaluation.vendors.map((result, vi) => {
          const points = PILLAR_ORDER.map((pid, i) => {
            const pillar = result.pillars.find((p) => p.pillar_id === pid);
            const pct = (pillar?.score.score_percent ?? 0) / 100;
            const a = -Math.PI / 2 + i * angleStep;
            const r = maxR * pct;
            return `${cx + r * Math.cos(a)},${cy + r * Math.sin(a)}`;
          }).join(" ");
          return (
            <polygon
              key={result.vendor.id}
              points={points}
              fill={CHART_COLORS[vi % CHART_COLORS.length]}
              fillOpacity="0.2"
              stroke={CHART_COLORS[vi % CHART_COLORS.length]}
              strokeWidth="2"
            />
          );
        })}
      </svg>
      <div className="radar-legend">
        {evaluation.vendors.map((v, i) => (
          <span key={v.vendor.id}>
            <i style={{ background: CHART_COLORS[i % CHART_COLORS.length] }} />
            {v.vendor.name} ({v.overall_score.overall_score_percent.toFixed(0)}%)
          </span>
        ))}
      </div>
    </div>
  );
}

export function ComparisonView({ evaluation, pillars }: ComparisonViewProps) {
  const ranked = [...evaluation.vendors].sort(
    (a, b) => b.overall_score.overall_score_percent - a.overall_score.overall_score_percent,
  );

  const heatmapReqs = pillars.flatMap((p) =>
    p.requirements.map((r) => ({ ...r, pillarId: p.id })),
  );

  return (
    <section className="comparison">
      <h2>Vendor Comparison</h2>
      <p className="intro">
        Severity-weighted scores: critical criteria count 3× more than medium.
        Radar chart shows pillar balance; heatmap shows per-requirement compliance.
      </p>

      <div className="compare-grid">
        <div className="card">
          <h3>Leaderboard</h3>
          <ol className="leaderboard">
            {ranked.map((r, i) => (
              <li key={r.vendor.id}>
                <span className="rank">#{i + 1}</span>
                <span className="name">{r.vendor.name}</span>
                <span className="score" style={{ color: scoreColor(r.overall_score.overall_score_percent) }}>
                  {r.overall_score.overall_score_percent.toFixed(1)}%
                </span>
                {r.overall_score.critical_gaps.length > 0 && (
                  <span className="gaps">{r.overall_score.critical_gaps.length} critical gaps</span>
                )}
              </li>
            ))}
          </ol>
        </div>

        <div className="card">
          <h3>Pillar Radar</h3>
          <RadarChart evaluation={evaluation} />
        </div>
      </div>

      <div className="card">
        <h3>Pillar Scores</h3>
        <div className="pillar-bars">
          {PILLAR_ORDER.map((pid) => (
            <div key={pid} className="pillar-group">
              <h4>{PILLAR_LABELS[pid]}</h4>
              {ranked.map((r) => {
                const pillar = r.pillars.find((p) => p.pillar_id === pid);
                const pct = pillar?.score.score_percent ?? 0;
                return (
                  <div key={r.vendor.id} className="bar-row">
                    <span className="bar-label">{r.vendor.name}</span>
                    <div className="bar-track">
                      <div className="bar-fill" style={{ width: `${pct}%`, background: scoreColor(pct) }} />
                    </div>
                    <span className="bar-val">{pct.toFixed(0)}%</span>
                  </div>
                );
              })}
            </div>
          ))}
        </div>
      </div>

      <div className="card">
        <h3>Compliance Heatmap</h3>
        <div className="heatmap-scroll">
          <table className="heatmap">
            <thead>
              <tr>
                <th>Criterion</th>
                {ranked.map((r) => (
                  <th key={r.vendor.id}>{r.vendor.name.split(" ")[0]}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {heatmapReqs.map((req) => (
                <tr key={req.id}>
                  <td>
                    <code>{req.id}</code> {req.title}
                  </td>
                  {ranked.map((r) => {
                    let status = "untested";
                    for (const p of r.pillars) {
                      const found = p.requirements.find((x) => x.requirement_id === req.id);
                      if (found) { status = found.status; break; }
                    }
                    return (
                      <td key={r.vendor.id}>
                        <span className={`heat status-${status}`} title={status} />
                      </td>
                    );
                  })}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      <style>{`
        .comparison h2 { margin: 0; color: var(--mad-navy); }
        .intro { color: var(--mad-text-muted); font-size: 0.9rem; margin: 0.25rem 0 1.25rem; }
        .compare-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-bottom: 1rem; }
        @media (max-width: 800px) { .compare-grid { grid-template-columns: 1fr; } }
        .card {
          background: white; border-radius: 10px; padding: 1.25rem;
          box-shadow: 0 2px 8px rgba(10,22,40,0.08); margin-bottom: 1rem;
        }
        .card h3 { margin: 0 0 1rem; color: var(--mad-navy); font-size: 1rem; }
        .card h4 { margin: 0 0 0.5rem; font-size: 0.85rem; color: var(--mad-text-muted); }
        .leaderboard { list-style: none; padding: 0; margin: 0; }
        .leaderboard li {
          display: flex; align-items: center; gap: 0.75rem; padding: 0.6rem 0;
          border-bottom: 1px solid #e8eaed;
        }
        .rank { font-weight: 800; color: var(--mad-navy); opacity: 0.3; min-width: 2rem; }
        .name { flex: 1; font-weight: 600; }
        .score { font-family: var(--font-mono); font-weight: 800; font-size: 1.1rem; }
        .gaps { font-size: 0.75rem; color: var(--mad-critical); }
        .radar-wrap { display: flex; flex-direction: column; align-items: center; }
        .radar { width: 100%; max-width: 320px; }
        .axis-label { font-size: 9px; fill: var(--mad-text-muted); }
        .radar-legend { display: flex; flex-direction: column; gap: 0.35rem; margin-top: 0.5rem; font-size: 0.8rem; }
        .radar-legend span { display: flex; align-items: center; gap: 0.4rem; }
        .radar-legend i { width: 12px; height: 12px; border-radius: 2px; display: inline-block; }
        .pillar-bars { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 1.5rem; }
        .bar-row { display: grid; grid-template-columns: 100px 1fr 40px; gap: 0.5rem;
          align-items: center; margin-bottom: 0.35rem; font-size: 0.8rem; }
        .bar-label { color: var(--mad-text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
        .bar-track { height: 8px; background: #e8eaed; border-radius: 4px; overflow: hidden; }
        .bar-fill { height: 100%; border-radius: 4px; }
        .bar-val { font-family: var(--font-mono); font-weight: 600; text-align: right; }
        .heatmap-scroll { overflow-x: auto; }
        .heatmap { width: 100%; border-collapse: collapse; font-size: 0.8rem; }
        .heatmap th, .heatmap td { padding: 0.4rem 0.5rem; border: 1px solid #e8eaed; }
        .heatmap th { background: var(--mad-navy-light); color: white; }
        .heatmap code { font-size: 0.7rem; }
        .heat { display: block; width: 28px; height: 28px; border-radius: 4px; margin: 0 auto; }
        .heat.status-compliant { background: var(--mad-compliant); }
        .heat.status-partial { background: var(--mad-partial); }
        .heat.status-non_compliant { background: var(--mad-gap); }
        .heat.status-untested { background: #eceff1; }
      `}</style>
    </section>
  );
}
