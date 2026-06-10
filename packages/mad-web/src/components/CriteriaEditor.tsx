import { useState } from "react";
import type { Pillar, PillarId, Requirement, RequirementSeverity } from "../types";
import { PILLAR_LABELS } from "../types";

interface CriteriaEditorProps {
  pillars: Pillar[];
  onAdd: (pillarId: PillarId, requirement: Requirement) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

const PILLAR_IDS: PillarId[] = ["cybersecurity_dlp", "dfir", "platform_os"];

export function CriteriaEditor({ pillars, onAdd, onDelete }: CriteriaEditorProps) {
  const [open, setOpen] = useState(false);
  const [pillarId, setPillarId] = useState<PillarId>("cybersecurity_dlp");
  const [id, setId] = useState("");
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [severity, setSeverity] = useState<RequirementSeverity>("high");
  const [platforms, setPlatforms] = useState("ios, android");
  const [evaluationMethod, setEvaluationMethod] = useState("");
  const [technicalCriteria, setTechnicalCriteria] = useState("");
  const [saving, setSaving] = useState(false);

  const allRequirements = pillars.flatMap((p) =>
    p.requirements.map((r) => ({ ...r, pillarId: p.id, pillarName: p.name })),
  );

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!id.trim() || !title.trim()) return;
    setSaving(true);
    try {
      await onAdd(pillarId, {
        id: id.trim(),
        title: title.trim(),
        description: description.trim(),
        severity,
        platforms: platforms.split(",").map((s) => s.trim()).filter(Boolean),
        tags: [],
        evaluation_method: evaluationMethod.trim() || undefined,
        technical_criteria: technicalCriteria.trim() || undefined,
      });
      setId("");
      setTitle("");
      setDescription("");
      setEvaluationMethod("");
      setTechnicalCriteria("");
      setOpen(false);
    } finally {
      setSaving(false);
    }
  };

  return (
    <section className="criteria-editor">
      <div className="toolbar">
        <div>
          <h2>Evaluation Criteria</h2>
          <p className="intro">
            Define what each MDM vendor must demonstrate. Criteria are weighted by
            severity (critical ×3, high ×2, medium ×1) in the overall score.
          </p>
        </div>
        <button type="button" className="btn-primary" onClick={() => setOpen(!open)}>
          {open ? "Cancel" : "+ Add Criterion"}
        </button>
      </div>

      {open && (
        <form className="add-form" onSubmit={submit}>
          <div className="form-row">
            <label>
              Pillar
              <select value={pillarId} onChange={(e) => setPillarId(e.target.value as PillarId)}>
                {PILLAR_IDS.map((pid) => (
                  <option key={pid} value={pid}>
                    {PILLAR_LABELS[pid]}
                  </option>
                ))}
              </select>
            </label>
            <label>
              ID
              <input
                value={id}
                onChange={(e) => setId(e.target.value)}
                placeholder="e.g. dlp-004"
                required
              />
            </label>
            <label>
              Severity
              <select
                value={severity}
                onChange={(e) => setSeverity(e.target.value as RequirementSeverity)}
              >
                <option value="critical">Critical</option>
                <option value="high">High</option>
                <option value="medium">Medium</option>
              </select>
            </label>
          </div>
          <label>
            Title
            <input
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Short requirement title"
              required
            />
          </label>
          <label>
            Description
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={2}
              placeholder="What the MDM must be capable of"
            />
          </label>
          <div className="form-row">
            <label>
              Platforms
              <input
                value={platforms}
                onChange={(e) => setPlatforms(e.target.value)}
                placeholder="ios, android"
              />
            </label>
          </div>
          <label>
            Evaluation method
            <textarea
              value={evaluationMethod}
              onChange={(e) => setEvaluationMethod(e.target.value)}
              rows={2}
              placeholder="How to test this in a lab"
            />
          </label>
          <label>
            Technical criteria
            <textarea
              value={technicalCriteria}
              onChange={(e) => setTechnicalCriteria(e.target.value)}
              rows={2}
              placeholder="APIs, protocols, MDM payloads"
            />
          </label>
          <button type="submit" className="btn-primary" disabled={saving}>
            {saving ? "Saving…" : "Add Criterion"}
          </button>
        </form>
      )}

      <div className="criteria-table-wrap">
        <table className="criteria-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>Pillar</th>
              <th>Title</th>
              <th>Severity</th>
              <th>Platforms</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {allRequirements.map((req) => (
              <tr key={req.id}>
                <td><code>{req.id}</code></td>
                <td>{PILLAR_LABELS[req.pillarId]}</td>
                <td>
                  <strong>{req.title}</strong>
                  {req.description && <span className="desc">{req.description}</span>}
                </td>
                <td>
                  <span className={`badge sev-${req.severity}`}>{req.severity}</span>
                </td>
                <td>{req.platforms.join(", ")}</td>
                <td>
                  <button
                    type="button"
                    className="btn-danger"
                    onClick={() => onDelete(req.id)}
                    title="Remove criterion"
                  >
                    ×
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <style>{`
        .criteria-editor h2 { margin: 0; color: var(--mad-navy); }
        .intro { color: var(--mad-text-muted); margin: 0.25rem 0 0; font-size: 0.9rem; }
        .toolbar { display: flex; justify-content: space-between; align-items: flex-start;
          gap: 1rem; margin-bottom: 1.25rem; flex-wrap: wrap; }
        .btn-primary {
          background: var(--mad-navy); color: white; border: 2px solid var(--mad-cyan);
          padding: 0.5rem 1rem; border-radius: 6px; font-weight: 600; cursor: pointer;
        }
        .btn-primary:disabled { opacity: 0.6; cursor: not-allowed; }
        .btn-danger {
          background: none; border: 1px solid var(--mad-critical); color: var(--mad-critical);
          width: 28px; height: 28px; border-radius: 4px; cursor: pointer; font-size: 1.1rem;
        }
        .add-form {
          background: white; border-radius: 10px; padding: 1.25rem; margin-bottom: 1.25rem;
          box-shadow: 0 2px 8px rgba(10,22,40,0.08); display: flex; flex-direction: column; gap: 0.75rem;
        }
        .add-form label { display: flex; flex-direction: column; gap: 0.25rem;
          font-size: 0.8rem; font-weight: 600; color: var(--mad-navy); }
        .add-form input, .add-form select, .add-form textarea {
          font-weight: 400; padding: 0.5rem; border: 1px solid #dde1e6; border-radius: 4px;
          font-family: inherit; font-size: 0.9rem;
        }
        .form-row { display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 0.75rem; }
        .criteria-table-wrap {
          background: white; border-radius: 10px; overflow: auto;
          box-shadow: 0 2px 8px rgba(10,22,40,0.08);
        }
        .criteria-table { width: 100%; border-collapse: collapse; font-size: 0.85rem; }
        .criteria-table th, .criteria-table td { padding: 0.65rem 0.75rem; text-align: left;
          border-bottom: 1px solid #e8eaed; vertical-align: top; }
        .criteria-table th { background: var(--mad-navy-light); color: white; font-weight: 600; }
        .criteria-table code { font-family: var(--font-mono); font-size: 0.75rem;
          background: #f0f2f5; padding: 0.1rem 0.35rem; border-radius: 3px; }
        .criteria-table .desc { display: block; font-size: 0.8rem; color: var(--mad-text-muted);
          font-weight: 400; margin-top: 0.2rem; }
        .badge { font-size: 0.7rem; font-weight: 700; text-transform: uppercase;
          padding: 0.1rem 0.4rem; border-radius: 3px; }
        .sev-critical { background: #fde8ea; color: var(--mad-critical); }
        .sev-high { background: #fff3e0; color: var(--mad-high); }
        .sev-medium { background: #fff8e1; color: #b8860b; }
      `}</style>
    </section>
  );
}
