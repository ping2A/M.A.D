use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::pillar::{Pillar, PillarId, Requirement, RequirementSeverity};
use crate::policy::PolicyBundle;
use crate::scoring::ScoringConfig;
use crate::vendor::{Vendor, VendorAssessment, VendorId};

/// Mutable evaluation workspace: criteria, vendors, assessments, and scoring rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationWorkspace {
    pub policy_version: String,
    pub scoring: ScoringConfig,
    pub pillars: Vec<Pillar>,
    pub vendors: Vec<Vendor>,
    /// vendor_id → assessment
    pub assessments: HashMap<String, VendorAssessment>,
}

impl EvaluationWorkspace {
    pub fn from_policy(bundle: &PolicyBundle) -> Self {
        let policy_version = bundle
            .source_paths
            .first()
            .and_then(|p| PolicyBundle::load_file(p).ok())
            .map(|p| p.manifest.version)
            .unwrap_or_else(|| "1.0.0".into());

        Self {
            policy_version,
            scoring: ScoringConfig::default(),
            pillars: bundle.pillars.clone(),
            vendors: Vec::new(),
            assessments: HashMap::new(),
        }
    }

    pub fn total_requirements(&self) -> usize {
        self.pillars.iter().map(|p| p.requirements.len()).sum()
    }

    pub fn critical_requirements(&self) -> usize {
        self.pillars
            .iter()
            .flat_map(|p| &p.requirements)
            .filter(|r| r.severity == RequirementSeverity::Critical)
            .count()
    }

    pub fn add_requirement(&mut self, pillar_id: PillarId, requirement: Requirement) -> bool {
        if let Some(pillar) = self.pillars.iter_mut().find(|p| p.id == pillar_id) {
            if pillar.requirements.iter().any(|r| r.id == requirement.id) {
                return false;
            }
            pillar.requirements.push(requirement);
            true
        } else {
            false
        }
    }

    pub fn remove_requirement(&mut self, requirement_id: &str) -> bool {
        let mut removed = false;
        for pillar in &mut self.pillars {
            let before = pillar.requirements.len();
            pillar
                .requirements
                .retain(|r| r.id != requirement_id);
            if pillar.requirements.len() < before {
                removed = true;
            }
        }
        if removed {
            for assessment in self.assessments.values_mut() {
                assessment.requirements.remove(requirement_id);
            }
        }
        removed
    }

    pub fn set_assessment(
        &mut self,
        vendor_id: &str,
        requirement_id: &str,
        status: crate::vendor::ComplianceStatus,
        notes: Option<String>,
    ) {
        let assessment = self
            .assessments
            .entry(vendor_id.to_string())
            .or_insert_with(|| VendorAssessment {
                vendor_id: VendorId::new(vendor_id),
                requirements: HashMap::new(),
            });
        assessment.requirements.insert(
            requirement_id.to_string(),
            crate::vendor::RequirementAssessment {
                requirement_id: requirement_id.to_string(),
                status,
                notes,
            },
        );
    }

    pub fn to_policy_bundle(&self) -> PolicyBundle {
        PolicyBundle {
            pillars: self.pillars.clone(),
            source_paths: Vec::new(),
        }
    }
}
