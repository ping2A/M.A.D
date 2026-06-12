import type {
  ComplianceStatus,
  EvaluationReport,
  EvaluationWorkspace,
  PillarId,
  PolicySummary,
  ProcurementConfig,
  Requirement,
  ScoringConfig,
  Vendor,
  VendorImportMode,
  VendorImportResult,
  ValueStreamMap,
  VendorDocSection,
  VendorSetFile,
  WorkspaceImportResult,
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

export function addPillar(
  id: string,
  name: string,
  description: string,
): Promise<EvaluationWorkspace> {
  return fetchJson<EvaluationWorkspace>("/api/workspace/pillars", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ id, name, description }),
  });
}

export function updatePillar(
  id: string,
  name: string,
  description: string,
): Promise<EvaluationWorkspace> {
  return fetchJson<EvaluationWorkspace>(
    `/api/workspace/pillars/${encodeURIComponent(id)}`,
    {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name, description }),
    },
  );
}

export function deletePillar(id: string): Promise<void> {
  return fetchJson<void>(`/api/workspace/pillars/${encodeURIComponent(id)}`, {
    method: "DELETE",
  });
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

export function updateRequirement(
  id: string,
  pillarId: PillarId,
  requirement: Requirement,
): Promise<EvaluationWorkspace> {
  return fetchJson<EvaluationWorkspace>(
    `/api/workspace/requirements/${encodeURIComponent(id)}`,
    {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ pillar_id: pillarId, requirement }),
    },
  );
}

export function deleteRequirement(id: string): Promise<void> {
  return fetchJson<void>(`/api/workspace/requirements/${encodeURIComponent(id)}`, {
    method: "DELETE",
  });
}

export function addVendor(vendor: Vendor): Promise<EvaluationWorkspace> {
  return fetchJson<EvaluationWorkspace>("/api/workspace/vendors", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ vendor: { ...vendor, id: vendor.id } }),
  });
}

export function updateVendor(id: string, vendor: Vendor): Promise<EvaluationWorkspace> {
  return fetchJson<EvaluationWorkspace>(`/api/workspace/vendors/${encodeURIComponent(id)}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ vendor: { ...vendor, id: vendor.id } }),
  });
}

export function deleteVendor(id: string): Promise<void> {
  return fetchJson<void>(`/api/workspace/vendors/${encodeURIComponent(id)}`, {
    method: "DELETE",
  });
}

export async function exportWorkspaceJson(): Promise<void> {
  const response = await fetch("/api/workspace/export");
  if (!response.ok) {
    const body = await response.text();
    throw new Error(`${response.status} ${response.statusText} — ${body}`);
  }
  const blob = await response.blob();
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = "mad-workspace.json";
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}

export function importWorkspaceJson(
  json: string,
  vendorMode: VendorImportMode = "replace",
): Promise<{ result: WorkspaceImportResult; workspace: EvaluationWorkspace }> {
  return fetchJson<{ result: WorkspaceImportResult; workspace: EvaluationWorkspace }>(
    `/api/workspace/import?mode=${vendorMode}`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: json,
    },
  );
}

export async function exportVendorsJson(): Promise<void> {
  const response = await fetch("/api/workspace/vendors/export");
  if (!response.ok) {
    const body = await response.text();
    throw new Error(`${response.status} ${response.statusText} — ${body}`);
  }
  const blob = await response.blob();
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = "mad-vendors.json";
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}

export function loadExampleVendors(): Promise<{
  result: VendorImportResult;
  workspace: EvaluationWorkspace;
}> {
  return fetchJson<{ result: VendorImportResult; workspace: EvaluationWorkspace }>(
    "/api/workspace/vendors/load-example",
    { method: "POST" },
  );
}

export function importVendorsJson(
  file: VendorSetFile,
  mode: VendorImportMode = "merge",
): Promise<{ result: VendorImportResult; workspace: EvaluationWorkspace }> {
  return fetchJson<{ result: VendorImportResult; workspace: EvaluationWorkspace }>(
    `/api/workspace/vendors/import?mode=${mode}`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(file),
    },
  );
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

export function createValueStream(
  vendorId: string,
  name: string,
): Promise<EvaluationWorkspace> {
  return fetchJson<EvaluationWorkspace>(
    `/api/workspace/vendors/${encodeURIComponent(vendorId)}/value-streams`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name }),
    },
  );
}

export function updateValueStreamEntry(
  vendorId: string,
  streamId: string,
  name: string,
  valueStream: ValueStreamMap,
): Promise<EvaluationWorkspace> {
  return fetchJson<EvaluationWorkspace>(
    `/api/workspace/vendors/${encodeURIComponent(vendorId)}/value-streams/${encodeURIComponent(streamId)}`,
    {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name, ...valueStream }),
    },
  );
}

export function deleteValueStream(
  vendorId: string,
  streamId: string,
): Promise<EvaluationWorkspace> {
  return fetchJson<EvaluationWorkspace>(
    `/api/workspace/vendors/${encodeURIComponent(vendorId)}/value-streams/${encodeURIComponent(streamId)}`,
    { method: "DELETE" },
  );
}

export function createVendorDoc(
  vendorId: string,
  name: string,
): Promise<EvaluationWorkspace> {
  return fetchJson<EvaluationWorkspace>(
    `/api/workspace/vendors/${encodeURIComponent(vendorId)}/docs`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name }),
    },
  );
}

export function updateVendorDoc(
  vendorId: string,
  docId: string,
  section: VendorDocSection,
): Promise<EvaluationWorkspace> {
  return fetchJson<EvaluationWorkspace>(
    `/api/workspace/vendors/${encodeURIComponent(vendorId)}/docs/${encodeURIComponent(docId)}`,
    {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        name: section.name,
        color: section.color,
        overview: section.overview,
        items: section.items,
      }),
    },
  );
}

export function deleteVendorDoc(
  vendorId: string,
  docId: string,
): Promise<EvaluationWorkspace> {
  return fetchJson<EvaluationWorkspace>(
    `/api/workspace/vendors/${encodeURIComponent(vendorId)}/docs/${encodeURIComponent(docId)}`,
    { method: "DELETE" },
  );
}

export function updateProcurement(
  procurement: ProcurementConfig,
): Promise<EvaluationWorkspace> {
  return fetchJson<EvaluationWorkspace>("/api/workspace/procurement", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ procurement }),
  });
}

export type ReportFormat = "html" | "pdf";

const REPORT_FILENAMES: Record<ReportFormat, string> = {
  html: "mad-evaluation-report.html",
  pdf: "mad-evaluation-report.pdf",
};

export async function downloadReport(
  format: ReportFormat,
  locale: "en" | "fr" = "en",
  tags: string[] = [],
): Promise<void> {
  const params = new URLSearchParams();
  if (format === "html") params.set("lang", locale);
  if (tags.length > 0) params.set("tags", tags.join(","));
  const query = params.toString();
  const response = await fetch(`/api/report.${format}${query ? `?${query}` : ""}`);
  if (!response.ok) {
    const body = await response.text();
    throw new Error(`${response.status} ${response.statusText} — ${body}`);
  }
  const blob = await response.blob();
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = REPORT_FILENAMES[format];
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}
