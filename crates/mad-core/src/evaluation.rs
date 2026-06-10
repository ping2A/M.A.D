use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::MadResult;
use crate::pillar::{Pillar, PillarId};
use crate::policy::PolicyBundle;
use crate::pricing::{compute_annual_costs, ProcurementConfig};
use crate::scoring::ScoringConfig;
use crate::vendor::{
    ComplianceStatus, PillarScore, RequirementAssessment, Vendor, VendorAssessment, VendorScore,
};
use crate::workspace::EvaluationWorkspace;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementResult {
    pub requirement_id: String,
    pub title: String,
    pub severity: crate::pillar::RequirementSeverity,
    pub status: ComplianceStatus,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PillarEvaluation {
    pub pillar_id: PillarId,
    pub pillar_name: String,
    pub requirements: Vec<RequirementResult>,
    pub score: PillarScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub vendor: Vendor,
    pub pillars: Vec<PillarEvaluation>,
    pub overall_score: VendorScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationReport {
    pub policy_version: String,
    pub total_requirements: usize,
    pub critical_requirements: usize,
    pub scoring: ScoringConfig,
    pub procurement: ProcurementConfig,
    pub vendors: Vec<EvaluationResult>,
}

pub struct Evaluator {
    bundle: PolicyBundle,
    scoring: ScoringConfig,
    procurement: ProcurementConfig,
    vendors: Vec<Vendor>,
    assessments: Vec<VendorAssessment>,
}

impl Evaluator {
    pub fn new(bundle: PolicyBundle) -> Self {
        Self {
            bundle,
            scoring: ScoringConfig::default(),
            procurement: ProcurementConfig::default(),
            vendors: Vec::new(),
            assessments: Vec::new(),
        }
    }

    pub fn with_scoring(mut self, scoring: ScoringConfig) -> Self {
        self.scoring = scoring;
        self
    }

    pub fn with_procurement(mut self, procurement: ProcurementConfig) -> Self {
        self.procurement = procurement;
        self
    }

    pub fn from_workspace(workspace: &EvaluationWorkspace) -> Self {
        let mut evaluator = Self::new(workspace.to_policy_bundle())
            .with_scoring(workspace.scoring.clone())
            .with_procurement(workspace.procurement.clone());
        for vendor in &workspace.vendors {
            let assessment = workspace
                .assessments
                .get(&vendor.id.0)
                .cloned()
                .unwrap_or_else(|| VendorAssessment {
                    vendor_id: vendor.id.clone(),
                    requirements: HashMap::new(),
                });
            evaluator.add_vendor(vendor.clone(), assessment);
        }
        evaluator
    }

    pub fn add_vendor(&mut self, vendor: Vendor, assessment: VendorAssessment) {
        self.vendors.push(vendor);
        self.assessments.push(assessment);
    }

    pub fn policy(&self) -> &PolicyBundle {
        &self.bundle
    }

    pub fn evaluate(&self) -> MadResult<EvaluationReport> {
        let policy_version = self
            .bundle
            .source_paths
            .first()
            .and_then(|p| PolicyBundle::load_file(p).ok())
            .map(|p| p.manifest.version)
            .unwrap_or_else(|| "unknown".into());

        let mut vendors: Vec<EvaluationResult> = self
            .vendors
            .iter()
            .zip(self.assessments.iter())
            .map(|(vendor, assessment)| self.evaluate_vendor(vendor.clone(), assessment))
            .collect();

        apply_price_ranking(&mut vendors, &self.procurement);

        Ok(EvaluationReport {
            policy_version,
            total_requirements: self.bundle.total_requirements(),
            critical_requirements: self.bundle.critical_requirements(),
            scoring: self.scoring.clone(),
            procurement: self.procurement.clone(),
            vendors,
        })
    }

    fn evaluate_vendor(&self, vendor: Vendor, assessment: &VendorAssessment) -> EvaluationResult {
        let pillars: Vec<PillarEvaluation> = self
            .bundle
            .pillars
            .iter()
            .map(|pillar| self.evaluate_pillar(pillar, assessment))
            .collect();

        let overall_score = compute_overall_score(vendor.clone(), &pillars);

        EvaluationResult {
            vendor,
            pillars,
            overall_score,
        }
    }

    fn evaluate_pillar(&self, pillar: &Pillar, assessment: &VendorAssessment) -> PillarEvaluation {
        let requirements: Vec<RequirementResult> = pillar
            .requirements
            .iter()
            .map(|req| {
                let assessment_entry = assessment.requirements.get(&req.id);
                RequirementResult {
                    requirement_id: req.id.clone(),
                    title: req.title.clone(),
                    severity: req.severity,
                    status: assessment_entry
                        .map(|a| a.status)
                        .unwrap_or(ComplianceStatus::Untested),
                    notes: assessment_entry.and_then(|a| a.notes.clone()),
                }
            })
            .collect();

        let score = score_pillar(&pillar.id, &requirements, &self.scoring);

        PillarEvaluation {
            pillar_id: pillar.id.clone(),
            pillar_name: pillar.name.clone(),
            requirements,
            score,
        }
    }
}

fn score_pillar(
    pillar_id: &str,
    requirements: &[RequirementResult],
    scoring: &ScoringConfig,
) -> PillarScore {
    let mut compliant = 0usize;
    let mut partial = 0usize;
    let mut non_compliant = 0usize;
    let mut untested = 0usize;
    let mut earned = 0.0;
    let mut max_possible = 0.0;

    for req in requirements {
        match req.status {
            ComplianceStatus::Compliant => compliant += 1,
            ComplianceStatus::Partial => partial += 1,
            ComplianceStatus::NonCompliant => non_compliant += 1,
            ComplianceStatus::Untested => untested += 1,
        }
        let (e, m) = scoring.requirement_score(req.status, req.severity);
        earned += e;
        max_possible += m;
    }

    let total = requirements.len();
    let score_percent = if max_possible == 0.0 {
        0.0
    } else {
        (earned / max_possible) * 100.0
    };

    PillarScore {
        pillar_id: pillar_id.to_string(),
        compliant,
        partial,
        non_compliant,
        untested,
        total,
        score_percent,
    }
}

fn compute_overall_score(
    vendor: Vendor,
    pillar_evaluations: &[PillarEvaluation],
) -> VendorScore {
    let pillar_scores: Vec<PillarScore> = pillar_evaluations
        .iter()
        .map(|p| p.score.clone())
        .collect();

    let overall_score_percent = if pillar_scores.is_empty() {
        0.0
    } else {
        pillar_scores.iter().map(|s| s.score_percent).sum::<f64>() / pillar_scores.len() as f64
    };

    let critical_gaps: Vec<String> = pillar_evaluations
        .iter()
        .flat_map(|p| &p.requirements)
        .filter(|r| {
            r.severity == crate::pillar::RequirementSeverity::Critical
                && matches!(
                    r.status,
                    ComplianceStatus::NonCompliant | ComplianceStatus::Untested
                )
        })
        .map(|r| format!("{}: {}", r.requirement_id, r.title))
        .collect();

    VendorScore {
        vendor,
        pillar_scores,
        overall_score_percent,
        critical_gaps,
        annual_cost_per_device: None,
        total_annual_cost: None,
        price_currency: None,
        price_score_percent: None,
        composite_score_percent: None,
    }
}

fn apply_price_ranking(results: &mut [EvaluationResult], procurement: &ProcurementConfig) {
    let device_count = procurement.device_count;

    for result in results.iter_mut() {
        result.overall_score.annual_cost_per_device = None;
        result.overall_score.total_annual_cost = None;
        result.overall_score.price_currency = None;
        result.overall_score.price_score_percent = None;
        result.overall_score.composite_score_percent = None;

        let Some(pricing) = &result.vendor.pricing else {
            continue;
        };

        let (per_device, total) = compute_annual_costs(pricing, device_count);
        result.overall_score.annual_cost_per_device = per_device;
        result.overall_score.total_annual_cost = total;
        result.overall_score.price_currency = Some(pricing.currency.clone());
    }

    if !procurement.use_price_in_ranking {
        return;
    }

    let costs: Vec<Option<f64>> = results
        .iter()
        .map(|r| r.overall_score.annual_cost_per_device)
        .collect();
    let valid: Vec<f64> = costs.iter().filter_map(|c| *c).collect();
    if valid.is_empty() {
        return;
    }

    let min = valid.iter().copied().fold(f64::INFINITY, f64::min);
    let max = valid.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let weight = procurement.price_weight_percent.clamp(0.0, 100.0) / 100.0;

    for (result, cost) in results.iter_mut().zip(costs) {
        let Some(cost) = cost else {
            continue;
        };
        let price_score = if (max - min).abs() < f64::EPSILON {
            100.0
        } else {
            100.0 * (max - cost) / (max - min)
        };
        let capability = result.overall_score.overall_score_percent;
        let composite = (1.0 - weight) * capability + weight * price_score;
        result.overall_score.price_score_percent = Some(price_score);
        result.overall_score.composite_score_percent = Some(composite);
    }
}

pub fn sample_vendors() -> Vec<(Vendor, VendorAssessment)> {
    use crate::pricing::{BillingPeriod, VendorPricing};
    use crate::vendor::VendorId;
    use std::collections::HashMap;

    let intune = Vendor {
        id: VendorId::new("intune"),
        name: "Microsoft Intune".into(),
        description: "Cloud-based unified endpoint management".into(),
        website: Some("https://www.microsoft.com/en-us/security/business/endpoint-management/microsoft-intune".into()),
        pricing: Some(VendorPricing {
            currency: "USD".into(),
            billing_period: BillingPeriod::Monthly,
            price_per_device: Some(8.0),
            global_price: None,
            notes: Some("Typical enterprise per-device list price".into()),
        }),
        tags: vec!["cloud".into(), "microsoft".into(), "shortlist".into()],
    };

    let jamf = Vendor {
        id: VendorId::new("jamf"),
        name: "Jamf Pro".into(),
        description: "Apple-focused enterprise mobility management".into(),
        website: Some("https://www.jamf.com".into()),
        pricing: Some(VendorPricing {
            currency: "USD".into(),
            billing_period: BillingPeriod::Monthly,
            price_per_device: Some(12.0),
            global_price: None,
            notes: None,
        }),
        tags: vec!["apple".into(), "ios".into()],
    };

    let workspace_one = Vendor {
        id: VendorId::new("workspace_one"),
        name: "VMware Workspace ONE".into(),
        description: "Digital workspace platform with UEM capabilities".into(),
        website: Some("https://www.omnissa.com/products/workspace-one".into()),
        pricing: Some(VendorPricing {
            currency: "USD".into(),
            billing_period: BillingPeriod::Annual,
            price_per_device: Some(72.0),
            global_price: Some(50_000.0),
            notes: Some("Platform fee plus annual per-device license".into()),
        }),
        tags: vec!["uem".into(), "shortlist".into()],
    };

    fn assess(
        vendor_id: &str,
        entries: &[(&str, ComplianceStatus, Option<&str>)],
    ) -> VendorAssessment {
        VendorAssessment {
            vendor_id: VendorId::new(vendor_id),
            requirements: entries
                .iter()
                .map(|(id, status, notes)| {
                    (
                        (*id).to_string(),
                        RequirementAssessment {
                            requirement_id: (*id).to_string(),
                            status: *status,
                            notes: notes.map(|n| n.to_string()),
                        },
                    )
                })
                .collect::<HashMap<_, _>>(),
        }
    }

    vec![
        (
            intune,
            assess(
                "intune",
                &[
                    ("dlp-001", ComplianceStatus::Compliant, None),
                    ("dlp-002", ComplianceStatus::Compliant, None),
                    ("dlp-003", ComplianceStatus::Partial, Some("Remediation requires manual policy push")),
                    ("dfir-001", ComplianceStatus::Partial, Some("Network isolation available; volatile memory preservation limited")),
                    ("dfir-002", ComplianceStatus::Compliant, None),
                    ("dfir-003", ComplianceStatus::Compliant, None),
                    ("plat-001", ComplianceStatus::Compliant, None),
                    ("plat-002", ComplianceStatus::Compliant, None),
                    ("plat-003", ComplianceStatus::Partial, Some("OEMConfig support varies by OEM")),
                ],
            ),
        ),
        (
            jamf,
            assess(
                "jamf",
                &[
                    ("dlp-001", ComplianceStatus::Compliant, None),
                    ("dlp-002", ComplianceStatus::Partial, Some("Conditional access via third-party integrations")),
                    ("dlp-003", ComplianceStatus::Compliant, None),
                    ("dfir-001", ComplianceStatus::NonCompliant, Some("No volatile memory preservation during isolation")),
                    ("dfir-002", ComplianceStatus::Partial, Some("Silent log collection on supervised iOS only")),
                    ("dfir-003", ComplianceStatus::Partial, Some("SIEM via webhook; no native streaming API")),
                    ("plat-001", ComplianceStatus::Compliant, None),
                    ("plat-002", ComplianceStatus::NonCompliant, Some("Android support is limited compared to iOS")),
                    ("plat-003", ComplianceStatus::NonCompliant, None),
                ],
            ),
        ),
        (
            workspace_one,
            assess(
                "workspace_one",
                &[
                    ("dlp-001", ComplianceStatus::Compliant, None),
                    ("dlp-002", ComplianceStatus::Compliant, None),
                    ("dlp-003", ComplianceStatus::Compliant, None),
                    ("dfir-001", ComplianceStatus::Compliant, None),
                    ("dfir-002", ComplianceStatus::Compliant, None),
                    ("dfir-003", ComplianceStatus::Compliant, None),
                    ("plat-001", ComplianceStatus::Compliant, None),
                    ("plat-002", ComplianceStatus::Compliant, None),
                    ("plat-003", ComplianceStatus::Compliant, None),
                ],
            ),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_sample_vendors() {
        let bundle = PolicyBundle::load_dir("../../policies").expect("policies");
        let mut evaluator = Evaluator::new(bundle);
        for (vendor, assessment) in sample_vendors() {
            evaluator.add_vendor(vendor, assessment);
        }
        let report = evaluator.evaluate().expect("report");
        assert_eq!(report.vendors.len(), 3);
        assert!(report.total_requirements > 0);
    }
}
