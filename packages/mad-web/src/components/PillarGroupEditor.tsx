import { useState } from "react";
import { useLocale } from "../i18n/LocaleContext";
import { BUILTIN_PILLAR_IDS, type Pillar, type PillarId } from "../types";
import { pillarIcon } from "../utils/pillarIcons";
import { slugifyPillarId } from "../utils/suggestRequirementId";

interface PillarGroupEditorProps {
  pillars: Pillar[];
  selectedPillar: PillarId | "all";
  onSelectPillar: (id: PillarId | "all") => void;
  onAdd: (id: string, name: string, description: string) => Promise<void>;
  onUpdate: (id: string, name: string, description: string) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

const EMPTY_FORM = { id: "", name: "", description: "" };

function isBuiltin(id: string): boolean {
  return (BUILTIN_PILLAR_IDS as readonly string[]).includes(id);
}

export function PillarGroupEditor({
  pillars,
  selectedPillar,
  onSelectPillar,
  onAdd,
  onUpdate,
  onDelete,
}: PillarGroupEditorProps) {
  const { t, format, pillarLabel } = useLocale();
  const [mode, setMode] = useState<"closed" | "add" | "edit">("closed");
  const [editId, setEditId] = useState<string | null>(null);
  const [form, setForm] = useState(EMPTY_FORM);
  const [saving, setSaving] = useState(false);

  const resetForm = () => {
    setForm(EMPTY_FORM);
    setEditId(null);
    setMode("closed");
  };

  const openAdd = () => {
    setForm(EMPTY_FORM);
    setEditId(null);
    setMode("add");
  };

  const openEdit = (pillar: Pillar, e: React.MouseEvent) => {
    e.stopPropagation();
    setEditId(pillar.id);
    setForm({
      id: pillar.id,
      name: pillar.name,
      description: pillar.description,
    });
    setMode("edit");
  };

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.name.trim()) return;
    if (mode === "add" && !form.id.trim()) return;
    setSaving(true);
    try {
      const name = form.name.trim();
      const description = form.description.trim();
      if (mode === "edit" && editId) {
        await onUpdate(editId, name, description);
      } else {
        const id = form.id.trim() || slugifyPillarId(name);
        await onAdd(id, name, description);
        onSelectPillar(id);
      }
      resetForm();
    } finally {
      setSaving(false);
    }
  };

  const displayName = (pillar: Pillar) =>
    pillarLabel(pillar.id) !== pillar.id ? pillarLabel(pillar.id) : pillar.name;

  const handleDelete = (pillar: Pillar, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm(format(t.pillarGroups.confirmRemove, { id: pillar.id }))) {
      onDelete(pillar.id);
      if (selectedPillar === pillar.id) {
        onSelectPillar("all");
      }
    }
  };

  return (
    <div className="pillar-group-editor">
      <div className="pillar-group-toolbar">
        {mode === "closed" && (
          <button type="button" className="btn btn-secondary" onClick={openAdd}>
            {t.pillarGroups.add}
          </button>
        )}
      </div>

      {(mode === "add" || mode === "edit") && (
        <form className="form-panel criteria-form-panel" onSubmit={submit}>
          <div className="form-panel-heading">
            <h4>{mode === "edit" ? t.pillarGroups.edit : t.pillarGroups.new}</h4>
            <p className="form-panel-hint">
              {mode === "add" ? t.pillarGroups.addHint : t.pillarGroups.editHint}
            </p>
          </div>
          {mode === "add" && (
            <label>
              {t.common.id}
              <input
                value={form.id}
                onChange={(e) => setForm((prev) => ({ ...prev, id: e.target.value }))}
                onBlur={() => {
                  if (!form.id.trim() && form.name.trim()) {
                    setForm((prev) => ({ ...prev, id: slugifyPillarId(prev.name) }));
                  }
                }}
                placeholder={t.pillarGroups.placeholderId}
                pattern="[a-z][a-z0-9_]*"
                required
              />
              <span className="field-hint">{t.pillarGroups.idHint}</span>
            </label>
          )}
          {mode === "edit" && (
            <div className="readonly-id-field">
              <span className="field-label">{t.common.id}</span>
              <code>{form.id}</code>
            </div>
          )}
          <label>
            {t.common.name}
            <input
              value={form.name}
              onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))}
              placeholder={t.pillarGroups.placeholderName}
              required
            />
          </label>
          <label>
            {t.common.description}
            <textarea
              value={form.description}
              onChange={(e) => setForm((prev) => ({ ...prev, description: e.target.value }))}
              rows={2}
              placeholder={t.pillarGroups.placeholderDescription}
            />
          </label>
          <div className="form-actions">
            <button type="submit" className="btn btn-primary" disabled={saving}>
              {saving
                ? t.common.saving
                : mode === "edit"
                  ? t.common.saveChanges
                  : t.pillarGroups.add}
            </button>
            <button type="button" className="btn btn-secondary" onClick={resetForm}>
              {t.common.cancel}
            </button>
          </div>
        </form>
      )}

      <div className="pillar-group-grid" role="list">
        <button
          type="button"
          role="listitem"
          className={`pillar-group-card ${selectedPillar === "all" ? "selected" : ""}`}
          onClick={() => onSelectPillar("all")}
        >
          <span className="pillar-group-icon">⊞</span>
          <span className="pillar-group-name">{t.criteria.allPillars}</span>
          <span className="pillar-group-meta">
            {format(t.pillarGroups.requirementCount, {
              count: pillars.reduce((n, p) => n + p.requirements.length, 0),
            })}
          </span>
        </button>

        {pillars.map((pillar) => {
          const critical = pillar.requirements.filter((r) => r.severity === "critical").length;
          return (
            <button
              key={pillar.id}
              type="button"
              role="listitem"
              className={`pillar-group-card ${selectedPillar === pillar.id ? "selected" : ""}`}
              onClick={() => onSelectPillar(pillar.id)}
            >
              <span className="pillar-group-icon">{pillarIcon(pillar.id)}</span>
              <span className="pillar-group-name">{displayName(pillar)}</span>
              <code className="pillar-group-id">{pillar.id}</code>
              {isBuiltin(pillar.id) && (
                <span className="badge badge-muted">{t.pillarGroups.builtinLocked}</span>
              )}
              <span className="pillar-group-meta">
                {format(t.pillarGroups.requirementCount, {
                  count: pillar.requirements.length,
                })}
                {critical > 0 && (
                  <span className="critical-count">
                    · {critical} {t.pillarCard.critical}
                  </span>
                )}
              </span>
              <div className="pillar-group-actions">
                <button
                  type="button"
                  className="btn btn-ghost btn-icon"
                  onClick={(e) => openEdit(pillar, e)}
                  title={t.common.edit}
                >
                  ✎
                </button>
                {!isBuiltin(pillar.id) && (
                  <button
                    type="button"
                    className="btn btn-danger btn-icon"
                    onClick={(e) => handleDelete(pillar, e)}
                    title={t.common.remove}
                  >
                    ×
                  </button>
                )}
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
