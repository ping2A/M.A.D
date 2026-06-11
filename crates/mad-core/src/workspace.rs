use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::pillar::{builtin, Pillar, Requirement, RequirementSeverity};
use crate::policy::PolicyBundle;
use crate::pricing::ProcurementConfig;
use crate::scoring::ScoringConfig;
use crate::value_stream::{deserialize_vendor_value_streams, ValueStreamEntry, ValueStreamMap};
use crate::vendor::{Vendor, VendorAssessment, VendorId};
use crate::vendor_set::{
    sanitize_assessment, VendorImportMode, VendorImportResult, VendorSetFile,
};

/// Mutable evaluation workspace: criteria, vendors, assessments, and scoring rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationWorkspace {
    pub policy_version: String,
    pub scoring: ScoringConfig,
    #[serde(default)]
    pub procurement: ProcurementConfig,
    pub pillars: Vec<Pillar>,
    pub vendors: Vec<Vendor>,
    /// vendor_id → assessment
    pub assessments: HashMap<String, VendorAssessment>,
    /// vendor_id → value stream maps
    #[serde(default, deserialize_with = "deserialize_vendor_value_streams")]
    pub value_streams: HashMap<String, Vec<ValueStreamEntry>>,
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
            procurement: ProcurementConfig::default(),
            pillars: bundle.pillars.clone(),
            vendors: Vec::new(),
            assessments: HashMap::new(),
            value_streams: HashMap::new(),
        }
    }

    pub fn value_streams_for(&self, vendor_id: &str) -> &[ValueStreamEntry] {
        self.value_streams
            .get(vendor_id)
            .map(|entries| entries.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_value_stream_entry(&self, vendor_id: &str, stream_id: &str) -> Option<ValueStreamEntry> {
        self.value_streams
            .get(vendor_id)?
            .iter()
            .find(|entry| entry.id == stream_id)
            .cloned()
    }

    pub fn create_value_stream(&mut self, vendor_id: &str, name: impl Into<String>) -> Option<ValueStreamEntry> {
        if !self.vendors.iter().any(|v| v.id.0 == vendor_id) {
            return None;
        }
        let entry = ValueStreamEntry::new(name);
        self.value_streams
            .entry(vendor_id.to_string())
            .or_default()
            .push(entry.clone());
        Some(entry)
    }

    pub fn upsert_value_stream_entry(
        &mut self,
        vendor_id: &str,
        entry: ValueStreamEntry,
    ) -> bool {
        if !self.vendors.iter().any(|v| v.id.0 == vendor_id) {
            return false;
        }
        if entry.is_empty() {
            return self.remove_value_stream(vendor_id, &entry.id);
        }
        let streams = self.value_streams.entry(vendor_id.to_string()).or_default();
        if let Some(existing) = streams.iter_mut().find(|e| e.id == entry.id) {
            *existing = entry;
        } else {
            streams.push(entry);
        }
        true
    }

    /// Legacy single-map update (uses id `default`).
    pub fn set_value_stream(&mut self, vendor_id: &str, map: ValueStreamMap) -> bool {
        let stream_id = self
            .value_streams
            .get(vendor_id)
            .and_then(|entries| entries.first().map(|e| e.id.clone()))
            .unwrap_or_else(|| "default".into());
        let name = self
            .value_streams
            .get(vendor_id)
            .and_then(|entries| entries.first().map(|e| e.name.clone()))
            .unwrap_or_else(|| "Value stream".into());
        self.upsert_value_stream_entry(
            vendor_id,
            ValueStreamEntry {
                id: stream_id,
                name,
                map,
            },
        )
    }

    pub fn remove_value_stream(&mut self, vendor_id: &str, stream_id: &str) -> bool {
        let Some(streams) = self.value_streams.get_mut(vendor_id) else {
            return false;
        };
        let before = streams.len();
        streams.retain(|entry| entry.id != stream_id);
        let removed = before != streams.len();
        if streams.is_empty() {
            self.value_streams.remove(vendor_id);
        }
        removed
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

    pub fn add_pillar(&mut self, pillar: Pillar) -> bool {
        if pillar.id.trim().is_empty() {
            return false;
        }
        if self.pillars.iter().any(|p| p.id == pillar.id) {
            return false;
        }
        self.pillars.push(pillar);
        true
    }

    pub fn update_pillar(&mut self, pillar_id: &str, name: String, description: String) -> bool {
        let Some(pillar) = self.pillars.iter_mut().find(|p| p.id == pillar_id) else {
            return false;
        };
        pillar.name = name;
        pillar.description = description;
        true
    }

    pub fn remove_pillar(&mut self, pillar_id: &str) -> bool {
        if builtin::is_builtin(pillar_id) {
            return false;
        }
        let Some(pos) = self.pillars.iter().position(|p| p.id == pillar_id) else {
            return false;
        };
        let removed_req_ids: HashSet<String> = self.pillars[pos]
            .requirements
            .iter()
            .map(|r| r.id.clone())
            .collect();
        self.pillars.remove(pos);
        for assessment in self.assessments.values_mut() {
            for req_id in &removed_req_ids {
                assessment.requirements.remove(req_id);
            }
        }
        true
    }

    pub fn add_requirement(&mut self, pillar_id: &str, requirement: Requirement) -> bool {
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

    pub fn update_requirement(
        &mut self,
        requirement_id: &str,
        pillar_id: &str,
        requirement: Requirement,
    ) -> bool {
        let mut old_pillar_idx = None;
        let mut req_idx = None;
        for (pi, pillar) in self.pillars.iter().enumerate() {
            if let Some(ri) = pillar.requirements.iter().position(|r| r.id == requirement_id) {
                old_pillar_idx = Some(pi);
                req_idx = Some(ri);
                break;
            }
        }
        let (old_pi, ri) = match (old_pillar_idx, req_idx) {
            (Some(pi), Some(ri)) => (pi, ri),
            _ => return false,
        };

        if requirement.id != requirement_id
            && self
                .pillars
                .iter()
                .flat_map(|p| &p.requirements)
                .any(|r| r.id == requirement.id)
        {
            return false;
        }

        let new_pillar_idx = self
            .pillars
            .iter()
            .position(|p| p.id == pillar_id)
            .unwrap_or(old_pi);

        let removed = self.pillars[old_pi].requirements.remove(ri);
        self.pillars[new_pillar_idx]
            .requirements
            .push(requirement.clone());

        if requirement.id != requirement_id {
            for assessment in self.assessments.values_mut() {
                if let Some(entry) = assessment.requirements.remove(requirement_id) {
                    assessment.requirements.insert(
                        requirement.id.clone(),
                        crate::vendor::RequirementAssessment {
                            requirement_id: requirement.id.clone(),
                            status: entry.status,
                            notes: entry.notes,
                        },
                    );
                }
            }
        }

        let _ = removed;
        true
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

    pub fn add_vendor(&mut self, vendor: Vendor) -> bool {
        if self.vendors.iter().any(|v| v.id == vendor.id) {
            return false;
        }
        self.vendors.push(vendor);
        true
    }

    pub fn update_vendor(&mut self, vendor_id: &str, vendor: Vendor) -> bool {
        if vendor.id.0 != vendor_id
            && self.vendors.iter().any(|v| v.id == vendor.id)
        {
            return false;
        }
        let Some(idx) = self.vendors.iter().position(|v| v.id.0 == vendor_id) else {
            return false;
        };
        if vendor.id.0 != vendor_id {
            if let Some(assessment) = self.assessments.remove(vendor_id) {
                let mut updated = assessment;
                updated.vendor_id = vendor.id.clone();
                self.assessments.insert(vendor.id.0.clone(), updated);
            }
        }
        self.vendors[idx] = vendor;
        true
    }

    pub fn remove_vendor(&mut self, vendor_id: &str) -> bool {
        let before = self.vendors.len();
        self.vendors.retain(|v| v.id.0 != vendor_id);
        if self.vendors.len() < before {
            self.assessments.remove(vendor_id);
            self.value_streams.remove(vendor_id);
            true
        } else {
            false
        }
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

    pub fn requirement_ids(&self) -> HashSet<String> {
        self.pillars
            .iter()
            .flat_map(|p| p.requirements.iter().map(|r| r.id.clone()))
            .collect()
    }

    pub fn export_vendor_set(&self, exported_at: impl Into<String>) -> VendorSetFile {
        VendorSetFile::new(
            exported_at,
            self.vendors.clone(),
            self.assessments.clone(),
        )
    }

    pub fn import_vendor_set(
        &mut self,
        file: VendorSetFile,
        mode: VendorImportMode,
    ) -> VendorImportResult {
        let valid_ids = self.requirement_ids();
        let mut result = VendorImportResult {
            added: 0,
            updated: 0,
            skipped: 0,
            removed: 0,
        };

        if mode == VendorImportMode::Replace {
            result.removed = self.vendors.len();
            self.vendors.clear();
            self.assessments.clear();
        }

        for vendor in file.vendors {
            let vendor_id = vendor.id.0.clone();
            let assessment = file
                .assessments
                .get(&vendor_id)
                .cloned()
                .unwrap_or_else(|| VendorAssessment {
                    vendor_id: vendor.id.clone(),
                    requirements: HashMap::new(),
                });
            let assessment = sanitize_assessment(assessment, &valid_ids);

            if let Some(idx) = self.vendors.iter().position(|v| v.id.0 == vendor_id) {
                self.vendors[idx] = vendor.clone();
                let entry = self
                    .assessments
                    .entry(vendor_id)
                    .or_insert_with(|| VendorAssessment {
                        vendor_id: vendor.id.clone(),
                        requirements: HashMap::new(),
                    });
                if mode == VendorImportMode::Merge {
                    for (req_id, req_assessment) in assessment.requirements {
                        entry.requirements.insert(req_id, req_assessment);
                    }
                } else {
                    *entry = assessment;
                }
                result.updated += 1;
            } else if self.add_vendor(vendor) {
                self.assessments.insert(vendor_id, assessment);
                result.added += 1;
            } else {
                result.skipped += 1;
            }
        }

        result
    }
}
