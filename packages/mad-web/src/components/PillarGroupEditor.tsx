import { useState } from "react";
import { useLocale } from "../i18n/LocaleContext";
import { BUILTIN_PILLAR_IDS, type Pillar } from "../types";
import { slugifyPillarId } from "../utils/suggestRequirementId";

interface PillarGroupEditorProps {
  pillars: Pillar[];
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

  const openEdit = (pillar: Pillar) => {
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
      }
      resetForm();
    } finally {
      setSaving(false);
    }
  };

  const displayName = (pillar: Pillar) =>
    pillarLabel(pillar.id) !== pillar.id ? pillarLabel(pillar.id) : pillar.name;

  return (
    <section className="pillar-group-editor card">
      <div className="toolbar">
        <div>
          <h2 className="section-title">{t.pillarGroups.title}</h2>
          <p className="section-intro">{t.pillarGroups.intro}</p>
        </div>
        {mode === "closed" && (
          <button type="button" className="btn btn-secondary" onClick={openAdd}>
            {t.pillarGroups.add}
          </button>
        )}
      </div>

      {(mode === "add" || mode === "edit") && (
        <form className="form-panel" onSubmit={submit}>
          <h3>{mode === "edit" ? t.pillarGroups.edit : t.pillarGroups.new}</h3>
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
            </label>
          )}
          {mode === "edit" && (
            <p className="readonly-id">
              <code>{form.id}</code>
            </p>
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

      <div className="pillar-group-list">
        {pillars.map((pillar) => (
          <div key={pillar.id} className="pillar-group-row">
            <div>
              <strong>{displayName(pillar)}</strong>
              <code className="pillar-id">{pillar.id}</code>
              {isBuiltin(pillar.id) && (
                <span className="badge builtin">{t.pillarGroups.builtinLocked}</span>
              )}
              {pillar.description && <p className="desc">{pillar.description}</p>}
              <span className="meta">
                {format(t.pillarGroups.requirementCount, {
                  count: pillar.requirements.length,
                })}
              </span>
            </div>
            <div className="row-actions">
              <button
                type="button"
                className="btn btn-ghost btn-icon"
                onClick={() => openEdit(pillar)}
                title={t.common.edit}
              >
                ✎
              </button>
              {!isBuiltin(pillar.id) && (
                <button
                  type="button"
                  className="btn btn-danger btn-icon"
                  onClick={() => {
                    if (
                      confirm(
                        format(t.pillarGroups.confirmRemove, { id: pillar.id }),
                      )
                    ) {
                      onDelete(pillar.id);
                    }
                  }}
                  title={t.common.remove}
                >
                  ×
                </button>
              )}
            </div>
          </div>
        ))}
      </div>

      <style>{`
        .pillar-group-editor { margin-bottom: 1.5rem; }
        .pillar-group-list { display: flex; flex-direction: column; gap: 0.6rem; }
        .pillar-group-row {
          display: flex; justify-content: space-between; align-items: flex-start;
          padding: 0.75rem; border: 1px solid var(--mad-border); border-radius: 8px;
        }
        .pillar-id { margin-left: 0.5rem; font-size: 0.75rem; color: var(--mad-text-muted); }
        .badge.builtin {
          margin-left: 0.5rem; font-size: 0.7rem; padding: 0.1rem 0.4rem;
          background: var(--mad-surface-2); border-radius: 4px;
        }
        .desc { margin: 0.25rem 0; font-size: 0.85rem; color: var(--mad-text-muted); }
        .meta { font-size: 0.8rem; color: var(--mad-text-muted); }
        .readonly-id { margin: 0 0 0.75rem; }
        .row-actions { display: flex; gap: 0.25rem; flex-shrink: 0; }
      `}</style>
    </section>
  );
}
