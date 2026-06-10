use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PillarId {
    CybersecurityDlp,
    Dfir,
    PlatformOs,
}

impl PillarId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CybersecurityDlp => "cybersecurity_dlp",
            Self::Dfir => "dfir",
            Self::PlatformOs => "platform_os",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::CybersecurityDlp => "Cybersecurity & Data Loss Prevention",
            Self::Dfir => "Digital Forensics & Incident Response",
            Self::PlatformOs => "Platform & OS Native Support",
        }
    }

    pub fn all() -> &'static [PillarId] {
        &[
            PillarId::CybersecurityDlp,
            PillarId::Dfir,
            PillarId::PlatformOs,
        ]
    }
}

impl std::fmt::Display for PillarId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
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
