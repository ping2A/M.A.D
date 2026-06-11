import type { ComplianceStatus } from "../types";

export type ComplianceStatusVariant = "legend" | "menu" | "matrix" | "inline";

interface ComplianceStatusBadgeProps {
  status: ComplianceStatus;
  label: string;
  variant?: ComplianceStatusVariant;
  className?: string;
}

export function StatusIcon({
  status,
  size = 14,
}: {
  status: ComplianceStatus;
  size?: number;
}) {
  const stroke = "currentColor";
  const sw = 2;

  switch (status) {
    case "compliant":
      return (
        <svg width={size} height={size} viewBox="0 0 16 16" aria-hidden>
          <circle cx="8" cy="8" r="6.5" fill="none" stroke={stroke} strokeWidth={sw} />
          <path
            d="M5 8.2 L7.2 10.5 L11.5 5.8"
            fill="none"
            stroke={stroke}
            strokeWidth={sw}
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      );
    case "partial":
      return (
        <svg width={size} height={size} viewBox="0 0 16 16" aria-hidden>
          <circle cx="8" cy="8" r="6.5" fill="none" stroke={stroke} strokeWidth={sw} />
          <path
            d="M5 8 H11"
            fill="none"
            stroke={stroke}
            strokeWidth={sw}
            strokeLinecap="round"
          />
        </svg>
      );
    case "non_compliant":
      return (
        <svg width={size} height={size} viewBox="0 0 16 16" aria-hidden>
          <circle cx="8" cy="8" r="6.5" fill="none" stroke={stroke} strokeWidth={sw} />
          <path
            d="M5.5 5.5 L10.5 10.5 M10.5 5.5 L5.5 10.5"
            fill="none"
            stroke={stroke}
            strokeWidth={sw}
            strokeLinecap="round"
          />
        </svg>
      );
    case "untested":
    default:
      return (
        <svg width={size} height={size} viewBox="0 0 16 16" aria-hidden>
          <circle
            cx="8"
            cy="8"
            r="6.5"
            fill="none"
            stroke={stroke}
            strokeWidth={sw}
            strokeDasharray="3 2"
          />
          <circle cx="8" cy="11.5" r="0.9" fill={stroke} stroke="none" />
          <path
            d="M8 4.8 V8.2"
            fill="none"
            stroke={stroke}
            strokeWidth={sw}
            strokeLinecap="round"
          />
        </svg>
      );
  }
}

export function complianceStatusClass(
  status: ComplianceStatus,
  variant: ComplianceStatusVariant = "inline",
): string {
  return `compliance-status compliance-status--${status} compliance-status--${variant}`;
}

export function ComplianceStatusBadge({
  status,
  label,
  variant = "inline",
  className = "",
}: ComplianceStatusBadgeProps) {
  const iconSize = variant === "matrix" ? 18 : variant === "legend" ? 14 : 15;
  return (
    <span className={`${complianceStatusClass(status, variant)} ${className}`.trim()}>
      <span className="compliance-status-icon">
        <StatusIcon status={status} size={iconSize} />
      </span>
      {(variant === "legend" || variant === "menu" || variant === "inline") && (
        <span className="compliance-status-label">{label}</span>
      )}
    </span>
  );
}
