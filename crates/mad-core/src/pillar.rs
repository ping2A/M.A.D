use serde::{Deserialize, Serialize};

/// Stable identifier for a criteria group (pillar).
pub type PillarId = String;

pub mod builtin {
    pub const CYBERSECURITY_DLP: &str = "cybersecurity_dlp";
    pub const DFIR: &str = "dfir";
    pub const PLATFORM_OS: &str = "platform_os";

    pub fn all() -> &'static [&'static str] {
        &[CYBERSECURITY_DLP, DFIR, PLATFORM_OS]
    }

    pub fn is_builtin(id: &str) -> bool {
        matches!(id, CYBERSECURITY_DLP | DFIR | PLATFORM_OS)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementSeverity {
    Critical,
    High,
    Medium,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: RequirementSeverity,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// How this requirement is verified during vendor evaluation (API probes, lab tests, etc.).
    #[serde(default)]
    pub evaluation_method: Option<String>,
    /// Platform APIs, MDM payloads, or protocols involved in meeting this requirement.
    #[serde(default)]
    pub technical_criteria: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pillar {
    pub id: PillarId,
    pub name: String,
    pub description: String,
    pub requirements: Vec<Requirement>,
}

impl Pillar {
    pub fn critical_count(&self) -> usize {
        self.requirements
            .iter()
            .filter(|r| r.severity == RequirementSeverity::Critical)
            .count()
    }

    pub fn requirement_count(&self) -> usize {
        self.requirements.len()
    }
}
