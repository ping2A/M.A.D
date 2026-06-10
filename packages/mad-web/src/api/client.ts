import type {
  ComplianceStatus,
  EvaluationReport,
  EvaluationWorkspace,
  PillarId,
  PolicySummary,
  Requirement,
  ScoringConfig,
} from "../types";

async function fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, init);
  if (!response.ok) {
    const body = await response.text();
    throw new Error(`API error: ${response.status} ${response.statusText} — ${body}`);
  }
  if (response.status === 204) {
    return undefined as T;
  }
  return response.json() as Promise<T>;
}

export function getPolicy(): Promise<PolicySummary> {
  return fetchJson<PolicySummary>("/api/policy");
}

export function getEvaluation(): Promise<EvaluationReport> {
  return fetchJson<EvaluationReport>("/api/evaluation");
}

export function getWorkspace(): Promise<EvaluationWorkspace> {
  return fetchJson<EvaluationWorkspace>("/api/workspace");
}

export function addRequirement(
  pillarId: PillarId,
  requirement: Requirement,
): Promise<EvaluationWorkspace> {
  return fetchJson<EvaluationWorkspace>("/api/workspace/requirements", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ pillar_id: pillarId, requirement }),
  });
}

export function deleteRequirement(id: string): Promise<void> {
  return fetchJson<void>(`/api/workspace/requirements/${encodeURIComponent(id)}`, {
    method: "DELETE",
  });
}

export function setAssessment(
  vendorId: string,
  requirementId: string,
  status: ComplianceStatus,
  notes?: string,
): Promise<EvaluationWorkspace> {
  return fetchJson<EvaluationWorkspace>("/api/workspace/assessments", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      vendor_id: vendorId,
      requirement_id: requirementId,
      status,
      notes: notes ?? null,
    }),
  });
}

export function updateScoring(scoring: ScoringConfig): Promise<EvaluationWorkspace> {
  return fetchJson<EvaluationWorkspace>("/api/workspace/scoring", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ scoring }),
  });
}
