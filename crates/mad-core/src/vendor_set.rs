use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::vendor::{Vendor, VendorAssessment};

/// Portable vendor list with assessments for import/export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorSetFile {
    pub format_version: u32,
    pub exported_at: String,
    pub vendors: Vec<Vendor>,
    pub assessments: HashMap<String, VendorAssessment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VendorImportMode {
    #[default]
    Merge,
    Replace,
}

#[derive(Debug, Clone, Serialize)]
pub struct VendorImportResult {
    pub added: usize,
    pub updated: usize,
    pub skipped: usize,
    pub removed: usize,
}

impl VendorSetFile {
    pub fn new(exported_at: impl Into<String>, vendors: Vec<Vendor>, assessments: HashMap<String, VendorAssessment>) -> Self {
        Self {
            format_version: 1,
            exported_at: exported_at.into(),
            vendors,
            assessments,
        }
    }
}

pub fn sanitize_assessment(
    assessment: VendorAssessment,
    valid_requirement_ids: &HashSet<String>,
) -> VendorAssessment {
    let requirements = assessment
        .requirements
        .into_iter()
        .filter(|(id, _)| valid_requirement_ids.contains(id))
        .collect();
    VendorAssessment {
        vendor_id: assessment.vendor_id,
        requirements,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vendor::{ComplianceStatus, RequirementAssessment, VendorId};

    #[test]
    fn sanitize_drops_unknown_requirements() {
        let valid = HashSet::from(["r1".to_string()]);
        let assessment = VendorAssessment {
            vendor_id: VendorId::new("v1"),
            requirements: HashMap::from([
                (
                    "r1".to_string(),
                    RequirementAssessment {
                        requirement_id: "r1".to_string(),
                        status: ComplianceStatus::Compliant,
                        notes: None,
                    },
                ),
                (
                    "unknown".to_string(),
                    RequirementAssessment {
                        requirement_id: "unknown".to_string(),
                        status: ComplianceStatus::Partial,
                        notes: None,
                    },
                ),
            ]),
        };
        let cleaned = sanitize_assessment(assessment, &valid);
        assert_eq!(cleaned.requirements.len(), 1);
        assert!(cleaned.requirements.contains_key("r1"));
    }
}
