use std::collections::{HashMap, HashSet};

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
    /// When false, this criterion does not apply to the vendor (no tag overlap) and is excluded from scores.
    #[serde(default = "default_applicable")]
    pub applicable: bool,
}

fn default_applicable() -> bool {
    true
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
            .map(|pillar| self.evaluate_pillar(pillar, assessment, &vendor))
            .collect();

        let overall_score = compute_overall_score(vendor.clone(), &pillars);

        EvaluationResult {
            vendor,
            pillars,
            overall_score,
        }
    }

    fn evaluate_pillar(
        &self,
        pillar: &Pillar,
        assessment: &VendorAssessment,
        vendor: &Vendor,
    ) -> PillarEvaluation {
        let requirements: Vec<RequirementResult> = pillar
            .requirements
            .iter()
            .map(|req| {
                let applicable = requirement_applies_to_vendor(&req.tags, &vendor.tags);
                let assessment_entry = assessment.requirements.get(&req.id);
                RequirementResult {
                    requirement_id: req.id.clone(),
                    title: req.title.clone(),
                    severity: req.severity,
                    status: assessment_entry
                        .map(|a| a.status)
                        .unwrap_or(ComplianceStatus::Untested),
                    notes: assessment_entry.and_then(|a| a.notes.clone()),
                    applicable,
                }
            })
            .collect();

        let scored: Vec<RequirementResult> = requirements
            .iter()
            .filter(|r| r.applicable)
            .cloned()
            .collect();
        let score = score_pillar(&pillar.id, &scored, &self.scoring);

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

    let scoring_pillars: Vec<&PillarScore> =
        pillar_scores.iter().filter(|s| s.total > 0).collect();
    let overall_score_percent = if scoring_pillars.is_empty() {
        0.0
    } else {
        scoring_pillars.iter().map(|s| s.score_percent).sum::<f64>()
            / scoring_pillars.len() as f64
    };

    let critical_gaps: Vec<String> = pillar_evaluations
        .iter()
        .flat_map(|p| &p.requirements)
        .filter(|r| {
            r.applicable
                && r.severity == crate::pillar::RequirementSeverity::Critical
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

fn normalize_tag(tag: &str) -> String {
    tag.trim().to_lowercase()
}

/// A requirement applies when it has no tags (universal) or shares at least one tag with the vendor.
pub fn requirement_applies_to_vendor(req_tags: &[String], vendor_tags: &[String]) -> bool {
    let req: HashSet<String> = req_tags
        .iter()
        .map(|t| normalize_tag(t))
        .filter(|t| !t.is_empty())
        .collect();
    if req.is_empty() {
        return true;
    }
    let vendor: HashSet<String> = vendor_tags
        .iter()
        .map(|t| normalize_tag(t))
        .filter(|t| !t.is_empty())
        .collect();
    if vendor.is_empty() {
        return false;
    }
    req.iter().any(|t| vendor.contains(t))
}

/// When a tag filter is active, only tagged requirements sharing a filter tag are in scope.
pub fn requirement_matches_tags(req_tags: &[String], filter_tags: &[String]) -> bool {
    if filter_tags.is_empty() {
        return true;
    }
    let req: HashSet<String> = req_tags
        .iter()
        .map(|t| normalize_tag(t))
        .filter(|t| !t.is_empty())
        .collect();
    if req.is_empty() {
        return false;
    }
    let active: HashSet<String> = filter_tags.iter().map(|t| normalize_tag(t)).collect();
    req.iter().any(|t| active.contains(t))
}

pub fn requirement_tags_from_bundle(bundle: &PolicyBundle) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    for pillar in &bundle.pillars {
        for req in &pillar.requirements {
            map.insert(req.id.clone(), req.tags.clone());
        }
    }
    map
}

fn tag_on_any_criterion(tag: &str, req_tags: &HashMap<String, Vec<String>>) -> bool {
    let normalized = normalize_tag(tag);
    req_tags.values().any(|tags| {
        tags.iter()
            .any(|t| normalize_tag(t) == normalized)
    })
}

/// Split selected chips into tags on criteria vs vendor-only labels.
pub fn split_active_tags(
    active_tags: &[String],
    req_tags: &HashMap<String, Vec<String>>,
) -> (Vec<String>, Vec<String>) {
    let mut criteria_tags = Vec::new();
    let mut vendor_tags = Vec::new();
    let mut seen_criteria = HashSet::new();
    let mut seen_vendor = HashSet::new();
    for tag in active_tags {
        let normalized = normalize_tag(tag);
        if normalized.is_empty() {
            continue;
        }
        if tag_on_any_criterion(&normalized, req_tags) {
            if seen_criteria.insert(normalized.clone()) {
                criteria_tags.push(normalized);
            }
        } else if seen_vendor.insert(normalized.clone()) {
            vendor_tags.push(normalized);
        }
    }
    (criteria_tags, vendor_tags)
}

pub fn vendor_in_tag_filter_scope(
    vendor: &Vendor,
    active_tags: &[String],
    req_tags: &HashMap<String, Vec<String>>,
) -> bool {
    if active_tags.is_empty() {
        return true;
    }
    let (criteria_tags, vendor_tags) = split_active_tags(active_tags, req_tags);
    if !vendor_tags.is_empty() && vendor_matches_tags(vendor, &vendor_tags) {
        return true;
    }
    if !criteria_tags.is_empty() && vendor_matches_criteria_tag_filter(vendor, &criteria_tags, req_tags)
    {
        return true;
    }
    false
}

pub fn criterion_in_tag_filter_scope(
    requirement_tags: &[String],
    active_tags: &[String],
    req_tags: &HashMap<String, Vec<String>>,
) -> bool {
    if active_tags.is_empty() {
        return true;
    }
    let (criteria_tags, vendor_tags) = split_active_tags(active_tags, req_tags);
    if !criteria_tags.is_empty() {
        return requirement_matches_tags(requirement_tags, &criteria_tags);
    }
    !vendor_tags.is_empty()
}

fn recompute_vendor_for_tag_filter(
    result: &mut EvaluationResult,
    criteria_tags: &[String],
    req_tags: &HashMap<String, Vec<String>>,
    scoring: &ScoringConfig,
) {
    let vendor_tags = &result.vendor.tags;
    for pillar in &mut result.pillars {
        for req in &mut pillar.requirements {
            let tags = req_tags
                .get(&req.requirement_id)
                .map(|t| t.as_slice())
                .unwrap_or(&[]);
            let vendor_ok = requirement_applies_to_vendor(tags, vendor_tags);
            let tag_ok =
                criteria_tags.is_empty() || requirement_matches_tags(tags, criteria_tags);
            req.applicable = vendor_ok && tag_ok;
        }
        let scored: Vec<RequirementResult> = pillar
            .requirements
            .iter()
            .filter(|r| r.applicable)
            .cloned()
            .collect();
        pillar.score = score_pillar(&pillar.pillar_id, &scored, scoring);
    }
    result.overall_score = compute_overall_score(result.vendor.clone(), &result.pillars);
}

/// Parses a comma-separated tag list from a query string (`shortlist,ios`).
pub fn parse_vendor_tags_query(raw: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut tags = Vec::new();
    for part in raw.split(',') {
        let tag = part.trim().to_lowercase();
        if tag.is_empty() || !seen.insert(tag.clone()) {
            continue;
        }
        tags.push(tag);
    }
    tags
}

pub fn vendor_matches_criteria_tag_filter(
    vendor: &Vendor,
    filter_tags: &[String],
    req_tags: &HashMap<String, Vec<String>>,
) -> bool {
    if filter_tags.is_empty() {
        return true;
    }
    for tags in req_tags.values() {
        if !requirement_matches_tags(tags, filter_tags) {
            continue;
        }
        if requirement_applies_to_vendor(tags, &vendor.tags) {
            return true;
        }
    }
    false
}

pub fn vendor_matches_tags(vendor: &Vendor, tags: &[String]) -> bool {
    if tags.is_empty() {
        return true;
    }
    let active: HashSet<String> = tags.iter().map(|t| normalize_tag(t)).collect();
    vendor.tags.iter().any(|t| {
        let normalized = normalize_tag(t);
        !normalized.is_empty() && active.contains(&normalized)
    })
}

/// Keeps vendors matching any active tag, scopes criteria to matching tags, and recomputes scores.
pub fn filter_evaluation_by_tags(
    mut report: EvaluationReport,
    tags: &[String],
    req_tags: &HashMap<String, Vec<String>>,
) -> EvaluationReport {
    if tags.is_empty() {
        return report;
    }
    let (criteria_tags, _) = split_active_tags(tags, req_tags);
    report
        .vendors
        .retain(|result| vendor_in_tag_filter_scope(&result.vendor, tags, req_tags));
    let scoring = report.scoring.clone();
    for result in &mut report.vendors {
        recompute_vendor_for_tag_filter(result, &criteria_tags, req_tags, &scoring);
    }
    apply_price_ranking(&mut report.vendors, &report.procurement);
    report
}

pub fn filter_vendor_map<T: Clone>(
    map: &HashMap<String, T>,
    vendors: &[EvaluationResult],
) -> HashMap<String, T> {
    let ids: HashSet<&str> = vendors
        .iter()
        .map(|v| v.vendor.id.0.as_str())
        .collect();
    map.iter()
        .filter(|(id, _)| ids.contains(id.as_str()))
        .map(|(id, value)| (id.clone(), value.clone()))
        .collect()
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
        tags: vec![
            "cloud".into(),
            "microsoft".into(),
            "shortlist".into(),
            "containerization".into(),
            "dlp".into(),
            "zero-trust".into(),
            "idp".into(),
            "conditional-access".into(),
            "jailbreak".into(),
            "root".into(),
            "remediation".into(),
            "isolation".into(),
            "forensics".into(),
            "volatile-memory".into(),
            "triage".into(),
            "logs".into(),
            "silent".into(),
            "siem".into(),
            "audit".into(),
            "api".into(),
            "android".into(),
            "android-enterprise".into(),
            "work-profile".into(),
            "cobo".into(),
            "kiosk".into(),
            "oemconfig".into(),
            "knox".into(),
        ],
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
        tags: vec![
            "apple".into(),
            "ios".into(),
            "containerization".into(),
            "dlp".into(),
            "abm".into(),
            "supervised".into(),
            "jailbreak".into(),
            "root".into(),
            "remediation".into(),
            "isolation".into(),
            "forensics".into(),
            "volatile-memory".into(),
            "triage".into(),
            "logs".into(),
            "silent".into(),
            "siem".into(),
            "audit".into(),
            "api".into(),
        ],
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
        tags: vec![
            "uem".into(),
            "shortlist".into(),
            "containerization".into(),
            "dlp".into(),
            "zero-trust".into(),
            "jailbreak".into(),
            "root".into(),
            "remediation".into(),
            "android".into(),
            "android-enterprise".into(),
            "work-profile".into(),
            "cobo".into(),
            "kiosk".into(),
            "oemconfig".into(),
            "knox".into(),
            "siem".into(),
            "audit".into(),
            "api".into(),
        ],
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

/// Pre-filled vendor set (Intune, Jamf, Workspace ONE) for demos and onboarding.
pub fn sample_vendor_set_file() -> crate::vendor_set::VendorSetFile {
    use crate::vendor_set::VendorSetFile;
    let samples = sample_vendors();
    let vendors: Vec<Vendor> = samples.iter().map(|(v, _)| v.clone()).collect();
    let assessments: HashMap<String, VendorAssessment> = samples
        .into_iter()
        .map(|(v, a)| (v.id.0.clone(), a))
        .collect();
    VendorSetFile::new(
        "sample",
        vendors,
        assessments,
        HashMap::new(),
        HashMap::new(),
    )
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

    #[test]
    fn requirement_applies_when_tags_overlap() {
        assert!(requirement_applies_to_vendor(
            &["ios".into(), "apple".into()],
            &["apple".into()],
        ));
        assert!(!requirement_applies_to_vendor(
            &["android".into()],
            &["apple".into(), "ios".into()],
        ));
        assert!(requirement_applies_to_vendor(&[], &["anything".into()]));
        assert!(!requirement_applies_to_vendor(
            &["cloud".into()],
            &[],
        ));
        assert!(requirement_applies_to_vendor(
            &["Cloud".into()],
            &["cloud".into()],
        ));
    }

    #[test]
    fn overall_score_averages_only_pillars_with_applicable_requirements() {
        use crate::pillar::RequirementSeverity;
        use crate::vendor::VendorId;

        let vendor = Vendor {
            id: VendorId::new("apt-only"),
            name: "APT Vendor".into(),
            description: String::new(),
            website: None,
            pricing: None,
            tags: vec!["apt".into()],
        };

        let applicable_req = RequirementResult {
            requirement_id: "dfir-005".into(),
            title: "APT criterion".into(),
            severity: RequirementSeverity::Critical,
            status: ComplianceStatus::Compliant,
            notes: None,
            applicable: true,
        };
        let na_req = RequirementResult {
            requirement_id: "plat-002".into(),
            title: "Android only".into(),
            severity: RequirementSeverity::Critical,
            status: ComplianceStatus::Untested,
            notes: None,
            applicable: false,
        };

        let dfir_pillar = PillarEvaluation {
            pillar_id: "dfir".into(),
            pillar_name: "DFIR".into(),
            requirements: vec![applicable_req.clone()],
            score: score_pillar("dfir", &[applicable_req], &ScoringConfig::default()),
        };
        let platform_pillar = PillarEvaluation {
            pillar_id: "platform_os".into(),
            pillar_name: "Platform".into(),
            requirements: vec![na_req],
            score: score_pillar("platform_os", &[], &ScoringConfig::default()),
        };

        let overall = compute_overall_score(
            vendor,
            &[dfir_pillar, platform_pillar],
        );
        assert_eq!(overall.overall_score_percent, 100.0);
    }

    #[test]
    fn scores_exclude_non_applicable_requirements() {
        use crate::vendor::VendorId;

        let dir = std::path::Path::new("policies");
        if !dir.exists() {
            return;
        }
        let bundle = PolicyBundle::load_dir(dir).expect("policy");
        let mut evaluator = Evaluator::new(bundle);
        let vendor = Vendor {
            id: VendorId::new("apple-only"),
            name: "Apple MDM".into(),
            description: String::new(),
            website: None,
            pricing: None,
            tags: vec!["apple".into(), "ios".into()],
        };
        let assessment = VendorAssessment {
            vendor_id: VendorId::new("apple-only"),
            requirements: HashMap::new(),
        };
        evaluator.add_vendor(vendor, assessment);
        let report = evaluator.evaluate().expect("report");
        let result = &report.vendors[0];
        let abm = result
            .pillars
            .iter()
            .flat_map(|p| &p.requirements)
            .find(|r| r.requirement_id == "plat-001")
            .expect("abm requirement");
        assert!(abm.applicable);
        let android_kiosk = result
            .pillars
            .iter()
            .flat_map(|p| &p.requirements)
            .find(|r| r.requirement_id == "plat-002")
            .expect("android kiosk requirement");
        assert!(!android_kiosk.applicable);
        assert!(
            result.overall_score.overall_score_percent < 100.0
                || android_kiosk.status == ComplianceStatus::Untested
        );
    }

    #[test]
    fn filter_evaluation_by_tags_keeps_matches() {
        let dir = std::path::Path::new("policies");
        if !dir.exists() {
            return;
        }
        let bundle = PolicyBundle::load_dir(dir).expect("policy");
        let req_tags = requirement_tags_from_bundle(&bundle);
        let mut evaluator = Evaluator::new(bundle);
        for (vendor, assessment) in sample_vendors() {
            evaluator.add_vendor(vendor, assessment);
        }
        let mut report = evaluator.evaluate().expect("report");
        let filtered = filter_evaluation_by_tags(report, &["containerization".into()], &req_tags);
        assert!(!filtered.vendors.is_empty());
        assert!(
            filtered
                .vendors
                .iter()
                .any(|v| v.vendor.id.0 == "intune")
        );
    }

    #[test]
    fn filter_evaluation_by_tags_scopes_criteria() {
        let dir = std::path::Path::new("policies");
        if !dir.exists() {
            return;
        }
        let bundle = PolicyBundle::load_dir(dir).expect("policy");
        let req_tags = requirement_tags_from_bundle(&bundle);
        let mut evaluator = Evaluator::new(bundle);
        for (vendor, assessment) in sample_vendors() {
            evaluator.add_vendor(vendor, assessment);
        }
        let report = evaluator.evaluate().expect("report");
        let jamf = report
            .vendors
            .iter()
            .find(|v| v.vendor.id.0 == "jamf")
            .expect("jamf");
        let android = jamf
            .pillars
            .iter()
            .flat_map(|p| &p.requirements)
            .find(|r| r.requirement_id == "plat-002")
            .expect("android req");
        assert!(!android.applicable);

        let filtered = filter_evaluation_by_tags(report, &["apple".into()], &req_tags);
        let jamf_filtered = filtered
            .vendors
            .iter()
            .find(|v| v.vendor.id.0 == "jamf")
            .expect("jamf filtered");
        let abm = jamf_filtered
            .pillars
            .iter()
            .flat_map(|p| &p.requirements)
            .find(|r| r.requirement_id == "plat-001")
            .expect("abm");
        assert!(abm.applicable);
        let android_filtered = jamf_filtered
            .pillars
            .iter()
            .flat_map(|p| &p.requirements)
            .find(|r| r.requirement_id == "plat-002")
            .expect("android");
        assert!(!android_filtered.applicable);
    }

    #[test]
    fn requirement_matches_tags_filter() {
        assert!(requirement_matches_tags(&["apple".into()], &["apple".into()]));
        assert!(!requirement_matches_tags(
            &["android".into()],
            &["apple".into()]
        ));
        assert!(!requirement_matches_tags(&[], &["apple".into()]));
        assert!(requirement_matches_tags(&["ios".into()], &[]));
    }

    #[test]
    fn split_active_tags_separates_vendor_only_labels() {
        let mut req_tags = HashMap::new();
        req_tags.insert("r1".into(), vec!["apt".into()]);
        let (criteria, vendor) =
            split_active_tags(&["apt".into(), "shortlist".into()], &req_tags);
        assert_eq!(criteria, vec!["apt"]);
        assert_eq!(vendor, vec!["shortlist"]);
    }

    #[test]
    fn vendor_only_tag_filter_keeps_vendors_without_scoping_criteria() {
        use crate::vendor::VendorId;

        let mut req_tags = HashMap::new();
        req_tags.insert("r1".into(), vec![]);
        let vendor_a = Vendor {
            id: VendorId::new("a"),
            name: "A".into(),
            description: String::new(),
            website: None,
            pricing: None,
            tags: vec!["shortlist".into()],
        };
        let vendor_b = Vendor {
            id: VendorId::new("b"),
            name: "B".into(),
            description: String::new(),
            website: None,
            pricing: None,
            tags: vec![],
        };
        let make_result = |vendor: Vendor| EvaluationResult {
            vendor: vendor.clone(),
            pillars: vec![PillarEvaluation {
                pillar_id: "p".into(),
                pillar_name: "P".into(),
                requirements: vec![RequirementResult {
                    requirement_id: "r1".into(),
                    title: "Req".into(),
                    severity: crate::pillar::RequirementSeverity::Medium,
                    status: ComplianceStatus::Compliant,
                    notes: None,
                    applicable: true,
                }],
                score: PillarScore {
                    pillar_id: "p".into(),
                    compliant: 1,
                    partial: 0,
                    non_compliant: 0,
                    untested: 0,
                    total: 1,
                    score_percent: 100.0,
                },
            }],
            overall_score: VendorScore {
                vendor,
                pillar_scores: vec![],
                overall_score_percent: 100.0,
                critical_gaps: vec![],
                annual_cost_per_device: None,
                total_annual_cost: None,
                price_currency: None,
                price_score_percent: None,
                composite_score_percent: None,
            },
        };
        let report = EvaluationReport {
            policy_version: "1".into(),
            total_requirements: 1,
            critical_requirements: 0,
            scoring: ScoringConfig::default(),
            procurement: ProcurementConfig::default(),
            vendors: vec![make_result(vendor_a), make_result(vendor_b)],
        };
        let filtered = filter_evaluation_by_tags(report, &["shortlist".into()], &req_tags);
        assert_eq!(filtered.vendors.len(), 1);
        assert_eq!(filtered.vendors[0].vendor.id.0, "a");
        let req = &filtered.vendors[0].pillars[0].requirements[0];
        assert!(req.applicable);
        assert_eq!(filtered.vendors[0].overall_score.overall_score_percent, 100.0);
    }
}
