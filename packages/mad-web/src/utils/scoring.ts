import type { ComplianceStatus } from "../types";

export function scoreColor(percent: number): string {
  if (percent >= 90) return "var(--mad-compliant)";
  if (percent >= 70) return "var(--mad-partial)";
  return "var(--mad-gap)";
}

export function statusLabel(status: ComplianceStatus): string {
  return status.replace(/_/g, " ");
}

export const STATUS_CYCLE: ComplianceStatus[] = [
  "untested",
  "compliant",
  "partial",
  "non_compliant",
];

export function nextStatus(current: ComplianceStatus): ComplianceStatus {
  const i = STATUS_CYCLE.indexOf(current);
  return STATUS_CYCLE[(i + 1) % STATUS_CYCLE.length];
}

export const STATUS_COLORS: Record<ComplianceStatus, string> = {
  compliant: "#28a745",
  partial: "#e6a800",
  non_compliant: "#dc3545",
  untested: "#adb5bd",
};
