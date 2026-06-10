export type PillarId = "cybersecurity_dlp" | "dfir" | "platform_os";

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
}

export interface Vendor {
  id: string;
  name: string;
  description: string;
  website?: string | null;
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

export interface EvaluationWorkspace {
  policy_version: string;
  scoring: ScoringConfig;
  pillars: Pillar[];
  vendors: Vendor[];
  assessments: Record<string, VendorAssessment>;
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
  vendors: EvaluationResult[];
}

export const STATUS_CYCLE: ComplianceStatus[] = [
  "untested",
  "compliant",
  "partial",
  "non_compliant",
];

export const STATUS_LABELS: Record<ComplianceStatus, string> = {
  compliant: "Compliant",
  partial: "Partial",
  non_compliant: "Non-compliant",
  untested: "Untested",
};

export const PILLAR_LABELS: Record<PillarId, string> = {
  cybersecurity_dlp: "Cybersecurity & DLP",
  dfir: "DFIR",
  platform_os: "Platform & OS",
};
