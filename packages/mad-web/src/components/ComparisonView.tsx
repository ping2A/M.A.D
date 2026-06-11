import { useCallback, useEffect, useMemo, useState } from "react";
import { ComparisonFilterBar } from "./ComparisonFilterBar";
import { ComplianceStatusBadge } from "./ComplianceStatusBadge";
import type {
  ComplianceStatus,
  EvaluationReport,
  EvaluationResult,
  Pillar,
  PillarId,
} from "../types";
import { useLocale } from "../i18n/LocaleContext";
import { formatMoney } from "../utils/pricing";
import { buildFilteredEvaluation } from "../utils/comparisonFilter";
import { rankScore, rankVendors, usesCompositeRanking } from "../utils/ranking";

interface ComparisonViewProps {
  evaluation: EvaluationReport;
  pillars: Pillar[];
}

type CompareView = "overview" | "matrix" | "breakdown" | "heatmap" | "tradeoffs" | "value";

const CHART_COLORS = ["#00b4d8", "#1e3a5f", "#28a745", "#fd7e14", "#6f42c1"];

const STATUS_COLORS: Record<ComplianceStatus, string> = {
  compliant: "var(--mad-compliant)",
  partial: "var(--mad-partial)",
  non_compliant: "var(--mad-gap)",
  untested: "#b0bec5",
};

interface StatusCounts {
  compliant: number;
  partial: number;
  non_compliant: number;
  untested: number;
}

function scoreColor(pct: number): string {
  if (pct >= 90) return "var(--mad-compliant)";
  if (pct >= 70) return "var(--mad-partial)";
  return "var(--mad-gap)";
}

function getReqStatus(result: EvaluationResult, reqId: string): ComplianceStatus {
  for (const p of result.pillars) {
    const found = p.requirements.find((x) => x.requirement_id === reqId);
    if (found) return found.status;
  }
  return "untested";
}

function countStatuses(result: EvaluationResult): StatusCounts {
  const counts: StatusCounts = {
    compliant: 0,
    partial: 0,
    non_compliant: 0,
    untested: 0,
  };
  for (const p of result.pillars) {
    for (const r of p.requirements) {
      counts[r.status] += 1;
    }
  }
  return counts;
}

function RadarChart({
  results,
  pillarOrder,
  pillarShortName,
}: {
  results: EvaluationResult[];
  pillarOrder: PillarId[];
  pillarShortName: (id: PillarId) => string;
}) {
  const cx = 160;
  const cy = 160;
  const maxR = 120;
  const n = Math.max(pillarOrder.length, 1);
  const angleStep = (2 * Math.PI) / n;

  const axisPoints = pillarOrder.map((pid, i) => {
    const a = -Math.PI / 2 + i * angleStep;
    return {
      x: cx + maxR * Math.cos(a),
      y: cy + maxR * Math.sin(a),
      label: pillarShortName(pid),
    };
  });

  const gridLevels = [0.25, 0.5, 0.75, 1.0];

  return (
    <div className="radar-wrap">
      <svg viewBox="0 0 320 340" className="radar">
        {gridLevels.map((level) => (
          <polygon
            key={level}
            points={pillarOrder.map((_, i) => {
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
            <text
              x={p.x}
              y={p.y}
              textAnchor="middle"
              dominantBaseline="middle"
              className="axis-label"
              transform={`translate(${(p.x - cx) * 0.18}, ${(p.y - cy) * 0.18})`}
            >
              {p.label}
            </text>
          </g>
        ))}
        {results.map((result, vi) => {
          const points = pillarOrder.map((pid, i) => {
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
        {results.map((v, i) => (
          <span key={v.vendor.id}>
            <i style={{ background: CHART_COLORS[i % CHART_COLORS.length] }} />
            {v.vendor.name} ({v.overall_score.overall_score_percent.toFixed(0)}%)
          </span>
        ))}
      </div>
    </div>
  );
}

function PillarChampions({
  ranked,
  pillarOrder,
  pillarLabel,
}: {
  ranked: EvaluationResult[];
  pillarOrder: PillarId[];
  pillarLabel: (id: PillarId) => string;
}) {
  const { t } = useLocale();

  return (
    <div className="champion-grid">
      {pillarOrder.map((pid) => {
        const scores = ranked.map((r) => {
          const pillar = r.pillars.find((p) => p.pillar_id === pid);
          return { vendor: r.vendor.name, pct: pillar?.score.score_percent ?? 0 };
        });
        const max = Math.max(...scores.map((s) => s.pct));
        const leaders = scores.filter((s) => s.pct === max && max > 0);
        return (
          <div key={pid} className="champion-card">
            <span className="champion-pillar">{pillarLabel(pid)}</span>
            <span className="champion-score" style={{ color: scoreColor(max) }}>
              {max.toFixed(0)}%
            </span>
            <span className="champion-vendor">
              {leaders.length > 1
                ? `${t.compare.tied}: ${leaders.map((l) => l.vendor).join(", ")}`
                : leaders[0]?.vendor ?? "—"}
            </span>
          </div>
        );
      })}
    </div>
  );
}

function ValueComparison({
  evaluation,
  ranked,
}: {
  evaluation: EvaluationReport;
  ranked: EvaluationResult[];
}) {
  const { t } = useLocale();
  const compositeMode = usesCompositeRanking(evaluation);
  const maxCost = Math.max(
    ...ranked.map((r) => r.overall_score.annual_cost_per_device ?? 0),
    1,
  );

  return (
    <div className="value-comparison">
      {compositeMode && ranked[0]?.overall_score.composite_score_percent != null && (
        <div className="best-value-banner">
          {t.compare.bestValue}: <strong>{ranked[0].vendor.name}</strong> (
          {rankScore(ranked[0], evaluation).toFixed(1)}%)
        </div>
      )}

      <div className="value-chart">
        <h4>{t.compare.costVsCapability}</h4>
        <p className="card-hint">
          {t.compare.valueHint} · {evaluation.procurement.device_count}{" "}
          {t.procurement.deviceCount.toLowerCase()}
        </p>
        {ranked.map((r, i) => {
          const os = r.overall_score;
          const cap = os.overall_score_percent;
          const cost = os.annual_cost_per_device;
          const costPct = cost != null ? (cost / maxCost) * 100 : 0;
          return (
            <div key={r.vendor.id} className="value-row">
              <span className="value-rank">#{i + 1}</span>
              <span className="value-name">{r.vendor.name}</span>
              <div className="value-bars">
                <div className="value-bar-row">
                  <span>{t.procurement.capabilityScore}</span>
                  <div className="bar-track">
                    <div className="bar-fill cap" style={{ width: `${cap}%` }} />
                  </div>
                  <span>{cap.toFixed(0)}%</span>
                </div>
                {cost != null && (
                  <div className="value-bar-row">
                    <span>{t.procurement.costPerDeviceAnnual}</span>
                    <div className="bar-track">
                      <div className="bar-fill cost" style={{ width: `${costPct}%` }} />
                    </div>
                    <span>{formatMoney(cost, os.price_currency ?? "USD")}</span>
                  </div>
                )}
                {os.total_annual_cost != null && (
                  <div className="value-total">
                    {t.procurement.totalAnnual}:{" "}
                    {formatMoney(os.total_annual_cost, os.price_currency ?? "USD")}
                  </div>
                )}
                {os.price_score_percent != null && (
                  <div className="value-scores">
                    {t.procurement.priceScore}: {os.price_score_percent.toFixed(0)}%
                    {os.composite_score_percent != null && (
                      <> · {t.procurement.compositeScore}: {os.composite_score_percent.toFixed(1)}%</>
                    )}
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function SummaryMatrix({
  ranked,
  pillarOrder,
  pillarLabel,
  evaluation,
}: {
  ranked: EvaluationResult[];
  pillarOrder: PillarId[];
  pillarLabel: (id: PillarId) => string;
  evaluation: EvaluationReport;
}) {
  const { t } = useLocale();

  type RowKind = "score-high" | "score-low" | "neutral";
  interface MatrixRow {
    label: string;
    values: { text: string; raw: number; kind: RowKind }[];
  }

  const rows: MatrixRow[] = useMemo(() => {
    const overallRow: MatrixRow = {
      label: t.compare.metricOverall,
      values: ranked.map((r) => ({
        text: `${r.overall_score.overall_score_percent.toFixed(1)}%`,
        raw: r.overall_score.overall_score_percent,
        kind: "score-high",
      })),
    };

    const pillarRows = pillarOrder.map((pid) => ({
      label: pillarLabel(pid),
      values: ranked.map((r) => {
        const pillar = r.pillars.find((p) => p.pillar_id === pid);
        const pct = pillar?.score.score_percent ?? 0;
        return { text: `${pct.toFixed(1)}%`, raw: pct, kind: "score-high" as RowKind };
      }),
    }));

    const gapsRow: MatrixRow = {
      label: t.compare.metricCriticalGaps,
      values: ranked.map((r) => ({
        text: String(r.overall_score.critical_gaps.length),
        raw: r.overall_score.critical_gaps.length,
        kind: "score-low",
      })),
    };

    const priceRows: MatrixRow[] = [];
    if (ranked.some((r) => r.overall_score.annual_cost_per_device != null)) {
      priceRows.push({
        label: t.procurement.costPerDeviceAnnual,
        values: ranked.map((r) => ({
          text:
            r.overall_score.annual_cost_per_device != null
              ? formatMoney(
                  r.overall_score.annual_cost_per_device,
                  r.overall_score.price_currency ?? "USD",
                )
              : "—",
          raw: r.overall_score.annual_cost_per_device ?? -1,
          kind: "score-low",
        })),
      });
      priceRows.push({
        label: t.procurement.totalAnnual,
        values: ranked.map((r) => ({
          text:
            r.overall_score.total_annual_cost != null
              ? formatMoney(
                  r.overall_score.total_annual_cost,
                  r.overall_score.price_currency ?? "USD",
                  true,
                )
              : "—",
          raw: r.overall_score.total_annual_cost ?? -1,
          kind: "score-low",
        })),
      });
    }
    if (usesCompositeRanking(evaluation)) {
      priceRows.push({
        label: t.procurement.compositeScore,
        values: ranked.map((r) => ({
          text:
            r.overall_score.composite_score_percent != null
              ? `${r.overall_score.composite_score_percent.toFixed(1)}%`
              : "—",
          raw: r.overall_score.composite_score_percent ?? -1,
          kind: "score-high",
        })),
      });
    }

    const statusRows: MatrixRow[] = (["compliant", "partial", "non_compliant", "untested"] as const).map(
      (status) => ({
        label: t.status[status],
        values: ranked.map((r) => {
          const c = countStatuses(r)[status];
          return {
            text: String(c),
            raw: c,
            kind:
              status === "compliant"
                ? "score-high"
                : status === "non_compliant" || status === "untested"
                  ? "score-low"
                  : "neutral",
          };
        }),
      }),
    );

    return [overallRow, ...pillarRows, gapsRow, ...priceRows, ...statusRows];
  }, [ranked, pillarOrder, pillarLabel, evaluation, t]);

  function cellClass(kind: RowKind, raw: number, row: MatrixRow): string {
    if (kind === "neutral") return "";
    const raws = row.values.map((v) => v.raw);
    const best =
      kind === "score-high" ? Math.max(...raws) : Math.min(...raws);
    if (raw !== best) return "";
    if (kind === "score-high" && raw > 0) return "cell-best";
    if (kind === "score-low" && raw === best && raw > 0) return "cell-worst";
    if (kind === "score-low" && raw === 0 && best === 0) return "cell-best";
    return "";
  }

  return (
    <div className="matrix-scroll">
      <table className="summary-matrix">
        <thead>
          <tr>
            <th>{t.matrix.criterion}</th>
            {ranked.map((r) => (
              <th key={r.vendor.id}>{r.vendor.name}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.label}>
              <td className="row-label">{row.label}</td>
              {row.values.map((cell, i) => (
                <td key={ranked[i].vendor.id} className={cellClass(cell.kind, cell.raw, row)}>
                  {cell.text}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function StatusBreakdown({ ranked }: { ranked: EvaluationResult[] }) {
  const { t, statusLabel } = useLocale();
  const statuses: ComplianceStatus[] = ["compliant", "partial", "non_compliant", "untested"];

  return (
    <div className="breakdown-list">
      {ranked.map((r) => {
        const counts = countStatuses(r);
        const total = counts.compliant + counts.partial + counts.non_compliant + counts.untested;
        return (
          <div key={r.vendor.id} className="breakdown-row">
            <div className="breakdown-header">
              <span className="breakdown-name">{r.vendor.name}</span>
              <span className="breakdown-overall" style={{ color: scoreColor(r.overall_score.overall_score_percent) }}>
                {r.overall_score.overall_score_percent.toFixed(1)}%
              </span>
            </div>
            <div className="stacked-bar" title={t.compare.statusBreakdownHint}>
              {statuses.map((status) => {
                const n = counts[status];
                if (n === 0 || total === 0) return null;
                const pct = (n / total) * 100;
                return (
                  <div
                    key={status}
                    className="stack-segment"
                    style={{ width: `${pct}%`, background: STATUS_COLORS[status] }}
                    title={`${statusLabel(status)}: ${n}`}
                  />
                );
              })}
            </div>
            <div className="breakdown-legend">
              {statuses.map((status) => (
                <span key={status}>
                  <i style={{ background: STATUS_COLORS[status] }} />
                  {statusLabel(status)} {counts[status]}
                </span>
              ))}
            </div>
          </div>
        );
      })}
      <div className="status-legend-global">
        {statuses.map((status) => (
          <span key={status}>
            <i style={{ background: STATUS_COLORS[status] }} />
            {statusLabel(status)}
          </span>
        ))}
      </div>
    </div>
  );
}

function TradeoffsView({
  ranked,
  heatmapReqs,
}: {
  ranked: EvaluationResult[];
  heatmapReqs: { id: string; title: string; severity: string }[];
}) {
  const { t, format, statusLabel } = useLocale();

  const splitDecisions = useMemo(() => {
    return heatmapReqs.filter((req) => {
      const statuses = new Set(ranked.map((r) => getReqStatus(r, req.id)));
      return statuses.size > 1;
    });
  }, [heatmapReqs, ranked]);

  const uniqueStrengths = useMemo(() => {
    return heatmapReqs
      .map((req) => {
        const compliant = ranked.filter((r) => getReqStatus(r, req.id) === "compliant");
        if (compliant.length !== 1) return null;
        return { req, vendor: compliant[0].vendor.name };
      })
      .filter((x): x is NonNullable<typeof x> => x !== null);
  }, [heatmapReqs, ranked]);

  return (
    <>
      <div className="card">
        <h3>{t.compare.splitDecisions}</h3>
        <p className="card-hint">{t.compare.splitDecisionsHint}</p>
        {splitDecisions.length === 0 ? (
          <p className="empty-msg">{t.compare.noSplitDecisions}</p>
        ) : (
          <div className="tradeoff-scroll">
            <table className="tradeoff-table">
              <thead>
                <tr>
                  <th>{t.matrix.criterion}</th>
                  {ranked.map((r) => (
                    <th key={r.vendor.id}>{r.vendor.name}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {splitDecisions.map((req) => {
                  const statuses = ranked.map((r) => getReqStatus(r, req.id));
                  const uniqueCount = new Set(statuses).size;
                  return (
                    <tr key={req.id}>
                      <td>
                        <code>{req.id}</code>
                        <span className="req-title">{req.title}</span>
                        <span className="disagree-badge">
                          {format(t.compare.vendorsDisagree, { count: uniqueCount })}
                        </span>
                      </td>
                      {ranked.map((r) => {
                        const status = getReqStatus(r, req.id);
                        return (
                          <td key={r.vendor.id}>
                            <ComplianceStatusBadge
                              status={status}
                              label={statusLabel(status)}
                              variant="inline"
                            />
                          </td>
                        );
                      })}
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>

      <div className="card">
        <h3>{t.compare.uniqueStrengths}</h3>
        <p className="card-hint">{t.compare.uniqueStrengthsHint}</p>
        {uniqueStrengths.length === 0 ? (
          <p className="empty-msg">{t.compare.noUniqueStrengths}</p>
        ) : (
          <ul className="strength-list">
            {uniqueStrengths.map(({ req, vendor }) => (
              <li key={req.id}>
                <code>{req.id}</code>
                <span className="req-title">{req.title}</span>
                <span className="strength-badge">{format(t.compare.onlyCompliant, { vendor })}</span>
              </li>
            ))}
          </ul>
        )}
      </div>
    </>
  );
}

export function ComparisonView({ evaluation, pillars }: ComparisonViewProps) {
  const { t, format, pillarLabel, statusLabel } = useLocale();
  const [view, setView] = useState<CompareView>("overview");
  const allVendorIds = useMemo(
    () => evaluation.vendors.map((v) => v.vendor.id),
    [evaluation.vendors],
  );
  const [selectedVendorIds, setSelectedVendorIds] = useState<Set<string>>(
    () => new Set(allVendorIds),
  );
  const [activeTags, setActiveTags] = useState<Set<string>>(() => new Set());

  useEffect(() => {
    setSelectedVendorIds((prev) => {
      const next = new Set<string>();
      for (const id of allVendorIds) {
        if (prev.has(id)) next.add(id);
      }
      for (const id of allVendorIds) {
        if (!prev.has(id)) next.add(id);
      }
      return next.size > 0 ? next : new Set(allVendorIds);
    });
  }, [allVendorIds]);

  const filteredEvaluation = useMemo(
    () => buildFilteredEvaluation(evaluation, selectedVendorIds, activeTags),
    [evaluation, selectedVendorIds, activeTags],
  );
  const ranked = useMemo(() => rankVendors(filteredEvaluation), [filteredEvaluation]);
  const compositeMode = usesCompositeRanking(filteredEvaluation);

  const pillarOrder = useMemo(() => pillars.map((p) => p.id), [pillars]);

  const pillarDisplayName = useCallback(
    (id: PillarId) => {
      const pillar = pillars.find((p) => p.id === id);
      if (pillar && pillarLabel(id) === id) return pillar.name;
      return pillarLabel(id);
    },
    [pillars, pillarLabel],
  );

  const pillarShortName = useCallback(
    (id: PillarId) => pillarDisplayName(id).split(" ")[0],
    [pillarDisplayName],
  );

  const heatmapReqs = useMemo(
    () => pillars.flatMap((p) => p.requirements.map((r) => ({ ...r, pillarId: p.id }))),
    [pillars],
  );

  const viewButtons: { id: CompareView; label: string }[] = [
    { id: "overview", label: t.compare.views.overview },
    { id: "matrix", label: t.compare.views.matrix },
    { id: "breakdown", label: t.compare.views.breakdown },
    { id: "heatmap", label: t.compare.views.heatmap },
    { id: "tradeoffs", label: t.compare.views.tradeoffs },
    { id: "value", label: t.compare.views.value },
  ];

  return (
    <section className="comparison">
      <h2>{t.compare.title}</h2>
      <p className="intro">{t.compare.intro}</p>

      <ComparisonFilterBar
        evaluation={evaluation}
        selectedVendorIds={selectedVendorIds}
        activeTags={activeTags}
        onSelectedVendorIdsChange={setSelectedVendorIds}
        onActiveTagsChange={setActiveTags}
      />

      {ranked.length === 0 ? (
        <div className="card">
          <p className="empty-msg">{t.compare.noVendorsFiltered}</p>
        </div>
      ) : (
        <>
      <div className="view-tabs" role="tablist">
        {viewButtons.map((btn) => (
          <button
            key={btn.id}
            type="button"
            role="tab"
            aria-selected={view === btn.id}
            className={`view-tab ${view === btn.id ? "active" : ""}`}
            onClick={() => setView(btn.id)}
          >
            {btn.label}
          </button>
        ))}
      </div>

      {view === "overview" && (
        <>
          <div className="compare-grid">
            <div className="card">
              <h3>{t.compare.leaderboard}</h3>
              <ol className="leaderboard">
                {ranked.map((r, i) => (
                  <li key={r.vendor.id}>
                    <span className="rank">#{i + 1}</span>
                    <span className="name">{r.vendor.name}</span>
                    <div className="leader-bar-track">
                      <div
                        className="leader-bar-fill"
                        style={{
                          width: `${r.overall_score.overall_score_percent}%`,
                          background: scoreColor(r.overall_score.overall_score_percent),
                        }}
                      />
                    </div>
                    <span className="score" style={{ color: scoreColor(rankScore(r, filteredEvaluation)) }}>
                      {rankScore(r, filteredEvaluation).toFixed(1)}%
                      {compositeMode && r.overall_score.composite_score_percent != null ? " ★" : ""}
                    </span>
                    {r.overall_score.annual_cost_per_device != null && (
                      <span className="leader-cost">
                        {formatMoney(
                          r.overall_score.annual_cost_per_device,
                          r.overall_score.price_currency ?? "USD",
                          true,
                        )}
                        /yr·dev
                      </span>
                    )}
                    {r.overall_score.critical_gaps.length > 0 && (
                      <span className="gaps">
                        {format(t.compare.criticalGaps, {
                          count: r.overall_score.critical_gaps.length,
                        })}
                      </span>
                    )}
                  </li>
                ))}
              </ol>
            </div>

            <div className="card">
              <h3>{t.compare.pillarRadar}</h3>
              <RadarChart
                results={ranked}
                pillarOrder={pillarOrder}
                pillarShortName={pillarShortName}
              />
            </div>
          </div>

          <div className="card">
            <h3>{t.compare.pillarChampions}</h3>
            <PillarChampions
              ranked={ranked}
              pillarOrder={pillarOrder}
              pillarLabel={pillarDisplayName}
            />
          </div>

          <div className="card">
            <h3>{t.compare.pillarScores}</h3>
            <div className="pillar-bars">
              {pillarOrder.map((pid) => (
                <div key={pid} className="pillar-group">
                  <h4>{pillarDisplayName(pid)}</h4>
                  {ranked.map((r) => {
                    const pillar = r.pillars.find((p) => p.pillar_id === pid);
                    const pct = pillar?.score.score_percent ?? 0;
                    const isBest =
                      pct ===
                      Math.max(
                        ...ranked.map(
                          (v) => v.pillars.find((p) => p.pillar_id === pid)?.score.score_percent ?? 0,
                        ),
                      );
                    return (
                      <div key={r.vendor.id} className={`bar-row ${isBest && pct > 0 ? "bar-best" : ""}`}>
                        <span className="bar-label">{r.vendor.name}</span>
                        <div className="bar-track">
                          <div
                            className="bar-fill"
                            style={{ width: `${pct}%`, background: scoreColor(pct) }}
                          />
                        </div>
                        <span className="bar-val">{pct.toFixed(0)}%</span>
                      </div>
                    );
                  })}
                </div>
              ))}
            </div>
          </div>
        </>
      )}

      {view === "matrix" && (
        <div className="card">
          <h3>{t.compare.summaryMatrix}</h3>
          <p className="card-hint">{t.compare.summaryMatrixHint}</p>
          <SummaryMatrix
            ranked={ranked}
            pillarOrder={pillarOrder}
            pillarLabel={pillarDisplayName}
            evaluation={filteredEvaluation}
          />
        </div>
      )}

      {view === "breakdown" && (
        <div className="card">
          <h3>{t.compare.statusBreakdown}</h3>
          <p className="card-hint">{t.compare.statusBreakdownHint}</p>
          <StatusBreakdown ranked={ranked} />
        </div>
      )}

      {view === "heatmap" && (
        <div className="card">
          <h3>{t.compare.heatmap}</h3>
          <div className="heatmap-scroll">
            <table className="heatmap">
              <thead>
                <tr>
                  <th>{t.matrix.criterion}</th>
                  {ranked.map((r) => (
                    <th key={r.vendor.id}>{r.vendor.name}</th>
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
                      const status = getReqStatus(r, req.id);
                      return (
                        <td key={r.vendor.id}>
                          <span className={`heat status-${status}`} title={statusLabel(status)} />
                        </td>
                      );
                    })}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {view === "value" && (
        <div className="card">
          <h3>{t.compare.valueTitle}</h3>
          <ValueComparison evaluation={filteredEvaluation} ranked={ranked} />
        </div>
      )}

      {view === "tradeoffs" && (
        <>
          <p className="card-hint tradeoffs-intro">{t.compare.tradeoffsHint}</p>
          <TradeoffsView ranked={ranked} heatmapReqs={heatmapReqs} />
        </>
      )}

        </>
      )}

      <style>{`
        .comparison h2 { margin: 0; color: var(--mad-navy); }
        .intro { color: var(--mad-text-muted); font-size: 0.9rem; margin: 0.25rem 0 1rem; }
        .view-tabs {
          display: flex; flex-wrap: wrap; gap: 0.35rem; margin-bottom: 1.25rem;
        }
        .view-tab {
          background: white; border: 1px solid var(--mad-border); border-radius: 999px;
          padding: 0.45rem 0.9rem; font-size: 0.8rem; font-weight: 600;
          color: var(--mad-text-muted); cursor: pointer; font-family: inherit;
        }
        .view-tab:hover { border-color: var(--mad-cyan); color: var(--mad-navy); }
        .view-tab.active {
          background: var(--mad-navy); color: white; border-color: var(--mad-navy);
        }
        .compare-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-bottom: 1rem; }
        @media (max-width: 800px) { .compare-grid { grid-template-columns: 1fr; } }
        .card {
          background: white; border-radius: 10px; padding: 1.25rem;
          box-shadow: 0 2px 8px rgba(10,22,40,0.08); margin-bottom: 1rem;
        }
        .card h3 { margin: 0 0 0.5rem; color: var(--mad-navy); font-size: 1rem; }
        .card h4 { margin: 0 0 0.5rem; font-size: 0.85rem; color: var(--mad-text-muted); }
        .card-hint { margin: 0 0 1rem; font-size: 0.82rem; color: var(--mad-text-muted); line-height: 1.5; }
        .tradeoffs-intro { margin-top: 0; }
        .empty-msg { color: var(--mad-text-muted); font-size: 0.9rem; margin: 0; }
        .leaderboard { list-style: none; padding: 0; margin: 0; }
        .leaderboard li {
          display: grid; grid-template-columns: 2rem 1fr minmax(80px, 1.2fr) 3.5rem 5rem auto;
          align-items: center; gap: 0.5rem; padding: 0.6rem 0;
          border-bottom: 1px solid #e8eaed;
        }
        @media (max-width: 640px) {
          .leaderboard li {
            grid-template-columns: 2rem 1fr 3.5rem;
          }
          .leader-bar-track, .gaps { display: none; }
        }
        .rank { font-weight: 800; color: var(--mad-navy); opacity: 0.3; }
        .name { font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
        .leader-bar-track { height: 8px; background: #e8eaed; border-radius: 4px; overflow: hidden; }
        .leader-bar-fill { height: 100%; border-radius: 4px; }
        .score { font-family: var(--font-mono); font-weight: 800; font-size: 1rem; text-align: right; }
        .leader-cost { font-size: 0.7rem; color: var(--mad-text-muted); white-space: nowrap; }
        .gaps { font-size: 0.72rem; color: var(--mad-critical); white-space: nowrap; }
        .value-comparison { display: flex; flex-direction: column; gap: 1rem; }
        .best-value-banner {
          background: rgba(40, 167, 69, 0.12);
          border: 1px solid var(--mad-compliant);
          border-radius: 8px;
          padding: 0.65rem 1rem;
          font-size: 0.88rem;
          color: var(--mad-navy);
        }
        .value-row {
          display: grid; grid-template-columns: 2rem 120px 1fr; gap: 0.75rem;
          padding: 0.75rem 0; border-bottom: 1px solid var(--mad-border);
        }
        .value-rank { font-weight: 800; color: var(--mad-muted, var(--mad-text-muted)); opacity: 0.5; }
        .value-name { font-weight: 700; color: var(--mad-navy); }
        .value-bar-row {
          display: grid; grid-template-columns: 110px 1fr 72px; gap: 0.5rem;
          align-items: center; font-size: 0.78rem; margin-bottom: 0.35rem;
        }
        .value-bar-row .bar-fill.cap { background: var(--mad-compliant); height: 100%; border-radius: 4px; }
        .value-bar-row .bar-fill.cost { background: var(--mad-partial); height: 100%; border-radius: 4px; }
        .value-total, .value-scores { font-size: 0.75rem; color: var(--mad-text-muted); margin-top: 0.25rem; }
        .champion-grid {
          display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 0.75rem;
        }
        .champion-card {
          background: var(--mad-bg); border-radius: 8px; padding: 0.85rem 1rem;
          border-left: 3px solid var(--mad-cyan);
        }
        .champion-pillar { display: block; font-size: 0.75rem; color: var(--mad-text-muted); margin-bottom: 0.25rem; }
        .champion-score { display: block; font-size: 1.4rem; font-weight: 800; font-family: var(--font-mono); }
        .champion-vendor { display: block; font-size: 0.82rem; font-weight: 600; color: var(--mad-navy); margin-top: 0.2rem; }
        .radar-wrap { display: flex; flex-direction: column; align-items: center; }
        .radar { width: 100%; max-width: 320px; }
        .axis-label { font-size: 9px; fill: var(--mad-text-muted); }
        .radar-legend { display: flex; flex-direction: column; gap: 0.35rem; margin-top: 0.5rem; font-size: 0.8rem; }
        .radar-legend span { display: flex; align-items: center; gap: 0.4rem; }
        .radar-legend i { width: 12px; height: 12px; border-radius: 2px; display: inline-block; }
        .pillar-bars { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 1.5rem; }
        .bar-row {
          display: grid; grid-template-columns: 100px 1fr 40px; gap: 0.5rem;
          align-items: center; margin-bottom: 0.35rem; font-size: 0.8rem;
          padding: 0.15rem 0.25rem; border-radius: 4px;
        }
        .bar-row.bar-best { background: rgba(40, 167, 69, 0.08); }
        .bar-label { color: var(--mad-text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
        .bar-track { height: 8px; background: #e8eaed; border-radius: 4px; overflow: hidden; }
        .bar-fill { height: 100%; border-radius: 4px; }
        .bar-val { font-family: var(--font-mono); font-weight: 600; text-align: right; }
        .matrix-scroll, .heatmap-scroll, .tradeoff-scroll { overflow-x: auto; }
        .summary-matrix, .heatmap, .tradeoff-table {
          width: 100%; border-collapse: collapse; font-size: 0.82rem;
        }
        .summary-matrix th, .summary-matrix td,
        .heatmap th, .heatmap td,
        .tradeoff-table th, .tradeoff-table td {
          padding: 0.5rem 0.65rem; border: 1px solid #e8eaed; text-align: center;
        }
        .summary-matrix th, .heatmap th, .tradeoff-table th {
          background: var(--mad-navy-light); color: white; font-size: 0.78rem;
        }
        .summary-matrix .row-label, .tradeoff-table td:first-child {
          text-align: left; font-weight: 600; color: var(--mad-navy); background: #f8f9fa;
        }
        .summary-matrix .cell-best { background: rgba(40, 167, 69, 0.15); font-weight: 700; color: var(--mad-compliant); }
        .summary-matrix .cell-worst { background: rgba(220, 53, 69, 0.1); font-weight: 700; color: var(--mad-gap); }
        .heatmap code, .tradeoff-table code { font-size: 0.7rem; }
        .heat { display: block; width: 28px; height: 28px; border-radius: 4px; margin: 0 auto; }
        .heat.status-compliant { background: var(--mad-compliant); }
        .heat.status-partial { background: var(--mad-partial); }
        .heat.status-non_compliant { background: var(--mad-gap); }
        .heat.status-untested { background: #eceff1; }
        .breakdown-list { display: flex; flex-direction: column; gap: 1.25rem; }
        .breakdown-header { display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 0.4rem; }
        .breakdown-name { font-weight: 700; color: var(--mad-navy); }
        .breakdown-overall { font-family: var(--font-mono); font-weight: 800; font-size: 1.1rem; }
        .stacked-bar { display: flex; height: 14px; border-radius: 7px; overflow: hidden; background: #e8eaed; }
        .stack-segment { min-width: 2px; transition: width 0.2s; }
        .breakdown-legend { display: flex; flex-wrap: wrap; gap: 0.75rem; margin-top: 0.4rem; font-size: 0.75rem; color: var(--mad-text-muted); }
        .breakdown-legend span, .status-legend-global span {
          display: inline-flex; align-items: center; gap: 0.3rem;
        }
        .breakdown-legend i, .status-legend-global i {
          width: 10px; height: 10px; border-radius: 2px; display: inline-block;
        }
        .status-legend-global {
          display: flex; flex-wrap: wrap; gap: 1rem; margin-top: 0.5rem;
          padding-top: 0.75rem; border-top: 1px solid var(--mad-border);
          font-size: 0.78rem; color: var(--mad-text-muted);
        }
        .tradeoff-table td:first-child { min-width: 200px; }
        .req-title { display: block; margin-top: 0.15rem; font-weight: 500; }
        .disagree-badge {
          display: inline-block; margin-top: 0.25rem; font-size: 0.68rem;
          background: rgba(0, 180, 216, 0.15); color: var(--mad-cyan-dim);
          padding: 0.1rem 0.4rem; border-radius: 4px;
        }
        .status-pill {
          display: inline-block; font-size: 0.72rem; font-weight: 700;
          padding: 0.2rem 0.45rem; border-radius: 4px; white-space: nowrap;
        }
        .status-pill.status-compliant { background: rgba(40,167,69,0.15); color: var(--mad-compliant); }
        .status-pill.status-partial { background: rgba(230,168,0,0.15); color: var(--mad-partial); }
        .status-pill.status-non_compliant { background: rgba(220,53,69,0.12); color: var(--mad-gap); }
        .status-pill.status-untested { background: #eceff1; color: var(--mad-text-muted); }
        .strength-list { list-style: none; padding: 0; margin: 0; }
        .strength-list li {
          padding: 0.65rem 0; border-bottom: 1px solid #e8eaed;
          display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.35rem 0.75rem;
        }
        .strength-badge {
          font-size: 0.78rem; font-weight: 600; color: var(--mad-compliant);
          background: rgba(40,167,69,0.1); padding: 0.15rem 0.5rem; border-radius: 4px;
        }
      `}</style>
    </section>
  );
}
