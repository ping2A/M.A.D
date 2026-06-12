use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::vendor_doc::{deserialize_vendor_docs, VendorDocSection};
use crate::value_stream::{deserialize_vendor_value_streams, ValueStreamEntry};
use crate::vendor::{Vendor, VendorAssessment};

/// Portable vendor list with assessments and value stream maps for import/export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorSetFile {
    pub format_version: u32,
    pub exported_at: String,
    pub vendors: Vec<Vendor>,
    pub assessments: HashMap<String, VendorAssessment>,
    /// vendor_id → value stream maps (multiple per vendor)
    #[serde(default, deserialize_with = "deserialize_vendor_value_streams")]
    pub value_streams: HashMap<String, Vec<ValueStreamEntry>>,
    #[serde(default, deserialize_with = "deserialize_vendor_docs")]
    pub vendor_docs: HashMap<String, Vec<VendorDocSection>>,
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
    #[serde(default)]
    pub value_streams_imported: usize,
    #[serde(default, alias = "privacy_profiles_imported")]
    pub vendor_docs_imported: usize,
}

impl VendorSetFile {
    pub fn new(
        exported_at: impl Into<String>,
        vendors: Vec<Vendor>,
        assessments: HashMap<String, VendorAssessment>,
        value_streams: HashMap<String, Vec<ValueStreamEntry>>,
        vendor_docs: HashMap<String, Vec<VendorDocSection>>,
    ) -> Self {
        Self {
            format_version: 2,
            exported_at: exported_at.into(),
            vendors,
            assessments,
            value_streams,
            vendor_docs,
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

    #[test]
    fn vendor_set_roundtrips_value_streams() {
        use crate::value_stream::{ValueStreamEntry, ValueStreamMap};

        let file = VendorSetFile::new(
            "test",
            vec![],
            HashMap::new(),
            HashMap::from([(
                "intune".into(),
                vec![ValueStreamEntry {
                    id: "vsm-1".into(),
                    name: "Enrollment".into(),
                    map: ValueStreamMap::default(),
                }],
            )]),
            HashMap::new(),
        );
        let json = serde_json::to_string(&file).expect("serialize");
        let parsed: VendorSetFile = serde_json::from_str(&json).expect("parse");
        assert_eq!(parsed.value_streams.get("intune").map(|v| v.len()), Some(1));
    }

    #[test]
    fn vendor_set_roundtrips_vendor_docs() {
        use crate::vendor_doc::{VendorDocItem, VendorDocSection};

        let file = VendorSetFile::new(
            "test",
            vec![],
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(
                "intune".into(),
                vec![VendorDocSection {
                    id: "vdoc-1".into(),
                    name: "Privacy".into(),
                    color: None,
                    overview: None,
                    items: vec![VendorDocItem {
                        id: "item-1".into(),
                        group: None,
                        color: None,
                        title: "Auth".into(),
                        description: None,
                        notes: None,
                    }],
                }],
            )]),
        );
        let json = serde_json::to_string(&file).expect("serialize");
        let parsed: VendorSetFile = serde_json::from_str(&json).expect("parse");
        assert_eq!(
            parsed.vendor_docs.get("intune").map(|v| v.len()),
            Some(1)
        );
    }
}
