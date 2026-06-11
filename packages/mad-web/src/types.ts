export type BuiltinPillarId = "cybersecurity_dlp" | "dfir" | "platform_os";

export const BUILTIN_PILLAR_IDS: BuiltinPillarId[] = [
  "cybersecurity_dlp",
  "dfir",
  "platform_os",
];

/** Criteria group identifier (built-in or custom). */
export type PillarId = string;

export type RequirementSeverity = "critical" | "high" | "medium";

export type ComplianceStatus =
  | "compliant"
  | "partial"
  | "non_compliant"
  | "untested";

export interface Requirement {
  id: string;
  title: string;
  description: string;
  severity: RequirementSeverity;
  platforms: string[];
  tags: string[];
  evaluation_method?: string;
  technical_criteria?: string;
}

export interface Pillar {
  id: PillarId;
  name: string;
  description: string;
  requirements: Requirement[];
}

export interface ScoringConfig {
  compliant_points: number;
  partial_points: number;
  non_compliant_points: number;
  untested_points: number;
  critical_weight: number;
  high_weight: number;
  medium_weight: number;
  use_severity_weighting: boolean;
}

export interface PolicySummary {
  version: string;
  pillar_count: number;
  total_requirements: number;
  critical_requirements: number;
  pillars: Pillar[];
  scoring: ScoringConfig;
  procurement: ProcurementConfig;
  value_streams: Record<string, ValueStreamMap>;
}

export type BillingPeriod = "monthly" | "annual";

export interface VendorPricing {
  currency: string;
  billing_period: BillingPeriod;
  price_per_device?: number | null;
  global_price?: number | null;
  notes?: string | null;
}

export interface Vendor {
  id: string;
  name: string;
  description: string;
  website?: string | null;
  pricing?: VendorPricing | null;
  tags?: string[];
}

export interface ProcurementConfig {
  device_count: number;
  price_weight_percent: number;
  use_price_in_ranking: boolean;
}

export interface RequirementAssessment {
  requirement_id: string;
  status: ComplianceStatus;
  notes: string | null;
}

export interface VendorAssessment {
  vendor_id: string;
  requirements: Record<string, RequirementAssessment>;
}

export type VsmNodeType =
  | "process"
  | "decision"
  | "info"
  | "delay"
  | "external"
  | "customer"
  | "supplier"
  | "inventory"
  | "kaizen";

export interface VsmFlowTypeDef {
  id: string;
  label: string;
  color: string;
  dash?: string;
}

export interface VsmNode {
  id: string;
  label: string;
  node_type: VsmNodeType;
  x: number;
  y: number;
  width: number;
  height: number;
  notes?: string;
  role?: string;
  lead_time_minutes?: number;
  cycle_time_minutes?: number;
  author?: string;
}

export interface VsmEdge {
  id: string;
  from: string;
  to: string;
  label?: string;
  edge_type?: string;
  duration_minutes?: number;
}

export interface VsmMessage {
  id: string;
  text: string;
  node_id?: string;
  edge_id?: string;
}

export interface ValueStreamMap {
  nodes: VsmNode[];
  edges: VsmEdge[];
  messages: VsmMessage[];
  flow_types?: VsmFlowTypeDef[];
}

export interface EvaluationWorkspace {
  policy_version: string;
  scoring: ScoringConfig;
  procurement: ProcurementConfig;
  pillars: Pillar[];
  vendors: Vendor[];
  assessments: Record<string, VendorAssessment>;
  value_streams?: Record<string, ValueStreamMap>;
}

export interface VendorSetFile {
  format_version: number;
  exported_at: string;
  vendors: Vendor[];
  assessments: Record<string, VendorAssessment>;
}

export interface WorkspaceBundle {
  format_version: number;
  exported_at: string;
  workspace: EvaluationWorkspace;
}

export interface WorkspaceImportResult {
  kind: string;
  pillars: number;
  requirements: number;
  vendors: number;
  assessments: number;
  vendor_result?: VendorImportResult | null;
}

export type VendorImportMode = "merge" | "replace";

export interface VendorImportResult {
  added: number;
  updated: number;
  skipped: number;
  removed: number;
}

export interface PillarScore {
  pillar_id: PillarId;
  compliant: number;
  partial: number;
  non_compliant: number;
  untested: number;
  total: number;
  score_percent: number;
}

export interface RequirementResult {
  requirement_id: string;
  title: string;
  severity: RequirementSeverity;
  status: ComplianceStatus;
  notes: string | null;
}

export interface PillarEvaluation {
  pillar_id: PillarId;
  pillar_name: string;
  requirements: RequirementResult[];
  score: PillarScore;
}

export interface VendorScore {
  vendor: Vendor;
  pillar_scores: PillarScore[];
  overall_score_percent: number;
  critical_gaps: string[];
  annual_cost_per_device?: number | null;
  total_annual_cost?: number | null;
  price_currency?: string | null;
  price_score_percent?: number | null;
  composite_score_percent?: number | null;
}

export interface EvaluationResult {
  vendor: Vendor;
  pillars: PillarEvaluation[];
  overall_score: VendorScore;
}

export interface EvaluationReport {
  policy_version: string;
  total_requirements: number;
  critical_requirements: number;
  scoring: ScoringConfig;
  procurement: ProcurementConfig;
  vendors: EvaluationResult[];
}

export const STATUS_CYCLE: ComplianceStatus[] = [
  "untested",
  "compliant",
  "partial",
  "non_compliant",
];

