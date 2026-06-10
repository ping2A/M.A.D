import type { Requirement, RequirementResult } from "../types";

interface RequirementListProps {
  requirements: Requirement[] | RequirementResult[];
  showStatus?: boolean;
  showTechnical?: boolean;
}

function severityClass(severity: string): string {
  return `severity-${severity}`;
}

function statusClass(status: string): string {
  return `status-${status}`;
}

function statusLabel(status: string): string {
  return status.replace("_", " ");
}

export function RequirementList({ requirements, showStatus, showTechnical }: RequirementListProps) {
  return (
    <ul className="req-list">
      {requirements.map((req) => {
        const id = "requirement_id" in req ? req.requirement_id : req.id;
        const status = "status" in req ? req.status : null;
        const notes = "notes" in req ? req.notes : null;

        return (
          <li key={id} className="req-item">
            <div className="req-header">
              <code className="req-id">{id}</code>
              <span className={`req-severity ${severityClass(req.severity)}`}>
                {req.severity}
              </span>
              {showStatus && status && (
                <span className={`req-status ${statusClass(status)}`}>
                  {statusLabel(status)}
                </span>
              )}
            </div>
            <p className="req-title">{req.title}</p>
            {"description" in req && req.description && (
              <p className="req-desc">{req.description}</p>
            )}
            {showTechnical && "evaluation_method" in req && req.evaluation_method && (
              <div className="req-tech">
                <strong>Evaluation method:</strong> {req.evaluation_method}
              </div>
            )}
            {showTechnical && "technical_criteria" in req && req.technical_criteria && (
              <div className="req-tech">
                <strong>Technical criteria:</strong> {req.technical_criteria}
              </div>
            )}
            {notes && <p className="req-notes">{notes}</p>}
          </li>
        );
      })}
      <style>{`
        .req-list {
          list-style: none;
          margin: 0;
          padding: 0;
        }
        .req-item {
          padding: 1rem;
          border-bottom: 1px solid #e2e6ea;
        }
        .req-item:last-child {
          border-bottom: none;
        }
        .req-header {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          flex-wrap: wrap;
        }
        .req-id {
          font-family: var(--font-mono);
          font-size: 0.75rem;
          background: var(--mad-navy-light);
          color: var(--mad-cyan);
          padding: 0.15rem 0.5rem;
          border-radius: 4px;
        }
        .req-severity {
          font-size: 0.7rem;
          font-weight: 700;
          text-transform: uppercase;
          padding: 0.1rem 0.4rem;
          border-radius: 3px;
        }
        .severity-critical { background: #fde8ea; color: var(--mad-critical); }
        .severity-high { background: #fff3e0; color: var(--mad-high); }
        .severity-medium { background: #fff8e1; color: #b8860b; }
        .req-status {
          font-size: 0.7rem;
          font-weight: 600;
          text-transform: uppercase;
          padding: 0.1rem 0.4rem;
          border-radius: 3px;
          margin-left: auto;
        }
        .status-compliant { background: #e8f5e9; color: var(--mad-compliant); }
        .status-partial { background: #fff8e1; color: var(--mad-partial); }
        .status-non_compliant { background: #fde8ea; color: var(--mad-gap); }
        .status-untested { background: #eceff1; color: var(--mad-text-muted); }
        .req-title {
          margin: 0.4rem 0 0;
          font-weight: 600;
          font-size: 0.95rem;
        }
        .req-desc {
          margin: 0.25rem 0 0;
          font-size: 0.85rem;
          color: var(--mad-text-muted);
        }
        .req-tech {
          margin: 0.5rem 0 0;
          font-size: 0.8rem;
          color: var(--mad-text-muted);
          line-height: 1.5;
          padding: 0.5rem 0.75rem;
          background: #f4f6f8;
          border-radius: 4px;
        }
        .req-tech strong {
          color: var(--mad-navy);
          font-size: 0.75rem;
          text-transform: uppercase;
          letter-spacing: 0.03em;
        }
        .req-notes {
          margin: 0.5rem 0 0;
          font-size: 0.8rem;
          font-style: italic;
          color: var(--mad-high);
          padding-left: 0.75rem;
          border-left: 3px solid var(--mad-high);
        }
      `}</style>
    </ul>
  );
}
