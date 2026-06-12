use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

/// Single checklist row inside a vendor documentation section.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VendorDocItem {
    pub id: String,
    /// Optional user-defined subgroup (e.g. "Application", "End user").
    #[serde(default)]
    pub group: Option<String>,
    /// Preset id (`critical`, `warning`, …) or custom `#rrggbb` hex accent.
    #[serde(default)]
    pub color: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// User-defined documentation block attached to a vendor (not scored).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VendorDocSection {
    pub id: String,
    pub name: String,
    /// Section accent color (preset id or `#rrggbb` hex).
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub items: Vec<VendorDocItem>,
}

impl VendorDocSection {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: new_vendor_doc_id(),
            name: name.into(),
            color: None,
            overview: None,
            items: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.name.trim().is_empty()
            && self.overview.as_ref().is_none_or(|s| s.trim().is_empty())
            && self.items.is_empty()
    }
}

fn unique_doc_suffix() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    format!("{millis}-{seq}")
}

pub fn new_vendor_doc_id() -> String {
    format!("vdoc-{}", unique_doc_suffix())
}

pub fn new_vendor_doc_item_id() -> String {
    format!("vdoc-item-{}", unique_doc_suffix())
}

/// Assigns fresh ids when duplicates exist (legacy templates created in the same millisecond).
pub fn dedupe_vendor_doc_item_ids(items: &mut Vec<VendorDocItem>) -> bool {
    let mut seen = std::collections::HashSet::new();
    let mut changed = false;
    for item in items.iter_mut() {
        if seen.insert(item.id.clone()) {
            continue;
        }
        item.id = new_vendor_doc_item_id();
        seen.insert(item.id.clone());
        changed = true;
    }
    changed
}

pub fn normalize_vendor_doc_section(section: &mut VendorDocSection) -> bool {
    dedupe_vendor_doc_item_ids(&mut section.items)
}

/// Example starter section for MDM privacy reviews.
pub fn mdm_privacy_template_section() -> VendorDocSection {
    VendorDocSection {
        id: new_vendor_doc_id(),
        name: "Privacy".into(),
        color: Some("info".into()),
        overview: Some(
            "Document privacy posture for this MDM vendor. This section is informational only and does not affect capability scores.".into(),
        ),
        items: vec![
            VendorDocItem {
                id: new_vendor_doc_item_id(),
                group: Some("Application".into()),
                color: Some("critical".into()),
                title: "Admin console authentication".into(),
                description: Some("MFA, SSO/SAML, session timeout, and privileged access controls for administrators.".into()),
                notes: None,
            },
            VendorDocItem {
                id: new_vendor_doc_item_id(),
                group: Some("Application".into()),
                color: Some("warning".into()),
                title: "Application telemetry & logging".into(),
                description: Some("What operational telemetry the vendor collects from the management platform and retention periods.".into()),
                notes: None,
            },
            VendorDocItem {
                id: new_vendor_doc_item_id(),
                group: Some("Application".into()),
                color: Some("warning".into()),
                title: "Subprocessors & data residency".into(),
                description: Some("List of subprocessors, regions where admin/tenant data is stored, and DPA availability.".into()),
                notes: None,
            },
            VendorDocItem {
                id: new_vendor_doc_item_id(),
                group: Some("End user".into()),
                color: Some("info".into()),
                title: "Device inventory data".into(),
                description: Some("Which device attributes are collected (hardware IDs, installed apps, OS version) and purpose.".into()),
                notes: None,
            },
            VendorDocItem {
                id: new_vendor_doc_item_id(),
                group: Some("End user".into()),
                color: Some("critical".into()),
                title: "Location & usage data".into(),
                description: Some("Whether geolocation or usage analytics are collected from end-user devices.".into()),
                notes: None,
            },
            VendorDocItem {
                id: new_vendor_doc_item_id(),
                group: Some("End user".into()),
                color: Some("success".into()),
                title: "Employee transparency & consent".into(),
                description: Some("Privacy notice, work-profile boundaries, and consent mechanisms for supervised devices.".into()),
                notes: None,
            },
        ],
    }
}

/// Deserializes vendor → doc sections, accepting legacy single-profile JSON per vendor.
pub fn deserialize_vendor_docs<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Vec<VendorDocSection>>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: HashMap<String, serde_json::Value> =
        HashMap::deserialize(deserializer).unwrap_or_default();
    Ok(raw
        .into_iter()
        .map(|(vendor_id, value)| (vendor_id, parse_vendor_doc_sections(value)))
        .collect())
}

fn parse_vendor_doc_sections(value: serde_json::Value) -> Vec<VendorDocSection> {
    match value {
        serde_json::Value::Array(items) => items
            .into_iter()
            .filter_map(|item| serde_json::from_value::<VendorDocSection>(item).ok())
            .collect(),
        serde_json::Value::Object(obj) => {
            if obj.contains_key("name") || obj.contains_key("items") || obj.contains_key("overview") {
                if let Ok(section) = legacy_privacy_profile_to_section(&obj) {
                    if !section.is_empty() {
                        return vec![section];
                    }
                }
                serde_json::from_value(serde_json::Value::Object(obj))
                    .ok()
                    .into_iter()
                    .collect()
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

fn legacy_privacy_profile_to_section(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<VendorDocSection, serde_json::Error> {
    #[derive(Deserialize)]
    struct LegacyItem {
        id: String,
        #[serde(default)]
        category: Option<LegacyCategory>,
        title: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        notes: Option<String>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum LegacyCategory {
        Application,
        User,
    }

    #[derive(Deserialize)]
    struct LegacyProfile {
        #[serde(default)]
        overview: Option<String>,
        #[serde(default)]
        items: Vec<LegacyItem>,
    }

    let profile: LegacyProfile = serde_json::from_value(serde_json::Value::Object(obj.clone()))?;
    Ok(VendorDocSection {
        id: new_vendor_doc_id(),
        name: "Privacy".into(),
        color: None,
        overview: profile.overview,
        items: profile
            .items
            .into_iter()
            .map(|item| VendorDocItem {
                id: item.id,
                color: None,
                group: item.category.map(|c| match c {
                    LegacyCategory::Application => "Application".into(),
                    LegacyCategory::User => "End user".into(),
                }),
                title: item.title,
                description: item.description,
                notes: item.notes,
            })
            .collect(),
    })
}

/// Merges legacy `vendor_privacy` JSON into `vendor_docs` when loading older files.
pub fn migrate_legacy_vendor_privacy(mut value: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = value.as_object_mut() {
        if let Some(privacy) = obj.remove("vendor_privacy") {
            merge_privacy_into_docs(obj, privacy);
        }
        if let Some(workspace) = obj.get_mut("workspace") {
            let migrated = migrate_legacy_vendor_privacy(workspace.take());
            *workspace = migrated;
        }
    }
    value
}

fn merge_privacy_into_docs(obj: &mut serde_json::Map<String, serde_json::Value>, privacy: serde_json::Value) {
    let docs = obj
        .entry("vendor_docs")
        .or_insert_with(|| serde_json::json!({}));
    let Some(docs_map) = docs.as_object_mut() else {
        return;
    };
    let Some(privacy_map) = privacy.as_object() else {
        return;
    };
    for (vendor_id, profile) in privacy_map {
        if docs_map.contains_key(vendor_id) {
            continue;
        }
        let sections = parse_vendor_doc_sections(profile.clone());
        if !sections.is_empty() {
            docs_map.insert(
                vendor_id.clone(),
                serde_json::to_value(sections).unwrap_or_default(),
            );
        }
    }
}

/// Resolves a preset id or `#rrggbb` hex to a display color.
pub fn resolve_doc_color(color: Option<&str>) -> Option<String> {
    let raw = color.map(str::trim).filter(|s| !s.is_empty())?;
    if raw.starts_with('#') && (raw.len() == 4 || raw.len() == 7) {
        return Some(raw.to_ascii_lowercase());
    }
    Some(
        match raw {
            "critical" => "#dc3545",
            "warning" => "#e6a800",
            "info" => "#00b4d8",
            "success" => "#28a745",
            "neutral" => "#6c757d",
            _ => return None,
        }
        .into(),
    )
}

pub fn vendor_doc_count(docs: &HashMap<String, Vec<VendorDocSection>>) -> usize {
    docs.values().map(|sections| sections.len()).sum()
}

pub fn any_vendor_docs(docs: &HashMap<String, Vec<VendorDocSection>>) -> bool {
    docs.values().any(|sections| sections.iter().any(|s| !s.is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privacy_template_items_have_unique_ids() {
        let section = mdm_privacy_template_section();
        let mut ids = std::collections::HashSet::new();
        for item in &section.items {
            assert!(ids.insert(item.id.clone()), "duplicate item id: {}", item.id);
        }
    }

    #[test]
    fn dedupe_assigns_unique_ids() {
        let mut items = vec![
            VendorDocItem {
                id: "same".into(),
                group: None,
                color: None,
                title: "A".into(),
                description: None,
                notes: None,
            },
            VendorDocItem {
                id: "same".into(),
                group: None,
                color: None,
                title: "B".into(),
                description: None,
                notes: None,
            },
        ];
        assert!(dedupe_vendor_doc_item_ids(&mut items));
        assert_ne!(items[0].id, items[1].id);
    }

    #[test]
    fn migrates_legacy_privacy_profile() {
        let json = r#"{
            "vendor_privacy": {
                "intune": {
                    "overview": "test",
                    "items": [{
                        "id": "p1",
                        "category": "application",
                        "title": "Auth",
                        "description": null,
                        "notes": null
                    }]
                }
            }
        }"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let migrated = migrate_legacy_vendor_privacy(value);
        let docs = migrated["vendor_docs"]["intune"].as_array().unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0]["name"], "Privacy");
    }
}
