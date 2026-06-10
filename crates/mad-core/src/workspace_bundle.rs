use serde::{Deserialize, Serialize};

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
            vendor_result,
        }
    }
}

pub fn parse_workspace_import(json: &str) -> Result<ParsedWorkspaceImport, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("invalid JSON: {e}"))?;

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
}
