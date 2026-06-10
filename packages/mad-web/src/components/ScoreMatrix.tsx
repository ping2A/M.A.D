import { useMemo, useState } from "react";
import { useLocale } from "../i18n/LocaleContext";
import type { ComplianceStatus, EvaluationReport, Pillar, PillarId, RequirementSeverity } from "../types";

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

const ALL_STATUSES: ComplianceStatus[] = [
  "untested",
  "compliant",
  "partial",
  "non_compliant",
];

function statusClass(s: ComplianceStatus): string {
  return `cell status-${s}`;
}

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
  const [filterSeverity, setFilterSeverity] = useState<string>("all");
  const [expandedNotes, setExpandedNotes] = useState<string | null>(null);
  const [noteDraft, setNoteDraft] = useState("");

  const requirements = useMemo(() => {
    let reqs = pillars.flatMap((p) =>
      p.requirements.map((r) => ({ ...r, pillarId: p.id })),
    );
    if (filterPillar !== "all") {
      reqs = reqs.filter((r) => r.pillarId === filterPillar);
    }
    if (filterSeverity !== "all") {
      reqs = reqs.filter((r) => r.severity === filterSeverity);
    }
    return reqs;
  }, [pillars, filterPillar, filterSeverity]);

  const openNotes = (vendorId: string, requirementId: string) => {
    const key = `${vendorId}:${requirementId}`;
    const { notes } = getAssessment(evaluation, vendorId, requirementId);
    setExpandedNotes(key);
    setNoteDraft(notes ?? "");
  };

  const saveNotes = (vendorId: string, requirementId: string) => {
    const { status } = getAssessment(evaluation, vendorId, requirementId);
    onSetStatus(vendorId, requirementId, status, noteDraft.trim() || null);
    setExpandedNotes(null);
  };

  if (vendors.length === 0) {
    return (
      <section className="score-matrix">
        <h2 className="section-title">{t.matrix.title}</h2>
        <div className="empty-state card">
          <p>{format(t.matrix.empty, { tab: t.tabs.vendors })}</p>
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
        </div>
        {saving && (
          <span className="saving-indicator">
            <span className="saving-dot" /> {t.matrix.saving}
          </span>
        )}
      </div>

      <div className="matrix-controls">
        <div className="legend">
          {ALL_STATUSES.map((s) => (
            <span key={s} className={statusClass(s)}>
              {statusLabel(s)}
            </span>
          ))}
        </div>
        <div className="filters">
          <select
            value={filterPillar}
            onChange={(e) => setFilterPillar(e.target.value as PillarId | "all")}
          >
            <option value="all">{t.matrix.allPillars}</option>
            {pillars.map((p) => (
              <option key={p.id} value={p.id}>{pillarLabel(p.id) !== p.id ? pillarLabel(p.id) : p.name}</option>
            ))}
          </select>
          <select
            value={filterSeverity}
            onChange={(e) => setFilterSeverity(e.target.value)}
          >
            <option value="all">{t.matrix.allSeverities}</option>
            <option value="critical">{severityLabel("critical")}</option>
            <option value="high">{severityLabel("high")}</option>
            <option value="medium">{severityLabel("medium")}</option>
          </select>
        </div>
      </div>

      <div className="matrix-scroll">
        <table className="matrix">
          <thead>
            <tr>
              <th className="sticky-col">{t.matrix.criterion}</th>
              <th className="sticky-col narrow">{t.matrix.sev}</th>
              {vendors.map((v) => {
                const result = evaluation.vendors.find((e) => e.vendor.id === v.id);
                const score = result?.overall_score.overall_score_percent ?? 0;
                return (
                  <th key={v.id} className="vendor-col">
                    <span className="vendor-name">{v.name}</span>
                    <span className="vendor-score">{score.toFixed(0)}%</span>
                  </th>
                );
              })}
            </tr>
          </thead>
          <tbody>
            {requirements.map((req) => (
              <tr key={req.id}>
                <td className="sticky-col req-cell">
                  <code>{req.id}</code>
                  <span className="req-title">{req.title}</span>
                  <span className="req-pillar">{pillarLabel(req.pillarId)}</span>
                </td>
                <td className="sticky-col narrow">
                  <span className={`badge sev-${req.severity}`}>
                    {severityLabel(req.severity as RequirementSeverity)}
                  </span>
                </td>
                {vendors.map((v) => {
                  const { status, notes } = getAssessment(evaluation, v.id, req.id);
                  const noteKey = `${v.id}:${req.id}`;
                  const notesOpen = expandedNotes === noteKey;
                  return (
                    <td key={v.id} className="status-cell">
                      <select
                        className={statusClass(status)}
                        value={status}
                        onChange={(e) =>
                          onSetStatus(v.id, req.id, e.target.value as ComplianceStatus, notes)
                        }
                        title={statusLabel(status)}
                      >
                        {ALL_STATUSES.map((s) => (
                          <option key={s} value={s}>
                            {statusLabel(s)}
                          </option>
                        ))}
                      </select>
                      <div className="cell-actions">
                        <button
                          type="button"
                          className="cell-btn"
                          onClick={() => onCycle(v.id, req.id)}
                          title={t.matrix.cycleStatus}
                        >
                          ↻
                        </button>
                        <button
                          type="button"
                          className={`cell-btn ${notes ? "has-notes" : ""}`}
                          onClick={() =>
                            notesOpen ? setExpandedNotes(null) : openNotes(v.id, req.id)
                          }
                          title={
                            notes
                              ? format(t.matrix.notesTitle, { notes })
                              : t.matrix.addNotes
                          }
                        >
                          ✎
                        </button>
                      </div>
                      {notesOpen && (
                        <div className="notes-popover">
                          <textarea
                            value={noteDraft}
                            onChange={(e) => setNoteDraft(e.target.value)}
                            rows={2}
                            placeholder={t.matrix.notesPlaceholder}
                          />
                          <button
                            type="button"
                            className="btn btn-primary"
                            onClick={() => saveNotes(v.id, req.id)}
                          >
                            {t.common.save}
                          </button>
                        </div>
                      )}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <style>{`
        .matrix-controls {
          display: flex;
          justify-content: space-between;
          align-items: center;
          gap: 1rem;
          flex-wrap: wrap;
          margin-bottom: 1rem;
        }
        .legend { display: flex; gap: 0.5rem; flex-wrap: wrap; }
        .legend span {
          font-size: 0.72rem; font-weight: 600; padding: 0.2rem 0.5rem; border-radius: 4px;
        }
        .filters { display: flex; gap: 0.5rem; }
        .filters select {
          padding: 0.4rem 0.6rem; border: 1px solid var(--mad-border);
          border-radius: 6px; font-size: 0.82rem; font-family: inherit;
        }
        .matrix-scroll {
          overflow: auto;
          background: white;
          border-radius: var(--mad-radius);
          box-shadow: var(--mad-shadow);
          max-height: 70vh;
          border: 1px solid var(--mad-border);
        }
        .matrix { border-collapse: collapse; font-size: 0.8rem; min-width: 100%; }
        .matrix th, .matrix td {
          padding: 0.45rem 0.5rem;
          border: 1px solid #e8eaed;
          text-align: center;
          vertical-align: top;
        }
        .matrix th {
          background: var(--mad-navy-light);
          color: white;
          position: sticky;
          top: 0;
          z-index: 2;
        }
        .sticky-col {
          position: sticky;
          left: 0;
          z-index: 1;
          background: white;
          text-align: left !important;
          max-width: 220px;
        }
        .sticky-col.narrow { max-width: 70px; }
        .matrix thead .sticky-col { background: var(--mad-navy-light); z-index: 3; }
        .req-cell { display: flex; flex-direction: column; gap: 0.1rem; }
        .req-cell code { font-size: 0.68rem; color: var(--mad-cyan-dim); }
        .req-title { font-weight: 600; font-size: 0.8rem; line-height: 1.3; }
        .req-pillar { font-size: 0.68rem; color: var(--mad-text-muted); }
        .vendor-col { min-width: 110px; }
        .vendor-name { display: block; font-size: 0.72rem; }
        .vendor-score {
          display: block;
          font-family: var(--font-mono);
          color: var(--mad-cyan);
          font-size: 0.9rem;
          font-weight: 700;
        }
        .status-cell { position: relative; min-width: 100px; }
        .status-cell select {
          width: 100%;
          padding: 0.35rem 0.25rem;
          border: none;
          border-radius: 4px;
          font-weight: 700;
          font-size: 0.7rem;
          cursor: pointer;
          font-family: inherit;
        }
        .cell-actions { display: flex; gap: 0.15rem; justify-content: center; margin-top: 0.2rem; }
        .cell-btn {
          background: #f0f2f5;
          border: none;
          border-radius: 3px;
          width: 22px;
          height: 22px;
          font-size: 0.7rem;
          cursor: pointer;
          color: var(--mad-text-muted);
        }
        .cell-btn:hover { background: #e0e4e8; color: var(--mad-navy); }
        .cell-btn.has-notes { color: var(--mad-cyan-dim); font-weight: 700; }
        .notes-popover {
          position: absolute;
          top: 100%;
          left: 50%;
          transform: translateX(-50%);
          z-index: 10;
          background: white;
          border: 1px solid var(--mad-border);
          border-radius: 8px;
          padding: 0.5rem;
          box-shadow: 0 4px 16px rgba(0,0,0,0.12);
          width: 180px;
          display: flex;
          flex-direction: column;
          gap: 0.35rem;
        }
        .notes-popover textarea {
          font-family: inherit;
          font-size: 0.78rem;
          padding: 0.35rem;
          border: 1px solid var(--mad-border);
          border-radius: 4px;
          resize: vertical;
        }
        .notes-popover .btn { padding: 0.3rem 0.5rem; font-size: 0.75rem; }
        .empty-state { padding: 2rem; text-align: center; color: var(--mad-text-muted); }
      `}</style>
    </section>
  );
}
