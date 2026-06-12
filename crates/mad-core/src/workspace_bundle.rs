use serde::{Deserialize, Serialize};

use crate::vendor_doc::migrate_legacy_vendor_privacy;
use crate::vendor_set::{sanitize_assessment, VendorImportMode, VendorImportResult, VendorSetFile};
use crate::workspace::EvaluationWorkspace;

/// Full portable snapshot: criteria, vendors, assessments, scoring, and procurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceBundle {
    pub format_version: u32,
    pub exported_at: String,
    pub workspace: EvaluationWorkspace,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceImportResult {
    pub kind: &'static str,
    pub pillars: usize,
    pub requirements: usize,
    pub vendors: usize,
    pub assessments: usize,
    pub value_stream_maps: usize,
    pub vendor_doc_sections: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_result: Option<VendorImportResult>,
}

#[derive(Debug)]
pub enum ParsedWorkspaceImport {
    Bundle(WorkspaceBundle),
    RawWorkspace(EvaluationWorkspace),
    VendorSet(VendorSetFile),
}

impl WorkspaceBundle {
    pub fn new(exported_at: impl Into<String>, workspace: EvaluationWorkspace) -> Self {
        Self {
            format_version: 1,
            exported_at: exported_at.into(),
            workspace,
        }
    }
}

impl EvaluationWorkspace {
    pub fn export_bundle(&self, exported_at: impl Into<String>) -> WorkspaceBundle {
        WorkspaceBundle::new(exported_at, self.clone())
    }

    pub fn replace_from_workspace(&mut self, mut workspace: EvaluationWorkspace) {
        let valid_ids = workspace.requirement_ids();
        for assessment in workspace.assessments.values_mut() {
            *assessment = sanitize_assessment(assessment.clone(), &valid_ids);
        }
        *self = workspace;
    }

    pub fn import_parsed(
        &mut self,
        parsed: ParsedWorkspaceImport,
        vendor_mode: VendorImportMode,
    ) -> WorkspaceImportResult {
        match parsed {
            ParsedWorkspaceImport::Bundle(bundle) => {
                self.replace_from_workspace(bundle.workspace);
                self.import_summary("full", None)
            }
            ParsedWorkspaceImport::RawWorkspace(workspace) => {
                self.replace_from_workspace(workspace);
                self.import_summary("full", None)
            }
            ParsedWorkspaceImport::VendorSet(file) => {
                let vendor_result = self.import_vendor_set(file, vendor_mode);
                self.import_summary("vendors", Some(vendor_result))
            }
        }
    }

    fn import_summary(
        &self,
        kind: &'static str,
        vendor_result: Option<VendorImportResult>,
    ) -> WorkspaceImportResult {
        WorkspaceImportResult {
            kind,
            pillars: self.pillars.len(),
            requirements: self.total_requirements(),
            vendors: self.vendors.len(),
            assessments: self.assessments.len(),
            value_stream_maps: self.value_stream_map_count(),
            vendor_doc_sections: self.vendor_doc_section_count(),
            vendor_result,
        }
    }
}

pub fn parse_workspace_import(json: &str) -> Result<ParsedWorkspaceImport, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("invalid JSON: {e}"))?;
    let value = migrate_legacy_vendor_privacy(value);

    if value.get("workspace").is_some() {
        let bundle: WorkspaceBundle =
            serde_json::from_value(value).map_err(|e| format!("invalid workspace bundle: {e}"))?;
        return Ok(ParsedWorkspaceImport::Bundle(bundle));
    }

    if value.get("pillars").is_some() {
        let workspace: EvaluationWorkspace = serde_json::from_value(value)
            .map_err(|e| format!("invalid workspace data: {e}"))?;
        return Ok(ParsedWorkspaceImport::RawWorkspace(workspace));
    }

    if value.get("vendors").is_some() {
        let vendor_set: VendorSetFile =
            serde_json::from_value(value).map_err(|e| format!("invalid vendor set: {e}"))?;
        return Ok(ParsedWorkspaceImport::VendorSet(vendor_set));
    }

    Err("unrecognized import format: expected workspace bundle, workspace, or vendor set".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::PolicyBundle;

    #[test]
    fn roundtrip_bundle() {
        let dir = std::path::Path::new("policies");
        if !dir.exists() {
            return;
        }
        let bundle = PolicyBundle::load_dir(dir).expect("policy");
        let mut ws = EvaluationWorkspace::from_policy(&bundle);
        ws.policy_version = "test".into();
        let exported = ws.export_bundle("123");
        let json = serde_json::to_string(&exported).expect("serialize");
        let parsed = parse_workspace_import(&json).expect("parse");
        let mut target = EvaluationWorkspace::from_policy(&bundle);
        target.import_parsed(parsed, VendorImportMode::Replace);
        assert_eq!(target.policy_version, "test");
    }

    #[test]
    fn roundtrip_bundle_with_value_streams() {
        use crate::value_stream::{ValueStreamEntry, ValueStreamMap, VsmNode, VsmNodeType};

        let dir = std::path::Path::new("policies");
        if !dir.exists() {
            return;
        }
        let bundle = PolicyBundle::load_dir(dir).expect("policy");
        let mut ws = EvaluationWorkspace::from_policy(&bundle);
        let vendor_id = ws
            .vendors
            .first()
            .map(|v| v.id.0.clone())
            .unwrap_or_else(|| "vendor-a".into());
        ws.value_streams.insert(
            vendor_id.clone(),
            vec![ValueStreamEntry {
                id: "vsm-test".into(),
                name: "Enrollment".into(),
                map: ValueStreamMap {
                    nodes: vec![VsmNode {
                        id: "n1".into(),
                        label: "Start".into(),
                        node_type: VsmNodeType::Process,
                        x: 0.0,
                        y: 0.0,
                        width: 180.0,
                        height: 72.0,
                        notes: None,
                        role: None,
                        lead_time_minutes: None,
                        cycle_time_minutes: None,
                        author: None,
                    }],
                    edges: vec![],
                    messages: vec![],
                    flow_types: vec![],
                },
            }],
        );

        let json = serde_json::to_string(&ws.export_bundle("vsm-test")).expect("serialize");
        let parsed = parse_workspace_import(&json).expect("parse");
        let mut target = EvaluationWorkspace::from_policy(&bundle);
        target.import_parsed(parsed, VendorImportMode::Replace);
        let streams = target.value_streams.get(&vendor_id).expect("streams");
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].name, "Enrollment");
    }

    #[test]
    fn roundtrip_bundle_with_vendor_docs() {
        use crate::vendor_doc::{VendorDocItem, VendorDocSection};

        let dir = std::path::Path::new("policies");
        if !dir.exists() {
            return;
        }
        let bundle = PolicyBundle::load_dir(dir).expect("policy");
        let mut ws = EvaluationWorkspace::from_policy(&bundle);
        let vendor_id = ws
            .vendors
            .first()
            .map(|v| v.id.0.clone())
            .unwrap_or_else(|| "vendor-a".into());
        ws.vendor_docs.insert(
            vendor_id.clone(),
            vec![VendorDocSection {
                id: "vdoc-test".into(),
                name: "Security".into(),
                color: Some("info".into()),
                overview: Some("Overview".into()),
                items: vec![VendorDocItem {
                    id: "item-1".into(),
                    group: None,
                    color: None,
                    title: "MFA".into(),
                    description: Some("Required".into()),
                    notes: None,
                }],
            }],
        );

        let json = serde_json::to_string(&ws.export_bundle("docs-test")).expect("serialize");
        let parsed = parse_workspace_import(&json).expect("parse");
        let mut target = EvaluationWorkspace::from_policy(&bundle);
        let result = target.import_parsed(parsed, VendorImportMode::Replace);
        let docs = target.vendor_docs.get(&vendor_id).expect("docs");
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].name, "Security");
        assert_eq!(result.vendor_doc_sections, 1);
    }

    #[test]
    fn imports_legacy_vendor_set_without_description() {
        let dir = std::path::Path::new("policies");
        if !dir.exists() {
            return;
        }
        let bundle = PolicyBundle::load_dir(dir).expect("policy");
        let json = r#"{
            "format_version": 2,
            "exported_at": "legacy",
            "vendors": [{"id": "legacy-co", "name": "Legacy Co"}],
            "assessments": {}
        }"#;
        let parsed = parse_workspace_import(json).expect("parse");
        let mut ws = EvaluationWorkspace::from_policy(&bundle);
        ws.import_parsed(parsed, VendorImportMode::Merge);
        let vendor = ws.vendors.iter().find(|v| v.id.0 == "legacy-co").expect("vendor");
        assert_eq!(vendor.description, "");
    }
}
