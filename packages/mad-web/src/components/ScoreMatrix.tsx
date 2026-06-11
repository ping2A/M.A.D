import { Fragment, useMemo, useState } from "react";
import { ComplianceStatusBadge } from "./ComplianceStatusBadge";
import { MatrixStatusCell } from "./MatrixStatusCell";
import { RequirementDisplay } from "./RequirementDisplay";
import { useLocale } from "../i18n/LocaleContext";
import type {
  ComplianceStatus,
  EvaluationReport,
  Pillar,
  PillarId,
  RequirementSeverity,
} from "../types";
import { scoreColor } from "../utils/scoring";

interface ScoreMatrixProps {
  pillars: Pillar[];
  evaluation: EvaluationReport;
  onSetStatus: (
    vendorId: string,
    requirementId: string,
    status: ComplianceStatus,
    notes?: string | null,
  ) => void;
  onCycle: (vendorId: string, requirementId: string) => void;
  saving?: boolean;
}

type QuickFilter = "all" | "untested" | "gaps";

interface FlatRequirement {
  id: string;
  title: string;
  description: string;
  severity: RequirementSeverity;
  pillarId: PillarId;
  platforms: string[];
}

const LEGEND_STATUSES: ComplianceStatus[] = [
  "compliant",
  "partial",
  "non_compliant",
  "untested",
];

function getAssessment(
  evaluation: EvaluationReport,
  vendorId: string,
  requirementId: string,
): { status: ComplianceStatus; notes: string | null } {
  const vendor = evaluation.vendors.find((v) => v.vendor.id === vendorId);
  if (!vendor) return { status: "untested", notes: null };
  for (const pillar of vendor.pillars) {
    const req = pillar.requirements.find((r) => r.requirement_id === requirementId);
    if (req) return { status: req.status, notes: req.notes };
  }
  return { status: "untested", notes: null };
}

export function ScoreMatrix({
  pillars,
  evaluation,
  onSetStatus,
  onCycle,
  saving,
}: ScoreMatrixProps) {
  const { t, format, pillarLabel, statusLabel, severityLabel } = useLocale();
  const vendors = evaluation.vendors.map((v) => v.vendor);
  const [filterPillar, setFilterPillar] = useState<PillarId | "all">("all");
  const [filterSeverity, setFilterSeverity] = useState<RequirementSeverity | "all">("all");
  const [quickFilter, setQuickFilter] = useState<QuickFilter>("all");
  const [search, setSearch] = useState("");

  const pillarName = (id: PillarId) => {
    const pillar = pillars.find((p) => p.id === id);
    const label = pillarLabel(id);
    return label !== id ? label : pillar?.name ?? id;
  };

  const allRequirements = useMemo<FlatRequirement[]>(
    () =>
      pillars.flatMap((p) =>
        p.requirements.map((r) => ({
          id: r.id,
          title: r.title,
          description: r.description,
          severity: r.severity,
          pillarId: p.id,
          platforms: r.platforms,
        })),
      ),
    [pillars],
  );

  const vendorStatusesForReq = (reqId: string) =>
    vendors.map((v) => ({
      name: v.name,
      status: getAssessment(evaluation, v.id, reqId).status,
    }));

  const requirements = useMemo(() => {
    const q = search.trim().toLowerCase();
    let reqs = allRequirements;
    if (filterPillar !== "all") {
      reqs = reqs.filter((r) => r.pillarId === filterPillar);
    }
    if (filterSeverity !== "all") {
      reqs = reqs.filter((r) => r.severity === filterSeverity);
    }
    if (q) {
      reqs = reqs.filter(
        (r) =>
          r.id.toLowerCase().includes(q) ||
          r.title.toLowerCase().includes(q),
      );
    }
    if (quickFilter === "untested") {
      reqs = reqs.filter((r) =>
        vendors.some((v) => getAssessment(evaluation, v.id, r.id).status === "untested"),
      );
    } else if (quickFilter === "gaps") {
      reqs = reqs.filter((r) =>
        vendors.some((v) => {
          const s = getAssessment(evaluation, v.id, r.id).status;
          return s === "non_compliant" || s === "partial" || s === "untested";
        }),
      );
    }
    return reqs;
  }, [
    allRequirements,
    filterPillar,
    filterSeverity,
    search,
    quickFilter,
    vendors,
    evaluation,
  ]);

  const groupedRequirements = useMemo(() => {
    if (filterPillar !== "all") {
      return [{ pillarId: filterPillar, requirements: requirements }];
    }
    const order: PillarId[] = [];
    const map = new Map<PillarId, FlatRequirement[]>();
    for (const r of requirements) {
      if (!map.has(r.pillarId)) {
        map.set(r.pillarId, []);
        order.push(r.pillarId);
      }
      map.get(r.pillarId)!.push(r);
    }
    return order.map((pillarId) => ({
      pillarId,
      requirements: map.get(pillarId)!,
    }));
  }, [requirements, filterPillar]);

  const matrixStats = useMemo(() => {
    const reqIds = requirements.map((r) => r.id);
    let totalCells = 0;
    let untestedCells = 0;
    let gapCells = 0;
    for (const v of vendors) {
      for (const id of reqIds) {
        totalCells += 1;
        const s = getAssessment(evaluation, v.id, id).status;
        if (s === "untested") untestedCells += 1;
        if (s === "non_compliant" || s === "partial") gapCells += 1;
      }
    }
    return { totalCells, untestedCells, gapCells };
  }, [requirements, vendors, evaluation]);

  const vendorStats = useMemo(() => {
    const reqIds = requirements.map((r) => r.id);
    return vendors.map((v) => {
      const result = evaluation.vendors.find((e) => e.vendor.id === v.id);
      const score = result?.overall_score.overall_score_percent ?? 0;
      let scored = 0;
      let untested = 0;
      for (const id of reqIds) {
        const s = getAssessment(evaluation, v.id, id).status;
        if (s === "untested") untested += 1;
        else scored += 1;
      }
      const total = reqIds.length;
      const pct = total > 0 ? (scored / total) * 100 : 0;
      return { vendor: v, score, scored, untested, total, pct };
    });
  }, [vendors, requirements, evaluation]);

  const showPillarInRow = filterPillar === "all";
  const colSpan = 1 + vendors.length;

  const hasActiveFilters =
    filterPillar !== "all" ||
    filterSeverity !== "all" ||
    quickFilter !== "all" ||
    search.trim() !== "";

  const resetFilters = () => {
    setFilterPillar("all");
    setFilterSeverity("all");
    setQuickFilter("all");
    setSearch("");
  };

  if (vendors.length === 0) {
    return (
      <section className="score-matrix">
        <h2 className="section-title">{t.matrix.title}</h2>
        <div className="empty-state card">
          <p className="empty-state-title">{format(t.matrix.empty, { tab: t.tabs.vendors })}</p>
        </div>
      </section>
    );
  }

  return (
    <section className="score-matrix">
      <div className="toolbar">
        <div>
          <h2 className="section-title">{t.matrix.title}</h2>
          <p className="section-intro">{t.matrix.intro}</p>
          <p className="matrix-usage-hint">{t.matrix.usageHint}</p>
        </div>
        {saving && (
          <span className="saving-indicator">
            <span className="saving-dot" /> {t.matrix.saving}
          </span>
        )}
      </div>

      <div className="matrix-summary card">
        <div className="matrix-summary-stats">
          <span>
            {format(t.matrix.showingRequirements, { count: requirements.length })}
          </span>
          <span className="matrix-stat-muted">
            {format(t.matrix.cellStats, {
              total: matrixStats.totalCells,
              untested: matrixStats.untestedCells,
              gaps: matrixStats.gapCells,
            })}
          </span>
        </div>
        <div className="matrix-legend">
          {LEGEND_STATUSES.map((s) => (
            <ComplianceStatusBadge
              key={s}
              status={s}
              label={statusLabel(s)}
              variant="legend"
            />
          ))}
        </div>
      </div>

      <div className="matrix-filter-panel card">
        <div className="matrix-filter-top">
          <input
            type="search"
            className="matrix-search"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder={t.matrix.searchPlaceholder}
          />
          {hasActiveFilters && (
            <button type="button" className="btn btn-ghost btn-sm" onClick={resetFilters}>
              {t.matrix.resetFilters}
            </button>
          )}
        </div>

        <div className="matrix-filter-controls">
          <div className="matrix-filter-field">
            <label htmlFor="matrix-filter-group">{t.matrix.filterGroup}</label>
            <select
              id="matrix-filter-group"
              value={filterPillar}
              onChange={(e) => setFilterPillar(e.target.value as PillarId | "all")}
            >
              <option value="all">{t.matrix.allPillars}</option>
              {pillars.map((p) => (
                <option key={p.id} value={p.id}>
                  {pillarName(p.id)}
                </option>
              ))}
            </select>
          </div>

          <div className="matrix-filter-field">
            <label htmlFor="matrix-filter-severity">{t.matrix.filterSeverity}</label>
            <select
              id="matrix-filter-severity"
              value={filterSeverity}
              onChange={(e) =>
                setFilterSeverity(e.target.value as RequirementSeverity | "all")
              }
            >
              <option value="all">{t.matrix.allSeverities}</option>
              <option value="critical">{severityLabel("critical")}</option>
              <option value="high">{severityLabel("high")}</option>
              <option value="medium">{severityLabel("medium")}</option>
            </select>
          </div>

          <div className="matrix-filter-field matrix-filter-field-wide">
            <span className="matrix-field-label" id="matrix-filter-show-label">
              {t.matrix.filterShow}
            </span>
            <div
              className="segmented-control"
              role="group"
              aria-labelledby="matrix-filter-show-label"
            >
              {(
                [
                  ["all", t.matrix.filterAll],
                  ["untested", t.matrix.filterUntested],
                  ["gaps", t.matrix.filterGaps],
                ] as const
              ).map(([value, label]) => (
                <button
                  key={value}
                  type="button"
                  className={`segmented-option ${quickFilter === value ? "active" : ""}`}
                  onClick={() => setQuickFilter(value)}
                  aria-pressed={quickFilter === value}
                >
                  {label}
                </button>
              ))}
            </div>
          </div>
        </div>
      </div>

      {requirements.length === 0 ? (
        <div className="empty-state card">
          <p className="empty-state-title">{t.matrix.noResults}</p>
          <p className="empty-state-hint">{t.matrix.noResultsHint}</p>
        </div>
      ) : (
        <div className="matrix-scroll">
          <table className="matrix-table">
            <thead>
              <tr>
                <th className="matrix-sticky-col matrix-req-header">{t.matrix.criterion}</th>
                {vendorStats.map(({ vendor, score, scored, total, pct }) => (
                  <th key={vendor.id} className="matrix-vendor-header">
                    <span className="matrix-vendor-name">{vendor.name}</span>
                    <span
                      className="matrix-vendor-score"
                      style={{ color: scoreColor(score) }}
                    >
                      {score.toFixed(0)}%
                    </span>
                    <span className="matrix-vendor-progress-label">
                      {format(t.matrix.scoredCount, { scored, total })}
                    </span>
                    <div className="matrix-vendor-progress" aria-hidden>
                      <div
                        className="matrix-vendor-progress-fill"
                        style={{ width: `${pct}%` }}
                      />
                    </div>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {groupedRequirements.map((group) => (
                <Fragment key={group.pillarId}>
                  {showPillarInRow && groupedRequirements.length > 1 && (
                    <tr className="matrix-pillar-row">
                      <td colSpan={colSpan}>{pillarName(group.pillarId)}</td>
                    </tr>
                  )}
                  {group.requirements.map((req) => (
                    <tr key={req.id}>
                      <td className="matrix-sticky-col matrix-req-cell">
                        <RequirementDisplay
                          id={req.id}
                          title={req.title}
                          description={req.description}
                          severity={req.severity}
                          platforms={req.platforms}
                          pillarName={
                            showPillarInRow && groupedRequirements.length === 1
                              ? pillarName(req.pillarId)
                              : undefined
                          }
                          severityLabel={severityLabel}
                          statusLabel={statusLabel}
                          vendorStatuses={vendorStatusesForReq(req.id)}
                          expandLabel={t.matrix.expandRequirement}
                          collapseLabel={t.matrix.collapseRequirement}
                          variant="matrix"
                        />
                      </td>
                      {vendors.map((v) => {
                        const { status, notes } = getAssessment(evaluation, v.id, req.id);
                        return (
                          <td key={v.id} className="matrix-status-td">
                            <MatrixStatusCell
                              status={status}
                              notes={notes}
                              statusLabel={statusLabel}
                              cycleTitle={t.matrix.cycleStatus}
                              pickTitle={t.matrix.pickStatus}
                              addNotesTitle={t.matrix.addNotes}
                              notesTitle={t.matrix.notesTitle}
                              notesPlaceholder={t.matrix.notesPlaceholder}
                              saveLabel={t.common.save}
                              onCycle={() => onCycle(v.id, req.id)}
                              onSetStatus={(next, n) => onSetStatus(v.id, req.id, next, n)}
                            />
                          </td>
                        );
                      })}
                    </tr>
                  ))}
                </Fragment>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
