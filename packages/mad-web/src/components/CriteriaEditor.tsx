import { useMemo, useState } from "react";
import { usePolicyContent } from "../hooks/usePolicyContent";
import { useLocale } from "../i18n/LocaleContext";
import type { Pillar, PillarId, Requirement, RequirementSeverity } from "../types";
import { RequirementDisplay } from "./RequirementDisplay";
import { suggestNextRequirementId } from "../utils/suggestRequirementId";
import { formatTagsInput, parseTagsInput } from "../utils/comparisonFilter";

interface CriteriaEditorProps {
  pillars: Pillar[];
  filterPillar: PillarId | "all";
  onFilterPillarChange: (id: PillarId | "all") => void;
  onAdd: (pillarId: PillarId, requirement: Requirement) => Promise<void>;
  onUpdate: (id: string, pillarId: PillarId, requirement: Requirement) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

const DEFAULT_PILLAR: PillarId = "cybersecurity_dlp";

const emptyForm = (pillars: Pillar[], preferredPillar?: PillarId | "all") => {
  const pillarId =
    preferredPillar && preferredPillar !== "all"
      ? preferredPillar
      : pillars[0]?.id ?? DEFAULT_PILLAR;
  return {
    pillarId,
    id: suggestNextRequirementId(pillarId, pillars),
    title: "",
    description: "",
    severity: "high" as RequirementSeverity,
    platforms: "ios, android",
    tagsInput: "",
    evaluationMethod: "",
    technicalCriteria: "",
  };
};

export function CriteriaEditor({
  pillars,
  filterPillar,
  onFilterPillarChange,
  onAdd,
  onUpdate,
  onDelete,
}: CriteriaEditorProps) {
  const { t, format, pillarLabel, severityLabel } = useLocale();
  const { localizePillars } = usePolicyContent();
  const displayPillars = useMemo(() => localizePillars(pillars), [pillars, localizePillars]);
  const pillarIds = useMemo(() => pillars.map((p) => p.id), [pillars]);
  const [mode, setMode] = useState<"closed" | "add" | "edit">("closed");
  const [editId, setEditId] = useState<string | null>(null);
  const [form, setForm] = useState(() => emptyForm(pillars, filterPillar));
  const [saving, setSaving] = useState(false);

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

  const filtered =
    filterPillar === "all"
      ? allRequirements
      : allRequirements.filter((r) => r.pillarId === filterPillar);

  const showPillarColumn = filterPillar === "all";

  const resetForm = () => {
    setForm(emptyForm(pillars, filterPillar));
    setEditId(null);
    setMode("closed");
  };

  const openAdd = () => {
    setForm(emptyForm(pillars, filterPillar));
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
      tagsInput: formatTagsInput(canonical.tags),
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
        tags: parseTagsInput(form.tagsInput),
        evaluation_method: form.evaluationMethod.trim() || undefined,
        technical_criteria: form.technicalCriteria.trim() || undefined,
      };
      if (mode === "edit" && editId) {
        await onUpdate(editId, form.pillarId, requirement);
      } else {
        await onAdd(form.pillarId, requirement);
        onFilterPillarChange(form.pillarId);
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

  const countForPillar = (pid: PillarId | "all") =>
    pid === "all"
      ? allRequirements.length
      : allRequirements.filter((r) => r.pillarId === pid).length;

  return (
    <div className="criteria-editor">
      <div className="criteria-editor-toolbar">
        <div className="filter-chips" role="group" aria-label={t.common.filterByPillar}>
          <button
            type="button"
            className={`filter-chip ${filterPillar === "all" ? "active" : ""}`}
            onClick={() => onFilterPillarChange("all")}
          >
            {t.criteria.allPillars}
            <span className="chip-count">{countForPillar("all")}</span>
          </button>
          {pillarIds.map((pid) => (
            <button
              key={pid}
              type="button"
              className={`filter-chip ${filterPillar === pid ? "active" : ""}`}
              onClick={() => onFilterPillarChange(pid)}
            >
              {pillarName(pid)}
              <span className="chip-count">{countForPillar(pid)}</span>
            </button>
          ))}
        </div>
        {mode === "closed" && (
          <button type="button" className="btn btn-primary" onClick={openAdd}>
            {t.criteria.add}
          </button>
        )}
      </div>

      {(mode === "add" || mode === "edit") && (
        <form className="form-panel criteria-form-panel" onSubmit={submit}>
          <div className="form-panel-heading">
            <h4>{mode === "edit" ? t.criteria.edit : t.criteria.new}</h4>
            <p className="form-panel-hint">
              {mode === "add" ? t.criteria.addHint : t.criteria.editHint}
            </p>
          </div>
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
              {mode === "add" && (
                <span className="field-hint">{t.criteria.idHint}</span>
              )}
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
            <label>
              {t.criteria.tags}
              <input
                value={form.tagsInput}
                onChange={(e) => setField("tagsInput", e.target.value)}
                placeholder={t.criteria.placeholderTags}
              />
              <span className="field-hint">{t.criteria.tagsHint}</span>
            </label>
          </div>
          <details className="form-advanced">
            <summary>{t.criteria.advancedFields}</summary>
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
          </details>
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

      {filtered.length === 0 ? (
        <div className="empty-state card">
          <p className="empty-state-title">{t.criteriaPage.noRequirements}</p>
          <p className="empty-state-hint">{t.criteriaPage.noRequirementsHint}</p>
          {mode === "closed" && (
            <button type="button" className="btn btn-primary" onClick={openAdd}>
              {t.criteria.add}
            </button>
          )}
        </div>
      ) : (
        <div className="data-table-wrap">
          <table className="data-table">
            <thead>
              <tr>
                <th>{t.matrix.criterion}</th>
                {showPillarColumn && <th>{t.common.pillar}</th>}
                <th>{t.common.actions}</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((req) => (
                <tr key={req.id}>
                  <td className="requirement-table-cell">
                        <RequirementDisplay
                          id={req.id}
                          title={req.title}
                          description={req.description}
                          severity={req.severity}
                          platforms={req.platforms}
                          tags={canonicalById.get(req.id)?.tags ?? req.tags}
                          severityLabel={severityLabel}
                          variant="table"
                        />
                  </td>
                  {showPillarColumn && (
                    <td className="pillar-name-cell">{pillarName(req.pillarId)}</td>
                  )}
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
      )}
    </div>
  );
}
