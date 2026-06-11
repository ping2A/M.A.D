import { useState } from "react";
import type { ComplianceStatus, RequirementSeverity } from "../types";

export interface RequirementDisplayProps {
  id: string;
  title: string;
  description?: string;
  severity: RequirementSeverity;
  platforms?: string[];
  pillarName?: string;
  severityLabel: (s: RequirementSeverity) => string;
  variant?: "matrix" | "table";
  vendorStatuses?: { name: string; status: ComplianceStatus }[];
  statusLabel?: (s: ComplianceStatus) => string;
  expandLabel?: string;
  collapseLabel?: string;
}

export function RequirementDisplay({
  id,
  title,
  description,
  severity,
  platforms = [],
  pillarName,
  severityLabel,
  variant = "matrix",
  vendorStatuses,
  statusLabel,
  expandLabel = "Show details",
  collapseLabel = "Hide details",
}: RequirementDisplayProps) {
  const [expanded, setExpanded] = useState(false);
  const hasDetails = Boolean(description?.trim()) || platforms.length > 0;
  const showExpand = hasDetails && variant === "matrix";

  const statusTooltip =
    vendorStatuses && statusLabel
      ? vendorStatuses.map((v) => `${v.name}: ${statusLabel(v.status)}`).join("\n")
      : undefined;

  return (
    <div className={`requirement-display requirement-display--${variant}`}>
      <div className="requirement-display-top">
        <code className="requirement-id">{id}</code>
        <span className={`badge sev-${severity}`}>{severityLabel(severity)}</span>
        {pillarName && <span className="requirement-pillar-tag">{pillarName}</span>}
        {vendorStatuses && vendorStatuses.length > 0 && (
          <div className="requirement-status-strip" title={statusTooltip}>
            {vendorStatuses.map((v) => (
              <span
                key={v.name}
                className={`requirement-status-dot status-${v.status}`}
                title={statusLabel ? `${v.name}: ${statusLabel(v.status)}` : v.name}
              />
            ))}
          </div>
        )}
      </div>

      <div className="requirement-display-body">
        <p className="requirement-title">{title}</p>
        {description && (
          <p className={`requirement-desc ${expanded ? "expanded" : ""}`}>{description}</p>
        )}
        {platforms.length > 0 && (
          <div className="requirement-platforms">
            {platforms.map((p) => (
              <span key={p} className="requirement-platform-tag">
                {p}
              </span>
            ))}
          </div>
        )}
      </div>

      {showExpand && (
        <button
          type="button"
          className="requirement-expand-btn"
          onClick={() => setExpanded((v) => !v)}
          aria-expanded={expanded}
        >
          {expanded ? collapseLabel : expandLabel}
        </button>
      )}
    </div>
  );
}
