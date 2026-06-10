import { useMemo, useState } from "react";
import { usePolicyContent } from "../hooks/usePolicyContent";
import { useLocale } from "../i18n/LocaleContext";
import type { Pillar, PillarId, Requirement, RequirementSeverity } from "../types";
import { suggestNextRequirementId } from "../utils/suggestRequirementId";

interface CriteriaEditorProps {
  pillars: Pillar[];
  onAdd: (pillarId: PillarId, requirement: Requirement) => Promise<void>;
  onUpdate: (id: string, pillarId: PillarId, requirement: Requirement) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

const DEFAULT_PILLAR: PillarId = "cybersecurity_dlp";

const emptyForm = (pillars: Pillar[]) => ({
  pillarId: pillars[0]?.id ?? DEFAULT_PILLAR,
  id: suggestNextRequirementId(pillars[0]?.id ?? DEFAULT_PILLAR, pillars),
  title: "",
  description: "",
  severity: "high" as RequirementSeverity,
  platforms: "ios, android",
  evaluationMethod: "",
  technicalCriteria: "",
});

export function CriteriaEditor({ pillars, onAdd, onUpdate, onDelete }: CriteriaEditorProps) {
  const { t, format, pillarLabel, severityLabel } = useLocale();
  const { localizePillars } = usePolicyContent();
  const displayPillars = useMemo(() => localizePillars(pillars), [pillars, localizePillars]);
  const pillarIds = useMemo(() => pillars.map((p) => p.id), [pillars]);
  const [mode, setMode] = useState<"closed" | "add" | "edit">("closed");
  const [editId, setEditId] = useState<string | null>(null);
  const [form, setForm] = useState(() => emptyForm(pillars));
  const [saving, setSaving] = useState(false);
  const [filterPillar, setFilterPillar] = useState<PillarId | "all">("all");

  const pillarName = (id: PillarId) => {
    const localized = displayPillars.find((p) => p.id === id)?.name;
    return localized ?? pillarLabel(id);
  };

  const allRequirements = displayPillars.flatMap((p) =>
    p.requirements.map((r) => ({ ...r, pillarId: p.id, pillarName: p.name })),
  );

  const canonicalById = useMemo(() => {
    const map = new Map<string, Requirement & { pillarId: PillarId }>();
    for (const p of pillars) {
      for (const r of p.requirements) {
        map.set(r.id, { ...r, pillarId: p.id });
      }
    }
    return map;
  }, [pillars]);

  const filtered = filterPillar === "all"
    ? allRequirements
    : allRequirements.filter((r) => r.pillarId === filterPillar);

  const resetForm = () => {
    setForm(emptyForm(pillars));
    setEditId(null);
    setMode("closed");
  };

  const openAdd = () => {
    setForm(emptyForm(pillars));
    setEditId(null);
    setMode("add");
  };

  const openEdit = (req: Requirement & { pillarId: PillarId }) => {
    const canonical = canonicalById.get(req.id) ?? req;
    setEditId(canonical.id);
    setForm({
      pillarId: canonical.pillarId,
      id: canonical.id,
      title: canonical.title,
      description: canonical.description,
      severity: canonical.severity,
      platforms: canonical.platforms.join(", "),
      evaluationMethod: canonical.evaluation_method ?? "",
      technicalCriteria: canonical.technical_criteria ?? "",
    });
    setMode("edit");
  };

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.id.trim() || !form.title.trim()) return;
    setSaving(true);
    try {
      const requirement: Requirement = {
        id: form.id.trim(),
        title: form.title.trim(),
        description: form.description.trim(),
        severity: form.severity,
        platforms: form.platforms.split(",").map((s) => s.trim()).filter(Boolean),
        tags: [],
        evaluation_method: form.evaluationMethod.trim() || undefined,
        technical_criteria: form.technicalCriteria.trim() || undefined,
      };
      if (mode === "edit" && editId) {
        await onUpdate(editId, form.pillarId, requirement);
      } else {
        await onAdd(form.pillarId, requirement);
      }
      resetForm();
    } finally {
      setSaving(false);
    }
  };

  const setField = <K extends keyof typeof form>(key: K, value: (typeof form)[K]) => {
    setForm((prev) => {
      const next = { ...prev, [key]: value };
      if (key === "pillarId" && mode === "add") {
        next.id = suggestNextRequirementId(value as PillarId, pillars);
      }
      return next;
    });
  };

  const applySuggestedId = () => {
    setForm((prev) => ({
      ...prev,
      id: suggestNextRequirementId(prev.pillarId, pillars),
    }));
  };

  return (
    <section className="criteria-editor">
      <div className="toolbar">
        <div>
          <h2 className="section-title">{t.criteria.title}</h2>
          <p className="section-intro">{t.criteria.intro}</p>
        </div>
        {mode === "closed" && (
          <button type="button" className="btn btn-primary" onClick={openAdd}>
            {t.criteria.add}
          </button>
        )}
      </div>

      {(mode === "add" || mode === "edit") && (
        <form className="form-panel" onSubmit={submit}>
          <h3>{mode === "edit" ? t.criteria.edit : t.criteria.new}</h3>
          <div className="form-row">
            <label>
              {t.common.pillar}
              <select
                value={form.pillarId}
                onChange={(e) => setField("pillarId", e.target.value as PillarId)}
              >
                {pillarIds.map((pid) => (
                  <option key={pid} value={pid}>
                    {pillarName(pid)}
                  </option>
                ))}
              </select>
            </label>
            <label>
              {t.common.id}
              <div className="id-field">
                <input
                  value={form.id}
                  onChange={(e) => setField("id", e.target.value)}
                  placeholder={t.criteria.placeholderId}
                  required
                  disabled={mode === "edit"}
                />
                {mode === "add" && (
                  <button
                    type="button"
                    className="btn btn-ghost btn-sm"
                    onClick={applySuggestedId}
                  >
                    {t.criteria.suggestId}
                  </button>
                )}
              </div>
            </label>
            <label>
              {t.common.severity}
              <select
                value={form.severity}
                onChange={(e) => setField("severity", e.target.value as RequirementSeverity)}
              >
                <option value="critical">{severityLabel("critical")}</option>
                <option value="high">{severityLabel("high")}</option>
                <option value="medium">{severityLabel("medium")}</option>
              </select>
            </label>
          </div>
          <label>
            {t.common.title}
            <input
              value={form.title}
              onChange={(e) => setField("title", e.target.value)}
              placeholder={t.criteria.placeholderTitle}
              required
            />
          </label>
          <label>
            {t.common.description}
            <textarea
              value={form.description}
              onChange={(e) => setField("description", e.target.value)}
              rows={2}
              placeholder={t.criteria.placeholderDescription}
            />
          </label>
          <div className="form-row">
            <label>
              {t.common.platforms}
              <input
                value={form.platforms}
                onChange={(e) => setField("platforms", e.target.value)}
                placeholder={t.criteria.placeholderPlatforms}
              />
            </label>
          </div>
          <label>
            {t.criteria.evaluationMethod}
            <textarea
              value={form.evaluationMethod}
              onChange={(e) => setField("evaluationMethod", e.target.value)}
              rows={2}
              placeholder={t.criteria.placeholderEvaluation}
            />
          </label>
          <label>
            {t.criteria.technicalCriteria}
            <textarea
              value={form.technicalCriteria}
              onChange={(e) => setField("technicalCriteria", e.target.value)}
              rows={2}
              placeholder={t.criteria.placeholderTechnical}
            />
          </label>
          <div className="form-actions">
            <button type="submit" className="btn btn-primary" disabled={saving}>
              {saving
                ? t.common.saving
                : mode === "edit"
                  ? t.common.saveChanges
                  : t.criteria.add}
            </button>
            <button type="button" className="btn btn-secondary" onClick={resetForm}>
              {t.common.cancel}
            </button>
          </div>
        </form>
      )}

      <div className="filter-bar">
        <span>{t.common.filterByPillar}</span>
        <select
          value={filterPillar}
          onChange={(e) => setFilterPillar(e.target.value as PillarId | "all")}
        >
          <option value="all">
            {t.criteria.allPillars} ({allRequirements.length})
          </option>
          {pillarIds.map((pid) => (
            <option key={pid} value={pid}>
              {pillarName(pid)} (
              {allRequirements.filter((r) => r.pillarId === pid).length})
            </option>
          ))}
        </select>
      </div>

      <div className="data-table-wrap">
        <table className="data-table">
          <thead>
            <tr>
              <th>{t.common.id}</th>
              <th>{t.common.pillar}</th>
              <th>{t.common.title}</th>
              <th>{t.common.severity}</th>
              <th>{t.common.platforms}</th>
              <th>{t.common.actions}</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((req) => (
              <tr key={req.id}>
                <td><code>{req.id}</code></td>
                <td>{pillarName(req.pillarId)}</td>
                <td>
                  <strong>{req.title}</strong>
                  {req.description && <span className="desc">{req.description}</span>}
                </td>
                <td>
                  <span className={`badge sev-${req.severity}`}>
                    {severityLabel(req.severity)}
                  </span>
                </td>
                <td>{req.platforms.join(", ")}</td>
                <td>
                  <div className="row-actions">
                    <button
                      type="button"
                      className="btn btn-ghost btn-icon"
                      onClick={() => openEdit(req)}
                      title={t.common.edit}
                    >
                      ✎
                    </button>
                    <button
                      type="button"
                      className="btn btn-danger btn-icon"
                      onClick={() => {
                        if (confirm(format(t.criteria.confirmRemove, { id: req.id }))) {
                          onDelete(req.id);
                        }
                      }}
                      title={t.common.remove}
                    >
                      ×
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <style>{`
        .desc {
          display: block; font-size: 0.8rem; color: var(--mad-text-muted);
          font-weight: 400; margin-top: 0.2rem;
        }
        .filter-bar {
          display: flex; align-items: center; gap: 0.6rem;
          margin-bottom: 0.75rem; font-size: 0.85rem; color: var(--mad-text-muted);
        }
        .filter-bar select {
          padding: 0.4rem 0.6rem; border: 1px solid var(--mad-border);
          border-radius: 6px; font-family: inherit; font-size: 0.85rem;
        }
        .row-actions { display: flex; gap: 0.25rem; }
        .id-field { display: flex; gap: 0.35rem; align-items: center; }
        .id-field input { flex: 1; }
        .btn-sm { font-size: 0.75rem; padding: 0.25rem 0.5rem; white-space: nowrap; }
      `}</style>
    </section>
  );
}
