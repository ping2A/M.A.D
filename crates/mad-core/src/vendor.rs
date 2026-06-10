use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::pillar::PillarId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VendorId(pub String);

impl VendorId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for VendorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vendor {
    pub id: VendorId,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub website: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    Compliant,
    Partial,
    NonCompliant,
    Untested,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementAssessment {
    pub requirement_id: String,
    pub status: ComplianceStatus,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorAssessment {
    pub vendor_id: VendorId,
    pub requirements: HashMap<String, RequirementAssessment>,
}

impl VendorAssessment {
    pub fn status_for(&self, requirement_id: &str) -> ComplianceStatus {
        self.requirements
            .get(requirement_id)
            .map(|a| a.status)
            .unwrap_or(ComplianceStatus::Untested)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PillarScore {
    pub pillar_id: PillarId,
    pub compliant: usize,
    pub partial: usize,
    pub non_compliant: usize,
    pub untested: usize,
    pub total: usize,
    pub score_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorScore {
    pub vendor: Vendor,
    pub pillar_scores: Vec<PillarScore>,
    pub overall_score_percent: f64,
    pub critical_gaps: Vec<String>,
}
